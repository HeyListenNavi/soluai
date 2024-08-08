use actix::{Actor, StreamHandler};
use actix_web::{web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use crate::yolo::run_model;

struct PredictWebsocketActor;

impl Actor for PredictWebsocketActor {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PredictWebsocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Binary(bin)) => {
                let now = std::time::Instant::now();
                match run_model(bin.clone()) {
                    Ok(inferences) => println!("{:?}", inferences),
                    Err(e) => println!("{:?}", e)
                }
                let elapsed = now.elapsed();
                println!("Duration: {:.2?}", elapsed);
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