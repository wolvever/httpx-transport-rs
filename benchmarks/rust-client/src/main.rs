use std::time::Instant;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let iterations: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1000);
    let client = reqwest::Client::new();
    let url = "http://localhost:8000/";
    let start = Instant::now();
    for _ in 0..iterations {
        let resp = client.get(url).send().await.unwrap();
        resp.bytes().await.unwrap();
    }
    let elapsed = start.elapsed().as_secs_f64();
    println!("{elapsed}");
}
