export function connectWebSocket() {
    // Create WebSocket connection
    const socket = new WebSocket("ws://127.0.0.1:8000/api/predict");
    
    const video_cam = document.getElementById("video");
    const boxes_canvas = document.getElementById("canvas");
    const boxes_canvas_ctx = boxes_canvas.getContext("2d");
   
    
    socket.binaryType = "arraybuffer"
    socket.onclose = () => {
        socket = null;
        console.log("Connection is closed");
    };
    socket.onerror = (e) => {console.log(e.msg);}
    
    socket.onopen = (event) => {
        console.log("Connection opened");
        // Access the webcam
        navigator.mediaDevices.getUserMedia({ video: true })
        .then((stream) => {
            video_cam.srcObject = stream;
            video_cam.play();
            
            video_cam.addEventListener('canplay', () => {
                boxes_canvas.width = video_cam.videoWidth;
                boxes_canvas.height = video_cam.videoHeight
                setInterval(() => {
                        // Capture frame
                        const frame_canvas = document.createElement("canvas");
                        frame_canvas.width = video_cam.videoWidth;
                        frame_canvas.height = video_cam.videoHeight;
                        var frame_canvas_ctx = frame_canvas.getContext("2d");
                        frame_canvas_ctx.drawImage(video_cam, 0, 0, frame_canvas.width, frame_canvas.height);
                        
                        // Send frame
                        frame_canvas.toBlob((blob) => {
                            if (blob) {
                                blob.arrayBuffer().then((array_buffer) => {
                                    console.log("Sending frame: ");
                                    console.log(array_buffer);
                                    socket.send(array_buffer);
                                }).catch((error) => {
                                    console.error("Error converting blob to arrayBuffer:", error);
                                });
                            } else {
                                console.error("Failed to capture frame as blob");
                            }
                        }, "image/jpeg", 0.75);
                    
                    }, 5000);
                }, { once: true });
            })
            .catch((err) => {
                console.error("Error accessing the webcam: " + err);
            });
        
    };
    
    socket.onmessage = (event) => {
        const inferences = JSON.parse(event.data);
        console.log(inferences);

        boxes_canvas_ctx.clearRect(0, 0, boxes_canvas.width, boxes_canvas.height);
        boxes_canvas_ctx.strokeStyle = "#00FF00";
        boxes_canvas_ctx.lineWidth = 3;
        boxes_canvas_ctx.font = "18px serif";

        inferences.forEach((box) => {
            const x1 = box.bounding_box.x1;
            const y1 = box.bounding_box.y1;
            const x2 = box.bounding_box.x2;
            const y2 = box.bounding_box.y2;
            boxes_canvas_ctx.strokeRect(x1, y1, x2 - x1, y2 - y1);
            boxes_canvas_ctx.fillStyle = "#00ff00";
            const label = box.object + ": " + box.probability;
            const width = boxes_canvas_ctx.measureText(label).width;
            boxes_canvas_ctx.fillRect(x1, y1, width + 10, 25);
            boxes_canvas_ctx.fillStyle = "#000000";
            boxes_canvas_ctx.fillText(label, x1, y1 + 18);
        });
    }
}