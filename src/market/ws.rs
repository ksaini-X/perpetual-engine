use futures_util::StreamExt;
use serde_json::Value;
use tokio_tungstenite::connect_async;

pub async fn price_feed() {
    let url = "wss://stream.binance.com:9443/ws/btcusdt@trade";

    let (mut stream, _) = connect_async(url).await.unwrap();

    while let Some(msg) = stream.next().await {
        match msg {
            Ok(msg) => extract_price_feed(msg.to_string()),
            Err(e) => println!("Error: {e}"),
        }
    }
}

pub fn extract_price_feed(feed_text: String) {
    let feed_json = serde_json::from_str::<Value>(&feed_text).unwrap();
    println!("{}", feed_json["p"])
}
