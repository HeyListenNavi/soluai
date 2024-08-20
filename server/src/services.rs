use actix_web::{web::Payload, rt, HttpRequest, Responder};
use actix_ws::Message;
use futures_util::StreamExt as _;
use serde_json;
use crate::yolo::run_model;

pub async fn prediction_websocket(req: HttpRequest, body: Payload) -> actix_web::Result<impl Responder> {
    let (response, mut session, msg_stream) = actix_ws::handle(&req, body)?;
    let mut stream = msg_stream.max_frame_size(921600);
    
    println!("INFO - Conection made");
    
    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(Message::Binary(bin)) => {
                    let now = std::time::Instant::now();
                    println!("INFO - Data recieved");
                    match run_model(bin.clone()) {
                        Ok(inferences) => {
                            println!("INFO - Inference made\n{:?}", &inferences);
                            let response = serde_json::to_string(&inferences).unwrap();
                            session.text(response.as_str()).await.unwrap_or_else(|e| {
                                println!("ERROR - {:?}", e);
                            });
                        },
                        Err(e) => println!("{:?}", e)
                    }
                    let elapsed = now.elapsed();
                    println!("INFO - Duration, {:.2?}", elapsed);
                }
                Err(e) => {
                    println!("ERROR - Protocol error, {:?}", e);
                    break;
                }
                _ => {
                    println!("ERROR - Unrecognized message type");
                    break;
                }
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}