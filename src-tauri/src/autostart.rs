use std::path::PathBuf;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
use winreg::RegKey;

#[tauri::command]
pub async fn set_autostart(enabled: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    
    let run_key = hkcu
        .open_subkey_with_flags(path, KEY_WRITE | KEY_READ)
        .map_err(|e| format!("Failed to open registry key: {}", e))?;

    if enabled {
        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get current executable path: {}", e))?;
        
        // Добавляем флаг --minimized для запуска приложения в свернутом виде
        let cmd = format!("\"{}\" --minimized", current_exe.to_string_lossy());
        
        run_key
            .set_value("RazzBlock", &cmd)
            .map_err(|e| format!("Failed to set registry value: {}", e))?;
    } else {
        // Игнорируем ошибку, если ключа не существует
        let _ = run_key.delete_value("RazzBlock");
    }

    Ok(())
}

#[tauri::command]
pub async fn get_autostart_status() -> Result<bool, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    
    let run_key = hkcu
        .open_subkey_with_flags(path, KEY_READ)
        .map_err(|e| format!("Failed to open registry key: {}", e))?;

    let value: Result<String, _> = run_key.get_value("RazzBlock");
    Ok(value.is_ok())
}
