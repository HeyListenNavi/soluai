export class WebSocketManager {
    constructor(url) {
        this.url = url;
        this.socket = null;
    }

    connect() {
        if (!this.socket) {
            this.socket = new WebSocket(this.url);
            this.socket.binaryType = "arraybuffer";

            this.socket.onopen = () => {
                console.log("Connection opened");
            };

            this.socket.onclose = () => {
                console.log("Connection is closed");
                this.socket = null;
            };

            this.socket.onerror = (e) => {
                console.error("WebSocket error:", e);
            };
        }
    }

    send_frame_to_server(video_camera, box_canvas) {
        // Capture frame
        const frame_canvas = document.createElement("canvas");
        frame_canvas.width = video_camera.videoWidth;
        frame_canvas.height = video_camera.videoHeight;
        const frame_canvas_ctx = frame_canvas.getContext("2d");
        frame_canvas_ctx.drawImage(video_camera, 0, 0, frame_canvas.width, frame_canvas.height);

        // Convert frame to an ArrayBuffer and send it to the server
        frame_canvas.toBlob((blob) => {
            if (blob) {
                blob.arrayBuffer().then((array_buffer) => {
                    console.log("Sending frame:", array_buffer);
                    // Send array to server
                    this.socket.send(array_buffer);

                    // Clear canvas
                    const boxes_canvas_ctx = box_canvas.getContext("2d");
                    boxes_canvas_ctx.clearRect(0, 0, box_canvas.width, box_canvas.height);

                    this.socket.addEventListener("message", (event) => {
                        const inferences = JSON.parse(event.data);
                        console.log(inferences);
                        this.draw_boxes(inferences, box_canvas);
                    }, { once: true });
                }).catch((error) => {
                    console.error("Error converting blob to arrayBuffer:", error);
                });
            } else {
                console.error("Failed to capture frame as blob");
            }
        }, "image/jpeg", 0.65);
    }

    draw_boxes(inferences, box_canvas) {
        const boxes_canvas_ctx = box_canvas.getContext("2d");

        // Setup style
        boxes_canvas_ctx.lineWidth = 3;
        boxes_canvas_ctx.font = "bold 10pt Noto Sans";
        boxes_canvas_ctx.strokeStyle = "#ff0000";
        boxes_canvas_ctx.shadowBlur = 10;
        boxes_canvas_ctx.shadowColor = "#ff0000";
        boxes_canvas_ctx.shadowOffsetX = 2;
        boxes_canvas_ctx.shadowOffsetY = 2;

        inferences.forEach((box) => {
            // Draw box
            const x1 = box.bounding_box.x1;
            const y1 = box.bounding_box.y1;
            const x2 = box.bounding_box.x2;
            const y2 = box.bounding_box.y2;
            boxes_canvas_ctx.strokeRect(x1, y1, x2 - x1, y2 - y1, [5]);

            // Draw label
            const probability = (box.probability * 100).toFixed(1);
            const label = box.object + ": " + probability + "%";

            boxes_canvas_ctx.fillStyle = "#ff0000";
            boxes_canvas_ctx.fillText(label, x1, y1 - 8);
        });
    }

    translate_label(label) {
        const labelMap = {
            "handgun": "Pistola",
            "knife": "Cuchillo",
            "long weapon": "Arma larga"
        };

        return labelMap[label] || "Objeto desconocido"; // Fallback in case the weapon doesn't match
    }
}
