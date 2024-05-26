use reqwest::Client;
use serde::Serialize;


const HAOS_IP: &str = "192.168.10.43:42069";
const VERIFY_PATH: &str = "/verify_code";

#[derive(Serialize)]
struct VerifyRequest {
    code: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let code = "ough";
    let request = VerifyRequest { code: code.to_string() };

    let res = client.post(HAOS_IP + VERIFY_PATH)
        .json(&request)
        .send()
        .await?;

    println!("{:#?}", res.json::<serde_json::Value>().await?);
    Ok(())
}
