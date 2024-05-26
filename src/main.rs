use rand::RngCore;
use rand::rngs::OsRng;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};

use rusqlite::{Connection, params};
use chrono::{Utc, DateTime, Duration};

use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Mutex;

const DEFAULT_CODE_DURATION: i64 = 3600;
const MAX_CODE_DURATION: i64 = 86400;
const SERVER_IP: &str = "127.0.0.1:8080";


// just randomly generate 128 bits
fn create_code() -> String {
    let mut bytes: [u8; 16] = [0u8; 16]; // 128-bit key
    OsRng.fill_bytes(&mut bytes);

    URL_SAFE.encode(&bytes)
}

// supply the db Connection, make it if it's deleted
fn create_db() -> Connection {
    let conn = Connection::open("codes.db").unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS codes (
            code TEXT PRIMARY KEY,
            expiration_time TEXT
        )",
        [],
    ).unwrap();
    conn
}

// add code to db. there's no throttling here yet, could get DOS'd
fn save_code_to_db(conn: &Connection, code: &str, expiration_time: DateTime<Utc>) {
    conn.execute(
        "INSERT INTO codes (code, expiration_time) VALUES (?1, ?2)",
        params![code, expiration_time.to_rfc3339()],
    ).unwrap();
}

fn delete_code_from_db(conn: &Connection, code: &str) {
    conn.execute("DELETE FROM codes WHERE code=?1", params![code]).unwrap();
}

// all this just to check the db for the code
fn verify_code(conn: &Connection, code: &str) -> bool {
    // get expiration time
    let expiration_time: Result<chrono::DateTime<Utc>, _> = conn.query_row(
        "SELECT expiration_time FROM codes WHERE code=?1",
        rusqlite::params![code],
        |row| {
            let expiration_time_str: String = row.get(0)?;
            Ok(chrono::DateTime::parse_from_rfc3339(&expiration_time_str).unwrap().with_timezone(&Utc))
        }
    );

    // is it in the db and is it expired
    match expiration_time {
        Ok(expiration_time) => Utc::now() < expiration_time,
        Err(_) => false,
    }
}

fn open_door() {
    std::process::Command::new("door.sh").output().expect("DOOR STUCK");
}


// server shit
// there are 3 endpoints: one-time code producer (addcode) and consumer (verifycode), as well as just "open the door".
// TODO make the db not trivial to DOS

#[derive(Deserialize)]
struct AddCodeRequest {
    duration_seconds: Option<i64>,
}

#[derive(Deserialize)]
struct VerifyCodeRequest {
    code: String,
}


async fn add_code_endpoint(data: web::Data<Mutex<Connection>>, info: web::Json<AddCodeRequest>) -> impl Responder {
    // TODO auth goes here later :^)
    let conn = data.lock().unwrap();
    let code = create_code();

    let duration = Duration::seconds(info.duration_seconds.unwrap_or(DEFAULT_CODE_DURATION).min(MAX_CODE_DURATION));
    let expiration_time = Utc::now() + duration;

    save_code_to_db(&conn, &code, expiration_time);
    HttpResponse::Ok().json(serde_json::json!({"status": "success", "code": code}))
}

async fn verify_code_endpoint(data: web::Data<Mutex<Connection>>, info: web::Json<VerifyCodeRequest>) -> impl Responder {
    let conn = data.lock().unwrap();

    if verify_code(&conn, &info.code) {
        open_door();
        delete_code_from_db(&conn, &info.code);
        HttpResponse::Ok().json(serde_json::json!({"status": "success"}))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({"status": "failed"}))
    }
}

async fn open_endpoint() -> impl Responder {
    // auth also goes here in theory
    open_door();
    HttpResponse::Ok().json(serde_json::json!({"status": "success"}))
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let conn = create_db();
    let app_data = web::Data::new(Mutex::new(conn));

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(
                web::resource("/add_code")
                    .route(web::post().to(add_code_endpoint))
            )
            .service(
                web::resource("/verify_code")
                    .route(web::post().to(verify_code_endpoint))
            )
            .service(
                web::resource("/open")
                    .route(web::post().to(open_endpoint))
            )

    })
    .bind(SERVER_IP)?
    .run()
    .await
}

