//! Debug probe: inspect raw FortiGate response as reqwest sees it.
//! Usage: FORTITUI_DEV=<token> cargo run --example probe -- <host> <auto|http1>
use reqwest::Client;

#[tokio::main]
async fn main() {
    let host = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "172.22.129.1".into());
    let mode = std::env::args().nth(2).unwrap_or_else(|| "auto".into());
    let token = std::env::var("FORTITUI_DEV").unwrap_or_default();
    let url = format!("https://{host}/api/v2/monitor/system/status");
    let mut b = Client::builder().danger_accept_invalid_certs(true);
    b = if mode == "http1" { b.http1_only() } else { b };
    let c = b.build().unwrap();
    let r = c.get(&url).bearer_auth(&token).send().await.unwrap();
    println!("mode={mode} status: {}", r.status());
    println!("version: {:?}", r.version());
    for (k, v) in r.headers().iter() {
        println!("hdr {k}: {v:?}");
    }
    let txt = r.text().await.unwrap();
    println!("len={} body={}", txt.len(), &txt[..txt.len().min(200)]);
}
