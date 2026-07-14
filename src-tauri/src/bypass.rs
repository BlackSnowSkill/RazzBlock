use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use sysinfo::System;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

// Встраиваем бинарные файлы и библиотеки
const WINWS_EXE: &[u8] = include_bytes!("../../resources/bin/winws.exe");
const CYGWIN_DLL: &[u8] = include_bytes!("../../resources/bin/cygwin1.dll");
const WINDIVERT_DLL: &[u8] = include_bytes!("../../resources/bin/WinDivert.dll");
const WINDIVERT_SYS: &[u8] = include_bytes!("../../resources/bin/WinDivert64.sys");
const BIN_QUIC_1: &[u8] = include_bytes!("../../resources/bin/quic_initial_dbankcloud_ru.bin");
const BIN_QUIC_2: &[u8] = include_bytes!("../../resources/bin/quic_initial_www_google_com.bin");
const BIN_STUN: &[u8] = include_bytes!("../../resources/bin/stun.bin");
const BIN_TLS_1: &[u8] = include_bytes!("../../resources/bin/tls_clienthello_4pda_to.bin");
const BIN_TLS_2: &[u8] = include_bytes!("../../resources/bin/tls_clienthello_max_ru.bin");
const BIN_TLS_3: &[u8] = include_bytes!("../../resources/bin/tls_clienthello_www_google_com.bin");

// Встраиваем списки и шаблоны
const LIST_IPSET_ALL: &[u8] = include_bytes!("../../resources/lists/ipset-all.txt");
const LIST_IPSET_ALL_BACKUP: &[u8] = include_bytes!("../../resources/lists/ipset-all.txt.backup");
const LIST_IPSET_EXCLUDE_USER: &[u8] = include_bytes!("../../resources/lists/ipset-exclude-user.txt");
const LIST_IPSET_EXCLUDE: &[u8] = include_bytes!("../../resources/lists/ipset-exclude.txt");
const LIST_EXCLUDE_USER: &[u8] = include_bytes!("../../resources/lists/list-exclude-user.txt");
const LIST_EXCLUDE: &[u8] = include_bytes!("../../resources/lists/list-exclude.txt");
const LIST_GENERAL_USER: &[u8] = include_bytes!("../../resources/lists/list-general-user.txt");
const LIST_GENERAL: &[u8] = include_bytes!("../../resources/lists/list-general.txt");
const LIST_GOOGLE: &[u8] = include_bytes!("../../resources/lists/list-google.txt");

pub struct BypassState {
    pub child_pid: Mutex<Option<u32>>,
}

// Поиск и принудительное завершение всех процессов winws.exe
pub fn kill_existing_winws() {
    let s = System::new_all();
    for (_pid, process) in s.processes() {
        if process.name().to_lowercase() == "winws.exe" {
            let _ = process.kill();
        }
    }
}

// Рекурсивное копирование директории
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

// Очистка временных бинарных файлов во временной папке
pub fn clean_temp_binaries() {
    let temp_dir = std::env::temp_dir().join("RazzBlock_Bin");
    if temp_dir.exists() {
        // Убиваем winws перед удалением, чтобы файлы не были заблокированы драйвером
        kill_existing_winws();
        // Даем немного времени на освобождение ресурсов
        std::thread::sleep(std::time::Duration::from_millis(300));
        let _ = fs::remove_dir_all(temp_dir);
    }
}

// Извлечение бинарных файлов и списков из ресурсов Tauri
pub fn extract_resources(_app_handle: &AppHandle) -> Result<(PathBuf, PathBuf), String> {
    let temp_dir = std::env::temp_dir().join("RazzBlock_Bin");
    let app_data_dir = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("RazzBlock");

    let temp_bin_dir = temp_dir.join("bin");
    let temp_lists_dir = temp_dir.join("lists");

    // Создаем целевые папки
    fs::create_dir_all(&temp_bin_dir).map_err(|e| format!("Failed to create temp bin dir: {}", e))?;
    fs::create_dir_all(&temp_lists_dir).map_err(|e| format!("Failed to create temp lists dir: {}", e))?;
    fs::create_dir_all(&app_data_dir).map_err(|e| format!("Failed to create app data dir: {}", e))?;

    // Записываем встроенные бинарные файлы
    fs::write(temp_bin_dir.join("winws.exe"), WINWS_EXE).map_err(|e| format!("Failed to write winws.exe: {}", e))?;
    fs::write(temp_bin_dir.join("cygwin1.dll"), CYGWIN_DLL).map_err(|e| format!("Failed to write cygwin1.dll: {}", e))?;
    fs::write(temp_bin_dir.join("WinDivert.dll"), WINDIVERT_DLL).map_err(|e| format!("Failed to write WinDivert.dll: {}", e))?;
    fs::write(temp_bin_dir.join("WinDivert64.sys"), WINDIVERT_SYS).map_err(|e| format!("Failed to write WinDivert64.sys: {}", e))?;
    fs::write(temp_bin_dir.join("quic_initial_dbankcloud_ru.bin"), BIN_QUIC_1).map_err(|e| format!("Failed to write quic_initial_dbankcloud_ru.bin: {}", e))?;
    fs::write(temp_bin_dir.join("quic_initial_www_google_com.bin"), BIN_QUIC_2).map_err(|e| format!("Failed to write quic_initial_www_google_com.bin: {}", e))?;
    fs::write(temp_bin_dir.join("stun.bin"), BIN_STUN).map_err(|e| format!("Failed to write stun.bin: {}", e))?;
    fs::write(temp_bin_dir.join("tls_clienthello_4pda_to.bin"), BIN_TLS_1).map_err(|e| format!("Failed to write tls_clienthello_4pda_to.bin: {}", e))?;
    fs::write(temp_bin_dir.join("tls_clienthello_max_ru.bin"), BIN_TLS_2).map_err(|e| format!("Failed to write tls_clienthello_max_ru.bin: {}", e))?;
    fs::write(temp_bin_dir.join("tls_clienthello_www_google_com.bin"), BIN_TLS_3).map_err(|e| format!("Failed to write tls_clienthello_www_google_com.bin: {}", e))?;

    // Записываем встроенные списки (шаблоны)
    fs::write(temp_lists_dir.join("ipset-all.txt"), LIST_IPSET_ALL).map_err(|e| format!("Failed to write ipset-all.txt: {}", e))?;
    fs::write(temp_lists_dir.join("ipset-all.txt.backup"), LIST_IPSET_ALL_BACKUP).map_err(|e| format!("Failed to write ipset-all.txt.backup: {}", e))?;
    fs::write(temp_lists_dir.join("ipset-exclude-user.txt"), LIST_IPSET_EXCLUDE_USER).map_err(|e| format!("Failed to write ipset-exclude-user.txt: {}", e))?;
    fs::write(temp_lists_dir.join("ipset-exclude.txt"), LIST_IPSET_EXCLUDE).map_err(|e| format!("Failed to write ipset-exclude.txt: {}", e))?;
    fs::write(temp_lists_dir.join("list-exclude-user.txt"), LIST_EXCLUDE_USER).map_err(|e| format!("Failed to write list-exclude-user.txt: {}", e))?;
    fs::write(temp_lists_dir.join("list-exclude.txt"), LIST_EXCLUDE).map_err(|e| format!("Failed to write list-exclude.txt: {}", e))?;
    fs::write(temp_lists_dir.join("list-general-user.txt"), LIST_GENERAL_USER).map_err(|e| format!("Failed to write list-general-user.txt: {}", e))?;
    fs::write(temp_lists_dir.join("list-general.txt"), LIST_GENERAL).map_err(|e| format!("Failed to write list-general.txt: {}", e))?;
    fs::write(temp_lists_dir.join("list-google.txt"), LIST_GOOGLE).map_err(|e| format!("Failed to write list-google.txt: {}", e))?;

    // Инициализируем пользовательские списки в %APPDATA%
    let user_lists = vec![
        ("list-general-user.txt", LIST_GENERAL_USER),
        ("list-exclude-user.txt", LIST_EXCLUDE_USER),
        ("ipset-exclude-user.txt", LIST_IPSET_EXCLUDE_USER),
        ("ipset-all.txt", LIST_IPSET_ALL),
    ];

    for (list_name, bytes) in user_lists {
        let user_file_path = app_data_dir.join(list_name);
        if !user_file_path.exists() {
            fs::write(&user_file_path, bytes)
                .map_err(|e| format!("Failed to write user list {}: {}", list_name, e))?;
        }
    }

    Ok((temp_dir, app_data_dir))
}


// Построение аргументов для winws.exe в зависимости от стратегии
fn get_strategy_args(
    strategy: &str,
    bin_dir: &Path,
    lists_dir: &Path,
    app_data_dir: &Path,
) -> Vec<String> {
    // Вспомогательные пути к шаблонам
    let q_google = bin_dir.join("quic_initial_www_google_com.bin");
    let q_dbank = bin_dir.join("quic_initial_dbankcloud_ru.bin");
    let t_google = bin_dir.join("tls_clienthello_www_google_com.bin");
    let t_4pda = bin_dir.join("tls_clienthello_4pda_to.bin");
    let t_max = bin_dir.join("tls_clienthello_max_ru.bin");
    let stun = bin_dir.join("stun.bin");

    // Инициализация путей списков
    let l_general = lists_dir.join("list-general.txt");
    let l_general_u = app_data_dir.join("list-general-user.txt");
    let l_exclude = lists_dir.join("list-exclude.txt");
    let l_exclude_u = app_data_dir.join("list-exclude-user.txt");
    let l_google = lists_dir.join("list-google.txt");
    let ipset_all = app_data_dir.join("ipset-all.txt");
    let ipset_ex = lists_dir.join("ipset-exclude.txt");
    let ipset_ex_u = app_data_dir.join("ipset-exclude-user.txt");

    // Общие базовые фильтры перехвата
    let mut args = vec![
        format!("--wf-tcp=80,443,2053,2083,2087,2096,8443,12"),
        format!("--wf-udp=443,19294-19344,50000-50100,12"),
    ];

    match strategy {
        "alt" => {
            // Стратегия ALT (Fake + FakeDSplit)
            args.extend(vec![
                format!("--filter-udp=443"),
                format!("--hostlist={}", l_general.display()),
                format!("--hostlist={}", l_general_u.display()),
                format!("--hostlist-exclude={}", l_exclude.display()),
                format!("--hostlist-exclude={}", l_exclude_u.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-fake-quic={}", q_google.display()),
                
                format!("--new"),
                format!("--filter-udp=19294-19344,50000-50100"),
                format!("--filter-l7=discord,stun"),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-fake-discord={}", q_dbank.display()),
                format!("--dpi-desync-fake-stun={}", q_dbank.display()),
                format!("--dpi-desync-repeats=6"),
                
                format!("--new"),
                format!("--filter-tcp=2053,2083,2087,2096,8443"),
                format!("--hostlist-domains=discord.media"),
                format!("--dpi-desync=fake,fakedsplit"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-fooling=ts"),
                format!("--dpi-desync-fakedsplit-pattern=0x00"),
                format!("--dpi-desync-fake-tls={}", t_google.display()),
                
                format!("--new"),
                format!("--filter-tcp=443"),
                format!("--hostlist={}", l_google.display()),
                format!("--ip-id=zero"),
                format!("--dpi-desync=fake,fakedsplit"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-fooling=ts"),
                format!("--dpi-desync-fakedsplit-pattern=0x00"),
                format!("--dpi-desync-fake-tls={}", t_google.display()),
                
                format!("--new"),
                format!("--filter-tcp=80,443"),
                format!("--hostlist={}", l_general.display()),
                format!("--hostlist={}", l_general_u.display()),
                format!("--hostlist-exclude={}", l_exclude.display()),
                format!("--hostlist-exclude={}", l_exclude_u.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=fake,fakedsplit"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-fooling=ts"),
                format!("--dpi-desync-fakedsplit-pattern=0x00"),
                format!("--dpi-desync-fake-tls={}", stun.display()),
                format!("--dpi-desync-fake-tls={}", t_google.display()),
                format!("--dpi-desync-fake-http={}", t_max.display()),
                
                format!("--new"),
                format!("--filter-udp=443"),
                format!("--ipset={}", ipset_all.display()),
                format!("--hostlist-exclude={}", l_exclude.display()),
                format!("--hostlist-exclude={}", l_exclude_u.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-fake-quic={}", q_google.display()),
                
                format!("--new"),
                format!("--filter-tcp=80,443,8443"),
                format!("--ipset={}", ipset_all.display()),
                format!("--hostlist-exclude={}", l_exclude.display()),
                format!("--hostlist-exclude={}", l_exclude_u.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=fake,fakedsplit"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-fooling=ts"),
                format!("--dpi-desync-fakedsplit-pattern=0x00"),
                format!("--dpi-desync-fake-tls={}", stun.display()),
                format!("--dpi-desync-fake-tls={}", t_google.display()),
                format!("--dpi-desync-fake-http={}", t_max.display()),
                
                format!("--new"),
                format!("--filter-tcp=12"),
                format!("--ipset={}", ipset_all.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=fake,fakedsplit"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-any-protocol=1"),
                format!("--dpi-desync-cutoff=n4"),
                format!("--dpi-desync-fooling=ts"),
                format!("--dpi-desync-fakedsplit-pattern=0x00"),
                format!("--dpi-desync-fake-tls={}", stun.display()),
                format!("--dpi-desync-fake-tls={}", t_google.display()),
                format!("--dpi-desync-fake-http={}", t_max.display()),
                
                format!("--new"),
                format!("--filter-udp=12"),
                format!("--ipset={}", ipset_all.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-repeats=12"),
                format!("--dpi-desync-any-protocol=1"),
                format!("--dpi-desync-fake-unknown-udp={}", q_dbank.display()),
                format!("--dpi-desync-cutoff=n3"),
            ]);
        }
        "simple" => {
            // Стратегия SIMPLE FAKE
            args.extend(vec![
                format!("--filter-udp=443"),
                format!("--hostlist={}", l_general.display()),
                format!("--hostlist={}", l_general_u.display()),
                format!("--hostlist-exclude={}", l_exclude.display()),
                format!("--hostlist-exclude={}", l_exclude_u.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-fake-quic={}", q_google.display()),
                
                format!("--new"),
                format!("--filter-udp=19294-19344,50000-50100"),
                format!("--filter-l7=discord,stun"),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-fake-discord={}", q_dbank.display()),
                format!("--dpi-desync-fake-stun={}", q_dbank.display()),
                format!("--dpi-desync-repeats=6"),
                
                format!("--new"),
                format!("--filter-tcp=2053,2083,2087,2096,8443"),
                format!("--hostlist-domains=discord.media"),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-fooling=ts"),
                format!("--dpi-desync-fake-tls={}", t_google.display()),
                
                format!("--new"),
                format!("--filter-tcp=443"),
                format!("--hostlist={}", l_google.display()),
                format!("--ip-id=zero"),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-fooling=ts"),
                format!("--dpi-desync-fake-tls={}", t_google.display()),
                
                format!("--new"),
                format!("--filter-tcp=80,443"),
                format!("--hostlist={}", l_general.display()),
                format!("--hostlist={}", l_general_u.display()),
                format!("--hostlist-exclude={}", l_exclude.display()),
                format!("--hostlist-exclude={}", l_exclude_u.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-fooling=ts"),
                format!("--dpi-desync-fake-tls={}", stun.display()),
                format!("--dpi-desync-fake-tls={}", t_google.display()),
                format!("--dpi-desync-fake-http={}", t_max.display()),
                
                format!("--new"),
                format!("--filter-udp=443"),
                format!("--ipset={}", ipset_all.display()),
                format!("--hostlist-exclude={}", l_exclude.display()),
                format!("--hostlist-exclude={}", l_exclude_u.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-fake-quic={}", q_google.display()),
                
                format!("--new"),
                format!("--filter-tcp=80,443,8443"),
                format!("--ipset={}", ipset_all.display()),
                format!("--hostlist-exclude={}", l_exclude.display()),
                format!("--hostlist-exclude={}", l_exclude_u.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-fooling=ts"),
                format!("--dpi-desync-fake-tls={}", stun.display()),
                format!("--dpi-desync-fake-tls={}", t_google.display()),
                format!("--dpi-desync-fake-http={}", t_max.display()),
                
                format!("--new"),
                format!("--filter-tcp=12"),
                format!("--ipset={}", ipset_all.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-any-protocol=1"),
                format!("--dpi-desync-cutoff=n4"),
                format!("--dpi-desync-fooling=ts"),
                format!("--dpi-desync-fake-tls={}", stun.display()),
                format!("--dpi-desync-fake-tls={}", t_google.display()),
                format!("--dpi-desync-fake-http={}", t_max.display()),
                
                format!("--new"),
                format!("--filter-udp=12"),
                format!("--ipset={}", ipset_all.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-repeats=12"),
                format!("--dpi-desync-any-protocol=1"),
                format!("--dpi-desync-fake-unknown-udp={}", q_dbank.display()),
                format!("--dpi-desync-cutoff=n3"),
            ]);
        }
        _ => {
            // Стандартная стратегия (по умолчанию - multisplit)
            args.extend(vec![
                format!("--filter-udp=443"),
                format!("--hostlist={}", l_general.display()),
                format!("--hostlist={}", l_general_u.display()),
                format!("--hostlist-exclude={}", l_exclude.display()),
                format!("--hostlist-exclude={}", l_exclude_u.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-fake-quic={}", q_google.display()),
                
                format!("--new"),
                format!("--filter-udp=19294-19344,50000-50100"),
                format!("--filter-l7=discord,stun"),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-fake-discord={}", q_dbank.display()),
                format!("--dpi-desync-fake-stun={}", q_dbank.display()),
                format!("--dpi-desync-repeats=6"),
                
                format!("--new"),
                format!("--filter-tcp=2053,2083,2087,2096,8443"),
                format!("--hostlist-domains=discord.media"),
                format!("--dpi-desync=multisplit"),
                format!("--dpi-desync-split-seqovl=681"),
                format!("--dpi-desync-split-pos=1"),
                format!("--dpi-desync-split-seqovl-pattern={}", t_google.display()),
                
                format!("--new"),
                format!("--filter-tcp=443"),
                format!("--hostlist={}", l_google.display()),
                format!("--ip-id=zero"),
                format!("--dpi-desync=multisplit"),
                format!("--dpi-desync-split-seqovl=681"),
                format!("--dpi-desync-split-pos=1"),
                format!("--dpi-desync-split-seqovl-pattern={}", t_google.display()),
                
                format!("--new"),
                format!("--filter-tcp=80,443"),
                format!("--hostlist={}", l_general.display()),
                format!("--hostlist={}", l_general_u.display()),
                format!("--hostlist-exclude={}", l_exclude.display()),
                format!("--hostlist-exclude={}", l_exclude_u.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=multisplit"),
                format!("--dpi-desync-split-seqovl=568"),
                format!("--dpi-desync-split-pos=1"),
                format!("--dpi-desync-split-seqovl-pattern={}", t_4pda.display()),
                
                format!("--new"),
                format!("--filter-udp=443"),
                format!("--ipset={}", ipset_all.display()),
                format!("--hostlist-exclude={}", l_exclude.display()),
                format!("--hostlist-exclude={}", l_exclude_u.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-repeats=6"),
                format!("--dpi-desync-fake-quic={}", q_google.display()),
                
                format!("--new"),
                format!("--filter-tcp=80,443,8443"),
                format!("--ipset={}", ipset_all.display()),
                format!("--hostlist-exclude={}", l_exclude.display()),
                format!("--hostlist-exclude={}", l_exclude_u.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=multisplit"),
                format!("--dpi-desync-split-seqovl=568"),
                format!("--dpi-desync-split-pos=1"),
                format!("--dpi-desync-split-seqovl-pattern={}", t_4pda.display()),
                
                format!("--new"),
                format!("--filter-tcp=12"),
                format!("--ipset={}", ipset_all.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=multisplit"),
                format!("--dpi-desync-any-protocol=1"),
                format!("--dpi-desync-cutoff=n3"),
                format!("--dpi-desync-split-seqovl=568"),
                format!("--dpi-desync-split-pos=1"),
                format!("--dpi-desync-split-seqovl-pattern={}", t_4pda.display()),
                
                format!("--new"),
                format!("--filter-udp=12"),
                format!("--ipset={}", ipset_all.display()),
                format!("--ipset-exclude={}", ipset_ex.display()),
                format!("--ipset-exclude={}", ipset_ex_u.display()),
                format!("--dpi-desync=fake"),
                format!("--dpi-desync-repeats=12"),
                format!("--dpi-desync-any-protocol=1"),
                format!("--dpi-desync-fake-unknown-udp={}", q_dbank.display()),
                format!("--dpi-desync-cutoff=n2"),
            ]);
        }
    }

    args
}

// Запуск процесса обхода
pub fn start_bypass_internal(
    strategy: &str,
    temp_dir: &Path,
    app_data_dir: &Path,
) -> Result<u32, String> {
    // Сначала убеждаемся, что старые процессы завершены
    kill_existing_winws();

    let winws_path = temp_dir.join("bin").join("winws.exe");
    if !winws_path.exists() {
        return Err("winws.exe not found in temporary directory".to_string());
    }

    let bin_dir = temp_dir.join("bin");
    let lists_dir = temp_dir.join("lists");

    let args = get_strategy_args(strategy, &bin_dir, &lists_dir, app_data_dir);

    // Запускаем процесс winws.exe скрытно
    let mut cmd = Command::new(&winws_path);
    cmd.args(&args);
    cmd.current_dir(bin_dir);

    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW: запускать скрыто без консольного окна

    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn winws.exe: {}", e))?;

    Ok(child.id())
}

// --- Tauri Commands ---

#[tauri::command]
pub async fn start_bypass(
    app_handle: AppHandle,
    state: State<'_, BypassState>,
    strategy: String,
) -> Result<(), String> {
    // 1. Распаковываем файлы
    let (temp_dir, app_data_dir) = extract_resources(&app_handle)?;

    // 2. Запускаем обход
    let pid = start_bypass_internal(&strategy, &temp_dir, &app_data_dir)?;

    // 3. Сохраняем PID
    let mut guard = state.child_pid.lock().unwrap();
    *guard = Some(pid);

    Ok(())
}

#[tauri::command]
pub async fn stop_bypass(state: State<'_, BypassState>) -> Result<(), String> {
    // 1. Завершаем процессы winws.exe
    kill_existing_winws();

    let mut guard = state.child_pid.lock().unwrap();
    *guard = None;

    // 2. Удаляем временные файлы
    clean_temp_binaries();

    Ok(())
}

#[tauri::command]
pub async fn check_bypass_status(state: State<'_, BypassState>) -> Result<bool, String> {
    let guard = state.child_pid.lock().unwrap();
    if guard.is_none() {
        return Ok(false);
    }

    // Проверяем с помощью sysinfo, работает ли процесс с этим PID
    let pid = guard.unwrap();
    let s = System::new_all();
    let is_running = s.process(sysinfo::Pid::from(pid as usize)).is_some();

    Ok(is_running)
}

#[tauri::command]
pub async fn get_user_list(app_handle: AppHandle, name: String) -> Result<String, String> {
    let app_data_dir = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("RazzBlock");

    let file_path = app_data_dir.join(&name);
    if !file_path.exists() {
        // Гарантируем распаковку ресурсов, чтобы создать директорию и файлы в AppData
        let _ = extract_resources(&app_handle)?;
    }

    fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
pub async fn save_user_list(app_handle: AppHandle, name: String, content: String) -> Result<(), String> {
    let app_data_dir = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("RazzBlock");

    let file_path = app_data_dir.join(&name);
    if !file_path.exists() {
        let _ = extract_resources(&app_handle)?;
    }
    
    // Пишем контент в UTF-8
    fs::write(&file_path, content).map_err(|e| format!("Failed to write file: {}", e))
}
