//! Debug probe: inspect the raw FortiGate response as reqwest sees it.
//! Run: FORTIGATE_PROD=<token> cargo run --example probe -- <host>

use reqwest::Client;

#[tokio::main]
async fn main() {
    let host = std::env::args().nth(1).unwrap_or_else(|| "172.22.128.255".into());
    let token = std::env::var("FORTIGATE_PROD").unwrap_or_default();
    let url = format!("https://{host}/api/v2/monitor/system/status");
    let c = Client::builder()
        .danger_accept_invalid_certs(true)
        .http1_only()
        .build()
        .unwrap();
    let r = c.get(&url).bearer_auth(&token).send().await.unwrap();
    println!("status: {}", r.status());
    println!("version: {:?}", r.version());
    for (k, v) in r.headers().iter() {
        println!("hdr {k}: {v:?}");
    }
    let txt = r.text().await.unwrap();
    println!("len={} body={}", txt.len(), &txt[..txt.len().min(300)]);
}
