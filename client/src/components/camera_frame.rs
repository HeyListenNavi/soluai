use gloo_timers::callback::Interval;
use yew::prelude::*;
use gloo_utils::{window, document};
use web_sys::*;
use wasm_bindgen::{prelude::*, JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use js_sys::Array;

const SERVER_URL: &str = "ws://127.0.0.1:8000/api/predict";
const INTERVAL_MS: u32 = 150;

#[wasm_bindgen(module = "/assets/websockets.js")]
extern "C" {
    pub type WebSocketManager;

    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> WebSocketManager;

    #[wasm_bindgen(method)]
    pub fn connect(this: &WebSocketManager);

    #[wasm_bindgen(method)]
    pub fn send_frame_to_server(this: &WebSocketManager, video_camera: &JsValue, box_canvas: &JsValue);
}

pub enum Msg {
    SetupPredictionProcess,
    PredictionProcess,
    StopPredictionProcess,
    RenderCameras(usize),
    AddSources
}

pub struct CameraFrame {
    prediction_interval: Option<Interval>,
    cameras: usize,
    websocket_manager: WebSocketManager
}

impl Component for CameraFrame {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let websocket_manager = WebSocketManager::new(SERVER_URL);
        websocket_manager.connect();

        Self {
            prediction_interval: None,
            cameras: 1,
            websocket_manager
        }
    }
    
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetupPredictionProcess => {
                // Get list of devices from a promise
                // Get mediaDevices -> Enumarate devices -> Resolve promise

                let media_devices = match window().navigator().media_devices() {
                    Ok(devices) => devices,
                    Err(e) => {
                        console::error_1(&"Error getting media devices".into());
                        console::error_1(&e);
                        return false;
                    },
                };
                
                // Iterate through each camera
                let select_elements_list = document().query_selector_all("select#sources").unwrap_throw();
                let video_elements_list = document().query_selector_all("video#video").unwrap_throw();
                let cameras = (0..select_elements_list.length()).map(|i| {
                    (select_elements_list.item(i).unwrap_throw().unchecked_into::<HtmlSelectElement>(), video_elements_list.item(i).unwrap_throw().unchecked_into::<HtmlVideoElement>())
                });

                for (select_element, camera_element ) in cameras {
                    let camera_id = select_element.value();
                    
                    let mut constraints = MediaStreamConstraints::new();
                    let mut video_constraints = MediaTrackConstraints::new();
                    video_constraints.device_id(&camera_id.clone().into());
                    constraints.video(&video_constraints.into());
                    
                    let promise = match media_devices.get_user_media_with_constraints(&constraints) {
                        Ok(promise) => promise,
                        Err(e) => {
                            console::error_1(&"Error geting user media".into());
                            console::error_1(&e);
                            return false;
                        }
                    };

                    // Set user media as camera video source
                    spawn_local(async move {
                        match JsFuture::from(promise).await {
                            Ok(user_media) => {
                                let source = user_media.unchecked_into::<MediaStream>();
                                if camera_id.contains("Sin fuente") {
                                    camera_element.set_src_object(None);
                                } else {
                                    camera_element.set_src_object(Some(&source));
                                }
                            }
                            Err(e) => {
                                console::error_1(&"Error resolving enumarate devices promise".into());
                                console::error_1(&e);
                            }
                        }
                    })
                }

                let box_canvas_list = document().query_selector_all("canvas#canvas").unwrap_throw();
                let cameras = (0..box_canvas_list.length()).map(|i| {
                    box_canvas_list.item(i).unwrap_throw().unchecked_into::<HtmlCanvasElement>()
                });
                for box_canvas  in cameras {
                    box_canvas.set_width(640);
                    box_canvas.set_height(480);
                } 

                // Start prediction interval
                let interval_handle = {
                    let ctx_link = ctx.link().clone();
                    Interval::new(INTERVAL_MS, move || ctx_link.send_message(Msg::PredictionProcess))
                };
                
                self.prediction_interval = Some(interval_handle);
                
                return true;
            },
            Msg::PredictionProcess => {
                // Get cameras list
                let box_canvas_list = document().query_selector_all("canvas#canvas").unwrap_throw();
                let video_elements_list = document().query_selector_all("video#video").unwrap_throw();
                let cameras = (0..box_canvas_list.length()).map(|i| {
                    (box_canvas_list.item(i).unwrap_throw().unchecked_into::<HtmlCanvasElement>(), video_elements_list.item(i).unwrap_throw().unchecked_into::<HtmlVideoElement>())
                });

                // Process the frames of each camera
                for (box_canvas, camera_element ) in cameras {
                    self.websocket_manager.send_frame_to_server(&JsValue::from(camera_element), &JsValue::from(box_canvas));
                }               
                return false;
            },
            Msg::StopPredictionProcess => {
                // Drop interval
                self.prediction_interval = None;
                
                // Get elements lists 
                let select_elements_list = document().query_selector_all("select#sources").unwrap_throw();
                let video_elements_list = document().query_selector_all("video#video").unwrap_throw();
                let box_canvas_list = document().query_selector_all("canvas#canvas").unwrap_throw();
                let cameras = (0..select_elements_list.length()).map(|i| {
                    (select_elements_list.item(i).unwrap_throw().unchecked_into::<HtmlSelectElement>(), video_elements_list.item(i).unwrap_throw().unchecked_into::<HtmlVideoElement>(), box_canvas_list.item(i).unwrap_throw().unchecked_into::<HtmlCanvasElement>())
                });

                // Delete camera source -> Reset source index -> Clear box canvas
                for (select_element, camera_element, box_canvas ) in cameras {
                    // Clear cameras
                    camera_element.set_src_object(None);
                    select_element.set_selected_index(0);
                    
                    // Reset canvas
                    box_canvas.set_width(0);
                    box_canvas.set_height(0);
                    let box_canvas_ctx = box_canvas.get_context("2d").unwrap_throw().unwrap_throw().unchecked_into::<CanvasRenderingContext2d>();
                    box_canvas_ctx.clear_rect(0.0, 0.0, 640.0, 480.0);
                }
                
                // Stop each MediaStreamTrack
                let media_devices = match window().navigator().media_devices() {
                    Ok(devices) => devices,
                    Err(e) => {
                        console::error_1(&"Error getting media devices".into());
                        console::error_1(&e);
                        return false;
                    },
                };

                let mut constraints = MediaStreamConstraints::new();
                constraints.video(&JsValue::TRUE);
                
                let promise = match media_devices.get_user_media_with_constraints(&constraints) {
                    Ok(promise) => promise,
                    Err(e) => {
                        console::error_1(&"Error geting user media".into());
                        console::error_1(&e);
                        return false;
                    }
                };

                // Set user media as camera video source
                spawn_local(async move {
                    match JsFuture::from(promise).await {
                        Ok(user_media) => {
                            let media_stream = user_media.unchecked_into::<MediaStream>();
                            let media_stream_tracks = media_stream.get_tracks();
                            for track in media_stream_tracks {
                                track.unchecked_into::<MediaStreamTrack>().stop();
                            }
                        }
                        Err(e) => {
                            console::error_1(&"Error resolving enumarate devices promise".into());
                            console::error_1(&e);
                        }
                    }
                });

                return true;
            }
            Msg::RenderCameras(cameras_amount) => {
                self.cameras = cameras_amount;
                ctx.link().send_message(Msg::AddSources);
                return true;
            },
            Msg::AddSources => {
                // Get list of devices from a promise
                // Get mediaDevices -> Enumarate devices -> Resolve promise

                // Prepare user media promise
                let mut constraints = MediaStreamConstraints::new();
                constraints.video(&JsValue::TRUE);
                
                let media_devices = match window().navigator().media_devices() {
                    Ok(devices) => devices,
                    Err(e) => {
                        console::error_1(&"Error getting media devices".into());
                        console::error_1(&e);
                        return false;
                    },
                };

                let media_devices_promise = match media_devices.get_user_media_with_constraints(&constraints) {
                    Ok(promise) => promise,
                    Err(e) => {
                        console::error_1(&"Error geting user media promise".into());
                        console::error_1(&e);
                        return false;
                    }
                };

                spawn_local(async move {
                    JsFuture::from(media_devices_promise).await.unwrap_throw();
                    
                    
                    let enumarated_devices_promise = match media_devices.enumerate_devices() {
                        Ok(promise) => promise,
                        Err(e) => {
                            console::error_1(&"Error geting enumarated devices promise".into());
                            console::error_1(&e);
                            return ();
                        }
                    };

                    let select_elements_list = document().query_selector_all("select#sources").unwrap_throw();
                    
                    // Resolve enumarated devices promise
                    match JsFuture::from(enumarated_devices_promise).await {
                        Ok(devices_array) => {
                            let select_elements_array = Array::from_iter(
                                (0..select_elements_list.length())
                                    .map(|i| select_elements_list.item(i).unwrap())
                            );

                            // Add each camera to the sources list and append them into a document fragment 
                            for element in select_elements_array.iter() {
                                let option_list = element.unchecked_into::<HtmlOptionElement>();
                                option_list.set_inner_html("<option>Sin fuente</option>");
                                for device in Array::from(&devices_array).iter() {
                                    let device_info = device.unchecked_into::<MediaDeviceInfo>();
                                    if device_info.kind() == MediaDeviceKind::Videoinput {
                                        // Create option element and add it into the sources list
                                        let option = document().create_element("option").unwrap().unchecked_into::<HtmlOptionElement>();
                                        option.set_value(device_info.device_id().as_str());
                                        option.set_text_content(Some(device_info.label().as_str()));
                                        option_list.append_child(&option).expect_throw("Error appending sources to option element");
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            console::error_1(&"Error resolving enumarate devices promise".into());
                            console::error_1(&e);
                        }
                    }
                }); 
                return false;
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let is_predicting = self.prediction_interval.is_some();

        html! {
            <div class="camera-frame">
                <div class="cameras">
                {
                     for (0..self.cameras).map(|_| html! {
                        <div class="camera">
                            <div class="frame">
                                <canvas class="canvas" id="canvas"/>
                                <video class="video" id="video" autoplay={true}/>
                            </div>
                            <div class="source">
                                <label for="sources">{ "Fuente:" }</label>
                                <select name="sources" id="sources" class="select"></select>
                            </div>
                        </div>
                    })
                }
                </div>
                <div class="buttons">
                    <div class="amount">
                        <button onclick={ ctx.link().callback(|_| Msg::RenderCameras(1)) }>{ "1" }</button>
                        <button onclick={ ctx.link().callback(|_| Msg::RenderCameras(2)) }>{ "2" }</button>
                        <button onclick={ ctx.link().callback(|_| Msg::RenderCameras(4)) }>{ "4" }</button>
                        <button onclick={ ctx.link().callback(|_| Msg::RenderCameras(8)) }>{ "8" }</button>
                        <button onclick={ ctx.link().callback(|_| Msg::RenderCameras(16)) }>{ "16" }</button>
                    </div>
                    <div class="control">
                        <button disabled={is_predicting} class="start" onclick={ ctx.link().callback(|_| Msg::SetupPredictionProcess) }>
                            { "Comenzar" }
                        </button>
                        <button disabled={is_predicting == false} class="stop" onclick={ ctx.link().callback(|_| Msg::StopPredictionProcess) }>
                            { "Detener" }
                        </button>
                    </div>
                </div>
            </div>
        }
    }
    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_message(Msg::AddSources);
        }
    }
}
