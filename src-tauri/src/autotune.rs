use std::time::Duration;
use tauri::{AppHandle, State};
use crate::bypass::{start_bypass_internal, stop_bypass, extract_resources, BypassState, kill_existing_winws};

async fn test_url(url: &str) -> bool {
    // Создаем клиент reqwest с таймаутом
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(4))
        .danger_accept_invalid_certs(true)
        .build();

    if let Ok(client) = client {
        let request = client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await;

        if let Ok(response) = request {
            // Если получили ответ (даже ошибку сервера типа 403 или 404), значит соединение установлено
            // и DPI не сбросил/заблокировал пакеты
            let status = response.status();
            return status.is_success() || status.is_redirection() || status == reqwest::StatusCode::FORBIDDEN;
        }
    }
    false
}

// Фоновый автоподбор рабочей стратегии обхода
#[tauri::command]
pub async fn run_autotune(
    app_handle: AppHandle,
    state: State<'_, BypassState>,
) -> Result<String, String> {
    // 1. Распаковываем файлы во временную папку
    let (temp_dir, app_data_dir) = extract_resources(&app_handle)?;

    let strategies = vec!["standard", "alt", "simple"];
    
    for strategy in strategies {
        // Останавливаем предыдущую попытку, если она была
        kill_existing_winws();
        
        // Запускаем обход с текущей стратегией
        if let Ok(pid) = start_bypass_internal(strategy, &temp_dir, &app_data_dir) {
            // Сохраняем временный PID
            {
                let mut guard = state.child_pid.lock().unwrap();
                *guard = Some(pid);
            }

            // Ждем 2.5 секунды, пока драйвер WinDivert загрузится и winws.exe инициализируется
            tokio::time::sleep(Duration::from_millis(2500)).await;

            // Проверяем доступность YouTube
            // (Тестируем как основной домен, так и HTTPS рукопожатие)
            if test_url("https://www.youtube.com").await {
                // Если сработало, возвращаем имя рабочей стратегии
                // И оставляем её запущенной!
                return Ok(strategy.to_string());
            }
        }
    }

    // Если ничего не подошло, останавливаем процессы и возвращаем ошибку
    let _ = stop_bypass(state).await;
    Err("Не удалось подобрать рабочую стратегию. Пожалуйста, проверьте подключение к сети или настройте параметры вручную.".to_string())
}
