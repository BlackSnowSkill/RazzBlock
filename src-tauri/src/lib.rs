mod bypass;
mod autostart;
mod autotune;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State,
};

#[tauri::command]
fn hide_window(window: tauri::WebviewWindow) {
    let _ = window.hide();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // При старте очищаем старые временные файлы и убиваем зависшие процессы winws
    bypass::clean_temp_binaries();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(bypass::BypassState {
            child_pid: std::sync::Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            bypass::start_bypass,
            bypass::stop_bypass,
            bypass::check_bypass_status,
            bypass::get_user_list,
            bypass::save_user_list,
            autostart::set_autostart,
            autostart::get_autostart_status,
            autotune::run_autotune,
            hide_window,
        ])
        .setup(|app| {
            // Создаем пункты меню системного трея
            let show_i = MenuItemBuilder::new("Открыть RazzBlock")
                .id("show")
                .build(app)?;
            let quit_i = MenuItemBuilder::new("Выйти")
                .id("quit")
                .build(app)?;

            let menu = MenuBuilder::new(app)
                .items(&[&show_i, &quit_i])
                .build()?;

            // Получаем иконку по умолчанию
            let icon = app.default_window_icon().cloned();

            // Создаем иконку в системном трее
            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false); // Меню открывается только по правому клику

            if let Some(i) = icon {
                tray_builder = tray_builder.icon(i);
            }

            let _tray = tray_builder
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        // При клике левой кнопкой мыши открываем главное окно
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .on_menu_event(|tray_app, event| {
                    match event.id().as_ref() {
                        "quit" => {
                            // Останавливаем все процессы и очищаем ресурсы перед выходом
                            bypass::kill_existing_winws();
                            bypass::clean_temp_binaries();
                            tray_app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = tray_app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Читаем аргументы командной строки (для автозапуска)
            let args: Vec<String> = std::env::args().collect();
            let is_minimized = args.iter().any(|arg| arg == "--minimized");

            // Ищем главное окно
            if let Some(window) = app.get_webview_window("main") {
                if !is_minimized {
                    let _ = window.show();
                    let _ = window.set_focus();
                }

                // Перехватываем событие закрытия главного окна
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
