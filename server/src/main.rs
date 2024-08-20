use actix_cors::Cors;
use actix_web::{web::{self, scope}, App, HttpServer};
mod services;
mod yolo;
use services::prediction_websocket;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = match std::env::var("PORT") {
        Ok(port) => port,
        _ => String::from("8000"),
    };

    let address = format!("0.0.0.0:{port}");
    println!("INFO - Server started on address: {}", &address);

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().starts_with(b"http//localhost")
            })
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().starts_with(b"http//127.0.0.1")
            })
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"]);
        App::new()
            .wrap(cors)
            .service(
                scope("/api")
                .route("/predict", web::get().to(prediction_websocket))
            )
    })
    .bind(address.clone())?
    .run()
    .await
}