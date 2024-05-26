use rand::RngCore;
use rand::rngs::OsRng;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};

use rusqlite::{Connection, params};
use chrono::{Utc, DateTime};


fn create_code() -> String {
    let mut rng = OsRng;
    let mut bytes = [0u8; 16]; // 128-bit key
    rng.fill_bytes(&mut bytes);

    URL_SAFE.encode(&bytes)
}


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

fn save_code_to_db(conn: &Connection, code: &str, expiration_time: DateTime<Utc>) {
    conn.execute(
        "INSERT INTO codes (code, expiration_time) VALUES (?1, ?2)",
        params![code, expiration_time.to_rfc3339()],
    ).unwrap();
}

fn verify_code(conn: &Connection, code: &str) -> bool {
    let mut stmt = conn.prepare("SELECT expiration_time FROM codes WHERE code=?1").unwrap();
    let code_iter = stmt.query_map(params![code], |row| {
        let expiration_time: String = row.get(0)?;
        Ok(DateTime::parse_from_rfc3339(&expiration_time).unwrap())
    }).unwrap();

    for expiration_time in code_iter {
        if let Ok(expiration_time) = expiration_time {
            if Utc::now() < expiration_time {
                conn.execute("DELETE FROM codes WHERE code=?1", params![code]).unwrap();
                return true;
            }
        }
    }
    false
}


fn main() {
    let test_code = create_code();
    let test_2 = create_code();
    println!("{}", create_code());

    let db = create_db();

    let now = Utc::now() + chrono::Duration::days(1);

    save_code_to_db(&db, &test_code, now);

    println!("{}", verify_code(&db, &test_code));
    println!("{}", verify_code(&db, &test_2));

}

