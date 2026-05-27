// Предотвращает появление консольного окна на Windows в релизной сборке
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use reqwest::Client;
use serde::Serialize;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
#[derive(Serialize)]
struct CheckResult {
    http: String,
    tcp: String,
}

// Эта функция будет доступна во Vue
#[tauri::command]
async fn check_site_status(url: String) -> Result<CheckResult, String> {
    println!("Проверяю сайт: {}", url); // Это появится в терминале VS Code

    let clean_url = url.replace("https://", "").replace("http://", "");
    let address = format!("{}:443", clean_url);

    // TCP
    let tcp_status = match timeout(Duration::from_secs(2), TcpStream::connect(&address)).await {
        Ok(Ok(_)) => "OK".to_string(),
        _ => "Fail".to_string(),
    };

    // HTTP
    let client = Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;

    let full_url = if !url.starts_with("http") {
        format!("https://{}", url)
    } else {
        url.clone()
    };

    let http_status = match client.get(&full_url).send().await {
        Ok(res) => res.status().to_string(),
        Err(_) => "Error".to_string(),
    };

    println!(
        "Результат для {}: TCP={}, HTTP={}",
        url, tcp_status, http_status
    );

    Ok(CheckResult {
        http: http_status,
        tcp: tcp_status,
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![check_site_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
