use bluest::{Adapter, AdvertisingDevice};
use chrono::Utc;
use clap::Parser;
use futures_lite::stream::StreamExt;
use serde_json;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    server: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let adapter = Adapter::default()
        .await
        .ok_or("Bluetooth adapter not found")
        .unwrap();
    adapter.wait_available().await.unwrap();
    let client = reqwest::Client::new();

    println!("Starting scan Xiaomi Band");
    let mut scan = adapter.scan(&[]).await.unwrap();
    while let Some(discovered_device) = scan.next().await {
        if let Some(json_output) = handle_device(discovered_device) {
            println!("{}", json_output);

            if args.server != None {
                let response = client
                    .post(&format!("{:?}", args.server))
                    .json(&json_output)
                    .send()
                    .await
                    .unwrap()
                    .json::<serde_json::Value>()
                    .await;
                
                match response {
                    Ok(_res) => {
                    }
                    Err(err) => {
                        eprintln!("Request Error: {:#?}", err)
                    }
                }
            }
        }
    }
}

fn handle_device(discovered_device: AdvertisingDevice) -> Option<serde_json::Value> {
    let manufacturer_data = discovered_device.adv_data.manufacturer_data?;
    if manufacturer_data.company_id != 0x0157 {
        return None;
    }
    let name = discovered_device
        .device
        .name()
        .unwrap_or(String::from("(unknown)"));
    let id = discovered_device.device.id();
    let rssi = discovered_device.rssi.unwrap_or_default(); //dBm

    let heart_rate: Option<u8> = match manufacturer_data.data[3] {
        0xFF => None,
        x => Some(x.into()),
    };
    let json_output = serde_json::json!({
        "timestamp": Utc::now().timestamp(),
        "name": name,
        "id": id.to_string(),
        "rssi": rssi,
        "heart_rate": heart_rate
    });
    Some(json_output)
}
