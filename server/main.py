from fastapi import FastAPI, APIRouter, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from io import BytesIO
from PIL import Image
import json
from ultralytics import YOLO
import time
import uvicorn

# Models
class BoundingBox(BaseModel):
    x1: float
    y1: float
    x2: float
    y2: float

class Inference(BaseModel):
    bounding_box: BoundingBox
    object: str
    probability: float

class Object:
    object_name: str

    def __init__(self, object_name):
        self.object_name = object_name

try:
    model = YOLO("data/model.pt").to("cuda")
    print("Using CUDA")
except:
    model = YOLO("data/model.pt")
    print("Using CPU")

router = APIRouter()

class ConnectionManager:
    def __init__(self):
        self.active_connections: list[WebSocket] = []

    async def connect(self, websocket: WebSocket):
        await websocket.accept()
        self.active_connections.append(websocket)

    def disconnect(self, websocket: WebSocket):
        self.active_connections.remove(websocket)

    async def send_personal_message(self, message: str, websocket: WebSocket):
        await websocket.send_text(message)

    async def broadcast(self, message: str):
        for connection in self.active_connections:
            await connection.send_text(message)

manager = ConnectionManager()

@router.websocket("/api/predict")
async def predict(websocket: WebSocket):
    await manager.connect(websocket)
    try:
        while True:
            data = await websocket.receive_bytes()
            # Make an inference
            source = Image.open(BytesIO(data))
            results = model.predict(source=source, conf=0.75)

            # Only send message if there's at least one result
            if results[0].__len__() > 0:
                # Make a list of all results
                predictions = []

                for result in results:
                    boxes = result.boxes.cpu().numpy()
                    for box in boxes:

                        # Create a response based on the prediction response model
                        coordinates = box.xyxy[0].tolist()
                        inferenced_object = box.cls[0].tolist()
                        probability = box.conf[0].tolist()
                        inferenced_object = model.names[inferenced_object].upper()
                        box = BoundingBox(x1=coordinates[0], y1=coordinates[1], x2=coordinates[2], y2=coordinates[3])
                        prediction = Inference(bounding_box=box, object=inferenced_object, probability=probability)

                        predictions.append(prediction)

                # Serialize list of results to json
                predictions_json = json.dumps([prediction.dict() for prediction in predictions])

                # Send message to client
                await websocket.send_text(predictions_json)
            else:
                await websocket.send_text("No objects detected")
    except WebSocketDisconnect:
        manager.disconnect(websocket)

app = FastAPI()

app.include_router(router)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"], # Allows all origins
    allow_credentials=True,
    allow_methods=["*"], # Allows all methods
    allow_headers=["*"], # Allows all headers
)

if __name__ == "__main__":
    uvicorn.run("main:app", host="0.0.0.0", port=8000, log_level="info")
