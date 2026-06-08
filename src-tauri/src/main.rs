#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use sqlx::PgPool;
use reqwest::Client;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{interval, timeout};
use tauri_plugin_notification::NotificationExt;


#[derive(Serialize)]
struct CheckResult {
    http: String,
    tcp: String,
}


#[derive(Debug, Serialize)]
struct Site {
    id: i32,
    url: String,
}


type StatusMap = Arc<Mutex<HashMap<i32, bool>>>;

#[tauri::command]
async fn get_all_sites(
    pool: tauri::State<'_, PgPool>,
) -> Result<Vec<Site>, String> {
    let sites = sqlx::query_as!(Site, "SELECT id, url FROM sites WHERE is_active = true ORDER BY id ASC")
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())?;
        
    Ok(sites)
}


#[tauri::command]
async fn add_site_to_db(
    url: String,
    pool: tauri::State<'_, PgPool>,
) -> Result<i32, String> {
    let clean_url = url.replace("https://", "").replace("http://", "").trim().to_lowercase();


    let row = sqlx::query!(
        r#"
        INSERT INTO sites (url, is_active) 
        VALUES ($1, true) 
        ON CONFLICT (url) 
        DO UPDATE SET is_active = true 
        RETURNING id
        "#,
        clean_url
    )
    .fetch_one(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.id)
}


#[tauri::command]
async fn delete_site(
    id: i32,
    pool: tauri::State<'_, PgPool>,
) -> Result<(), String> {
    let result = sqlx::query!("DELETE FROM sites WHERE id = $1", id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err(format!("Сайт с id={} не найден", id));
    }

    Ok(())
}


#[tauri::command]
async fn check_site_status(
    url: String,
    client: tauri::State<'_, Client>,
) -> Result<CheckResult, String> {
    println!("Ручная проверка сайта: {}", url);

    let clean_url = url.replace("https://", "").replace("http://", "");
    let address = format!("{}:443", clean_url);


    let tcp_status = match timeout(Duration::from_secs(2), TcpStream::connect(&address)).await {
        Ok(Ok(_)) => "OK".to_string(),
        _ => "Fail".to_string(),
    };

    let full_url = if !url.starts_with("http") {
        format!("https://{}", url)
    } else {
        url.clone()
    };


    let http_status = match client.get(&full_url).send().await {
        Ok(res) => res.status().to_string(),
        Err(_) => "Error".to_string(),
    };

    Ok(CheckResult {
        http: http_status,
        tcp: tcp_status,
    })
}

#[derive(Serialize)]
struct TelemetryPoint {
    http_time: i32,
    tcp_time: i32,
    tcp_ok: bool,
}


#[tauri::command]
async fn get_site_history(
    site_id: i32,
    pool: tauri::State<'_, PgPool>,
) -> Result<Vec<TelemetryPoint>, String> {
    let logs = sqlx::query!(
        r#"
        SELECT http_response_time_ms, tcp_response_time_ms, is_tcp_available
        FROM site_monitoring_logs
        WHERE site_id = $1
        ORDER BY checked_at DESC
        LIMIT 30
        "#,
        site_id
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut history: Vec<TelemetryPoint> = logs
        .into_iter()
        .map(|row| TelemetryPoint {
            http_time: row.http_response_time_ms.unwrap_or(0),
            tcp_time: row.tcp_response_time_ms.unwrap_or(0),
            tcp_ok: row.is_tcp_available,
        })
        .collect();

    history.reverse(); 

    Ok(history)
}

#[tokio::main]
async fn main() {
    let db_url = "postgres://monitor_user:monitor_password@localhost:5432/site_monitoring";
    
    let pool = PgPool::connect(db_url)
        .await
        .expect("Критическая ошибка подключения к БД при старте");
    println!("Успешное подключение к PostgreSQL!");


    let http_client = Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .expect("Ошибка создания HTTP-клиента");

    let pool_clone = pool.clone();
    let client_clone = http_client.clone();


    let status_map: StatusMap = Arc::new(Mutex::new(HashMap::new()));

    tauri::Builder::default()
        .manage(pool)
        .manage(http_client)
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                let mut ticker = interval(Duration::from_secs(60));

                loop {
                    ticker.tick().await;
                    println!("Запуск автоматической фоновой проверки сайтов");


                    let sites = match sqlx::query_as!(Site, "SELECT id, url FROM sites WHERE is_active = true")
                        .fetch_all(&pool_clone)
                        .await 
                    {
                        Ok(data) => data,
                        Err(e) => {
                            eprintln!("Ошибка получения сайтов из БД: {}", e);
                            continue;
                        }
                    };

                    for site in sites {
                        let pool_worker_clone = pool_clone.clone();
                        let client_worker_clone = client_clone.clone();
                        let status_map_clone = status_map.clone();
                        let app_handle_clone = app_handle.clone();

                        tokio::spawn(async move {
                            let clean_url = site.url.replace("https://", "").replace("http://", "");
                            let tcp_url = format!("{}:443", clean_url);
                            let http_url = format!("https://{}", clean_url);

                            let http_start = Instant::now();
                            let mut is_http_ok = false;
                            let mut http_status_code = None;
                            
                            if let Ok(res) = client_worker_clone.get(&http_url).send().await {
                                is_http_ok = res.status().is_success() || res.status().is_redirection();
                                http_status_code = Some(res.status().as_u16() as i32);
                            }
                            let http_time = if is_http_ok { http_start.elapsed().as_millis() as i32 } else { 0 };

                            let tcp_start = Instant::now();
                            let mut is_tcp_ok = false;
                            
                            if let Ok(Ok(_stream)) = timeout(Duration::from_secs(3), TcpStream::connect(&tcp_url)).await {
                                is_tcp_ok = true;
                            }
                            let tcp_time = if is_tcp_ok { tcp_start.elapsed().as_millis() as i32 } else { 0 };

                            let insert_result = sqlx::query!(
                                r#"
                                INSERT INTO site_monitoring_logs 
                                (site_id, is_http_available, is_tcp_available, http_response_time_ms, tcp_response_time_ms, http_status_code)
                                VALUES ($1, $2, $3, $4, $5, $6)
                                "#,
                                site.id, is_http_ok, is_tcp_ok, http_time, tcp_time, http_status_code
                            )
                            .execute(&pool_worker_clone)
                            .await;

                            if let Err(e) = insert_result {
                                eprintln!("Ошибка записи лога для {}: {}", site.url, e);
                            } else {
                                println!("Авто-лог записан для {}: HTTP={}мс, TCP={}", site.url, http_time, is_tcp_ok);
                            }


                            let is_up = is_http_ok;
                            let mut map = status_map_clone.lock().await;
                            let previous = map.insert(site.id, is_up);

                            if let Some(was_up) = previous {
                                if was_up != is_up {
                                    let (title, body) = if is_up {
                                        ("✅ Сайт снова доступен", format!("{} восстановился", site.url))
                                    } else {
                                        ("Алярм, Сайт недоступен", format!("{} перестал отвечать", site.url))
                                    };

                                    if let Err(e) = app_handle_clone
                                        .notification()
                                        .builder()
                                        .title(title)
                                        .body(&body)
                                        .show()
                                    {
                                        eprintln!("Ошибка отправки уведомления для {}: {}", site.url, e);
                                    }
                                }
                            }
                        });
                    }
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())

        .invoke_handler(tauri::generate_handler![
            check_site_status,
            get_site_history,
            get_all_sites,
            add_site_to_db,
            delete_site
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}