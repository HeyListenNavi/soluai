use actix::{Actor, StreamHandler};
use actix_web::{web::Payload, get, rt, Error, HttpRequest, HttpResponse, Responder};
use actix_ws::Message;
use futures_util::StreamExt as _;
use serde_json;
use crate::yolo::run_model;

pub async fn prediction_websocket(req: HttpRequest, body: Payload) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;
    let mut stream = msg_stream.max_frame_size(102400);
    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(Message::Binary(bin)) => {
                    let now = std::time::Instant::now();
                    println!("Data recieved");
                    match run_model(bin.clone()) {
                        Ok(inferences) => {
                            println!("{:?}", &inferences);
                            let response = serde_json::to_string(&inferences).unwrap();
                            session.text(response.as_str()).await.unwrap_or_else(|e| {
                                println!("Error: {:?}", e);
                            });
                        },
                        Err(e) => println!("{:?}", e)
                    }
                    let elapsed = now.elapsed();
                    println!("Duration: {:.2?}", elapsed);
                }
                Err(e) => {
                    println!("{:?}", e);
                    break;
                }
                _ => {
                    println!("Unrecognized message type");
                    break;
                }
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}