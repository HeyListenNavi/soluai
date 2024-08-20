use std::{error::Error, io::Cursor, sync::{Arc, LazyLock}};
use bytes::Bytes;
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageReader};
use ndarray::{s, Array, Axis, Ix4,};
use ort::{inputs, CUDAExecutionProvider, OpenVINOExecutionProvider, Session, SessionOutputs};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inference {
    bounding_box: BoundingBox,
    object: String,
    probability: f32
}

#[rustfmt::skip]
const YOLOV8_CLASS_LABELS: [&str; 3] = [
    "handgun",
    "knife",
    "long weapon"
];

static SESSION: LazyLock<Arc<Session>> = LazyLock::new(|| {
    let session = Session::builder()
        .expect("Failed to load Session")
        .with_execution_providers([
            CUDAExecutionProvider::default().build(),
            OpenVINOExecutionProvider::default().build()
        ])
        .expect("Failed to load Execution Provider")
        .commit_from_file("data/model.onnx")
        .expect("Failed to load ONNX model");
    Arc::new(session)
});


fn intersection(box1: &BoundingBox, box2: &BoundingBox) -> f32 {
    let intersection = (box1.x2.min(box2.x2) - box1.x1.max(box2.x1)) * (box1.y2.min(box2.y2) - box1.y1.max(box2.y1));
    return intersection;
}

fn union(box1: &BoundingBox, box2: &BoundingBox) -> f32 {
    let union = ((box1.x2 - box1.x1) * (box1.y2 - box1.y1)) + ((box2.x2 - box2.x1) * (box2.y2 - box2.y1)) - intersection(box1, box2);
    return union;
}

fn model_input_pre_processing(original_image: DynamicImage) -> Array<f32, Ix4> {
    // Open -> Store height and width -> Resize image -> Normalize each pixel -> Store image pixels in array
    let image = original_image.resize_exact(640, 640, FilterType::CatmullRom);
    let mut input = Array::zeros((1, 3, 640, 640));
    for pixel in image.pixels() {
        let x = pixel.0 as usize;
        let y = pixel.1 as usize;
        let [r, g, b, _] = pixel.2.0;
        input[[0, 0, y, x]] = (r as f32) / 255.0;
        input[[0, 1, y, x]] = (g as f32) / 255.0;
        input[[0, 2, y, x]] = (b as f32) / 255.0;
    }
    return input;
}

fn model_output_post_processing(outputs: SessionOutputs, original_image_width: u32, original_image_height: u32) -> Result<Vec<Inference>, Box<dyn Error>> {
    // Get the output0 -> Extract it into a tensor of type f32 -> Transpose the tensor -> Create an owned copy for the output variable 
    let extracted_tensor = match outputs["output0"].try_extract_tensor::<f32>() {
        Ok(tensor) => tensor.t().into_owned(),
        Err(e) => {
            println!("Couldn't extract output into tensor: {:?}", e);
            return Err(e.into());
        }
    };

    // Create boxes
    let mut boxes = Vec::new();
    // Create 2 dimensional array using all the elements from the first and second dimension
    let output = extracted_tensor.slice(s![.., .., 0]);
    for row in output.axis_iter(Axis(0)) {
        let row: Vec<f32> = row.iter().copied().collect();
        let (class_id, probability) = row.iter()
            // skip bounding box coordinates
            .skip(4)
            .enumerate()
            // dereference any possible values
            .map(|(index, probability)| (index, *probability))
            // gets output with highest probability 
            .reduce(|accumulator, row| if row.1 > accumulator.1 { row } else { accumulator })
            .unwrap();

        if probability < 0.75 {
            continue;
        }

        let class = YOLOV8_CLASS_LABELS[class_id];
        let xc = row[0] / 640.0 * (original_image_width as f32);
        let yc = row[1] / 640.0 * (original_image_height as f32);
        let w = row[2] / 640.0 * (original_image_width as f32);
        let h = row[3] / 640.0 * (original_image_height as f32);
        
        boxes.push(
            Inference {
                bounding_box: BoundingBox {
                    x1: xc - w / 2.0,
                    y1: yc - h / 2.0,
                    x2: xc + w / 2.0,
                    y2: yc + h / 2.0
                },
                object: class.to_string(),
                probability: probability
            }
        );
    }

    // -----------------------------
    // Non-Maximum Suppression (NMS)
    // -----------------------------

    let mut result = Vec::new();
    
    // Sort boxes in descending order based on their probability and filter the boxes
    boxes.sort_by(|box1, box2| box2.probability.total_cmp(&box1.probability));
    while !boxes.is_empty() {
        result.push(boxes[0].clone());
        boxes = boxes
            .iter()
            .filter(|box1| intersection(&boxes[0].bounding_box, &box1.bounding_box) / union(&boxes[0].bounding_box, &box1.bounding_box) < 0.7)
            .cloned()
            .collect();
    }
    return Ok(result);
}

pub fn run_model(image_data: Bytes) -> Result<Vec<Inference>, Box<dyn Error>> {
    ort::init()
        .commit()?;

    // ----------------------------
    //    Pre-process the image    
    // ----------------------------

    let original_image = match ImageReader::new(Cursor::new(image_data)).with_guessed_format() {
        Ok(image) => image.decode().expect("Format not suported"),
        Err(e) => {
            println!("Error opening image: {:?}", e);
            return Err(e.into());
        }
    };
    let (image_width, image_height) = (original_image.width(), original_image.height());
    let model_input = model_input_pre_processing(original_image);

    // ----------------------------
    //        Run the model        
    // ----------------------------
    
    let input_values = match inputs![model_input] {
        Ok(values) => values,
        Err(e) => {
            println!("Error converting image Array into Session Input Values: {:?}", e);
            return Err(e.into());
        }
    };

    let session: &Session = &*SESSION;

    let outputs = match session.run(input_values) {
        Ok(output) => output,
        Err(e) => {
            println!("Error running session: {:?}", e);
            return Err(e.into());
        }
    };

    // ----------------------------
    //     Process the output      
    // ----------------------------

    let inferences = match model_output_post_processing(outputs, image_width, image_height) {
        Ok(inferences) => inferences,
        Err(e) => {
            println!("Error post-processing Session Outputs: {:?}", e);
            return Err(e.into());
        }
    };

    return Ok(inferences);
}