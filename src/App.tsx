import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type Tab = "dashboard" | "editor" | "settings";

function App() {
  const [activeTab, setActiveTab] = useState<Tab>("dashboard");
  const [isEnabled, setIsEnabled] = useState(false);
  const [statusText, setStatusText] = useState("Отключено");
  const [isTuning, setIsTuning] = useState(false);
  
  // Настройки
  const [strategy, setStrategy] = useState("standard"); // standard, alt, simple, autotune
  const [autostart, setAutostart] = useState(false);
  
  // Редактор списков
  const [selectedList, setSelectedList] = useState("list-general-user.txt");
  const [listContent, setListContent] = useState("");
  const [listSaveStatus, setListSaveStatus] = useState("");

  // Инициализация при старте
  useEffect(() => {
    async function init() {
      try {
        const running = await invoke<boolean>("check_bypass_status");
        setIsEnabled(running);
        setStatusText(running ? "Обход активен" : "Отключено");

        const auto = await invoke<boolean>("get_autostart_status");
        setAutostart(auto);
      } catch (e) {
        console.error("Initialization error:", e);
      }
    }
    init();
  }, []);

  // Загрузка списков при изменении выбора в редакторе
  useEffect(() => {
    if (activeTab === "editor") {
      loadListContent();
    }
  }, [selectedList, activeTab]);

  const loadListContent = async () => {
    try {
      setListSaveStatus("Загрузка...");
      const content = await invoke<string>("get_user_list", { name: selectedList });
      setListContent(content);
      setListSaveStatus("");
    } catch (e) {
      setListContent("");
      setListSaveStatus(`Ошибка при загрузке: ${e}`);
    }
  };

  const handleSaveList = async () => {
    try {
      setListSaveStatus("Сохранение...");
      await invoke("save_user_list", { name: selectedList, content: listContent });
      setListSaveStatus("Сохранено успешно!");
      setTimeout(() => setListSaveStatus(""), 3000);
      
      // Перезапускаем обход, если он активен, чтобы применить новые списки
      if (isEnabled && !isTuning) {
        setStatusText("Перезапуск обхода...");
        await invoke("start_bypass", { strategy });
        setStatusText("Обход активен");
      }
    } catch (e) {
      setListSaveStatus("Ошибка при сохранении");
    }
  };

  // Переключение ВКЛ/ВЫКЛ
  const handleToggleBypass = async () => {
    if (isTuning) return;

    if (isEnabled) {
      // Выключение
      try {
        setStatusText("Выключение...");
        await invoke("stop_bypass");
        setIsEnabled(false);
        setStatusText("Отключено");
      } catch (e) {
        setStatusText("Ошибка отключения");
        console.error(e);
      }
    } else {
      // Включение
      try {
        if (strategy === "autotune") {
          setIsTuning(true);
          setStatusText("Подбираем стратегию обхода...");
          try {
            const workingStrategy = await invoke<string>("run_autotune");
            setStrategy(workingStrategy);
            setIsEnabled(true);
            setIsTuning(false);
            setStatusText(`Подключено (${workingStrategy})`);
          } catch (err: any) {
            setIsTuning(false);
            setIsEnabled(false);
            setStatusText("Не удалось подобрать стратегию");
            alert(err);
          }
        } else {
          setStatusText("Запуск обхода...");
          await invoke("start_bypass", { strategy });
          setIsEnabled(true);
          setStatusText("Обход активен");
        }
      } catch (e: any) {
        setStatusText("Ошибка запуска");
        alert(e);
      }
    }
  };

  // Переключение автозапуска
  const handleToggleAutostart = async (val: boolean) => {
    try {
      await invoke("set_autostart", { enabled: val });
      setAutostart(val);
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="cozy-app">
      <header className="cozy-header" data-tauri-drag-region>
        <div className="logo-group" data-tauri-drag-region>
          <span className="app-title" data-tauri-drag-region>RazzBlock</span>
          <span className="app-version" data-tauri-drag-region>v1.0.0</span>
        </div>
        <button className="window-close-btn" onClick={() => invoke("hide_window")} title="Свернуть в трей">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </header>

      {/* Навигационные вкладки */}
      <nav className="cozy-nav">
        <button 
          className={`nav-item ${activeTab === "dashboard" ? "active" : ""}`}
          onClick={() => setActiveTab("dashboard")}
        >
          Панель
        </button>
        <button 
          className={`nav-item ${activeTab === "editor" ? "active" : ""}`}
          onClick={() => setActiveTab("editor")}
        >
          Списки
        </button>
        <button 
          className={`nav-item ${activeTab === "settings" ? "active" : ""}`}
          onClick={() => setActiveTab("settings")}
        >
          Настройки
        </button>
      </nav>

      {/* Контентная область */}
      <main className="cozy-content">
        {activeTab === "dashboard" && (
          <div className="tab-pane dashboard-pane">
            <div className="status-card">
              <span className="status-label">Текущее состояние</span>
              <h2 className={`status-value ${isEnabled ? "connected" : isTuning ? "tuning" : ""}`}>
                {statusText}
              </h2>
            </div>

            {/* Главная кнопка-тумблер */}
            <div className="power-container">
              <button 
                className={`power-button ${isEnabled ? "active" : ""} ${isTuning ? "loading" : ""}`}
                onClick={handleToggleBypass}
                disabled={isTuning}
              >
                {isTuning ? (
                  <svg className="spinner" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3">
                    <circle cx="12" cy="12" r="10" strokeDasharray="32" strokeDashoffset="8" />
                  </svg>
                ) : (
                  <svg className="power-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
                    <path d="M18.36 6.64a9 9 0 1 1-12.73 0M12 2v10" />
                  </svg>
                )}
              </button>
              <span className="power-hint">
                {isEnabled ? "Нажмите, чтобы выключить" : isTuning ? "Идет тестирование..." : "Нажмите для включения"}
              </span>
            </div>

            {/* Выбор стратегии обхода */}
            <div className="control-group">
              <label className="control-label">Стратегия обхода DPI</label>
              <select 
                className="cozy-select"
                value={strategy}
                onChange={(e) => setStrategy(e.target.value)}
                disabled={isEnabled || isTuning}
              >
                <option value="standard">Стандартная (Multisplit - по умолчанию)</option>
                <option value="alt">Альтернативная (ALT - Fake & FakeDSplit)</option>
                <option value="simple">Простая (Simple Fake - TLS/HTTP Fooling)</option>
                <option value="autotune">Автоподбор (Протестировать и выбрать лучшую)</option>
              </select>
              <p className="select-description">
                {strategy === "autotune" 
                  ? "Система автоматически проверит доступность YouTube на всех стратегиях и выберет рабочую." 
                  : "Если выбранная стратегия не разблокирует YouTube, отключите обход и попробуйте другую."}
              </p>
            </div>
          </div>
        )}

        {activeTab === "editor" && (
          <div className="tab-pane editor-pane">
            <div className="editor-controls">
              <div className="select-wrapper">
                <label className="control-label">Файл списка</label>
                <select 
                  className="cozy-select"
                  value={selectedList}
                  onChange={(e) => setSelectedList(e.target.value)}
                >
                  <option value="list-general-user.txt">Основной список (list-general-user)</option>
                  <option value="list-exclude-user.txt">Исключения сайтов (list-exclude-user)</option>
                  <option value="ipset-exclude-user.txt">Исключения IP-адресов (ipset-exclude-user)</option>
                  <option value="ipset-all.txt">Список IP-адресов (ipset-all)</option>
                </select>
              </div>
              {listSaveStatus && <span className="save-status">{listSaveStatus}</span>}
            </div>

            <textarea 
              className="cozy-textarea"
              value={listContent}
              onChange={(e) => setListContent(e.target.value)}
              placeholder="Введите домены (по одному на строку)..."
              spellCheck="false"
            />

            <button className="cozy-btn btn-primary" onClick={handleSaveList}>
              Сохранить изменения
            </button>
          </div>
        )}

        {activeTab === "settings" && (
          <div className="tab-pane settings-pane">
            <h3 className="section-title">Системные настройки</h3>
            
            <div className="settings-list">
              <div className="setting-item">
                <div className="setting-info">
                  <span className="setting-name">Автозапуск вместе с Windows</span>
                  <span className="setting-desc">Запускать RazzBlock при входе в систему в свернутом режиме</span>
                </div>
                <label className="cozy-switch">
                  <input 
                    type="checkbox" 
                    checked={autostart} 
                    onChange={(e) => handleToggleAutostart(e.target.checked)}
                  />
                  <span className="switch-slider"></span>
                </label>
              </div>

              <div className="setting-item">
                <div className="setting-info">
                  <span className="setting-name">Работа в системном трее</span>
                  <span className="setting-desc">При закрытии окна (нажатии на 'X') программа остается активной в трее</span>
                </div>
                <div className="setting-status">Включено</div>
              </div>
            </div>

            <div className="cozy-info-card">
              <svg className="info-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="12" cy="12" r="10" />
                <line x1="12" y1="16" x2="12" y2="12" />
                <line x1="12" y1="8" x2="12.01" y2="8" />
              </svg>
              <div className="info-text">
                <p><strong>RazzBlock</strong> работает на базе утилиты ядра WinDivert. Для перехвата пакетов приложению необходимы права администратора, которые запрашиваются при старте.</p>
                <p style={{ marginTop: "6px" }}>Все временные файлы драйверов извлекаются в Temp и автоматически удаляются при выходе из приложения.</p>
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
