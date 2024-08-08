use actix::{Actor, StreamHandler};
use actix_web::{web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

struct PredictWebsocketActor;

impl Actor for PredictWebsocketActor {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PredictWebsocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Binary(bin)) => {
                std::fs::write("received_image.png", bin).unwrap();
                println!("Received binary data");
            }
            Ok(ws::Message::Close(reason)) => {
                println!("Connection closed: {:?}", reason);
                ctx.close(reason);
            }
            _ => (),
        }
    }
}

pub async fn index(req: HttpRequest, stream: Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(PredictWebsocketActor {}, &req, stream);
    println!("{:?}", resp);
    resp
}