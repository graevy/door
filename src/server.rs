use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Deserialize)]
struct VerifyRequest {
    code: String,
}

async fn verify_code_endpoint(data: web::Data<Mutex<Connection>>, info: web::Json<VerifyRequest>) -> impl Responder {
    let conn = data.lock().unwrap();
    if verify_code(&conn, &info.code) {
        // finally some good fucking door script
        std::process::Command::new("door.sh").output().expect("DOOR STUCK");
        HttpResponse::Ok().json(serde_json::json!({"status": "success"}))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({"status": "failed"}))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conn = create_db();
    let app_data = web::Data::new(Mutex::new(conn));

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .route("/verify_code", web::post().to(verify_code_endpoint))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
