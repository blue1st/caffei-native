import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import packageJson from "../package.json";

interface AppStatus {
  is_on: boolean;
  is_manual: boolean;
  is_paused: boolean;
  active_reason: string | null;
  active_processes: string[];
}

interface ProcessItem {
  name: string;
}

function App() {
  const [status, setStatus] = useState<AppStatus>({ is_on: false, is_manual: false, is_paused: false, active_reason: null, active_processes: [] });
  const [processes, setProcesses] = useState<ProcessItem[]>([]);
  const [runningProcesses, setRunningProcesses] = useState<string[]>([]);
  const [showPicker, setShowPicker] = useState<boolean>(false);
  const [searchTerm, setSearchTerm] = useState<string>("");
  const [lastError, setLastError] = useState<string | null>(null);
  const [autostartEnabled, setAutostartEnabled] = useState<boolean>(false);

  useEffect(() => {
    // Check autostart status
    isEnabled()
      .then(setAutostartEnabled)
      .catch((e: any) => console.error("Failed to check autostart:", e));

    // Load initial data
    invoke("get_procs")
      .then((procs: unknown) => {
        const procArray = procs as string[];
        setProcesses(procArray.map(name => ({ name })));
      })
      .catch((e) => setLastError(String(e)));

    invoke("get_running_processes")
      .then((procs: unknown) => {
        setRunningProcesses(procs as string[]);
      })
      .catch((e) => setLastError(String(e)));

    const checkStatus = () => {
      invoke("get_status")
        .then((s: unknown) => {
          setStatus(s as AppStatus);
        })
        .catch((e) => {
          // Only set lastError if it's the first time or we really want to see it
          if (!lastError) setLastError(String(e));
        });
    };

    checkStatus();
    const interval = setInterval(checkStatus, 3000);
    return () => clearInterval(interval);
  }, []);

  const handleToggle = async () => {
    try {
      const newStatus = await invoke("toggle") as AppStatus;
      setStatus(newStatus);
      setLastError(null);
    } catch (error) {
      setLastError(`Toggle Error: ${error}`);
    }
  };

  const handleTogglePause = async () => {
    try {
      const newStatus = await invoke("toggle_pause") as AppStatus;
      setStatus(newStatus);
      setLastError(null);
    } catch (error) {
      setLastError(`Pause Toggle Error: ${error}`);
    }
  };

  const handleAddProcess = async (name: string) => {
    const trimmedName = name.trim();
    if (!trimmedName) return;

    try {
      const current = processes.map(p => p.name);
      if (current.includes(trimmedName)) {
        setShowPicker(false);
        return;
      }

      const nextProcs = [...current, trimmedName];
      await invoke("set_procs", { procs: nextProcs });

      setProcesses(prev => [...prev, { name: trimmedName }]);
      setShowPicker(false);
      setSearchTerm("");
      setLastError(null);
    } catch (error) {
      setLastError(`Add Error: ${error}`);
    }
  };

  const handlePickApp = async () => {
    try {
      const appName = await invoke("pick_app") as string;
      if (appName) {
        handleAddProcess(appName);
      }
      setLastError(null);
    } catch (error) {
      if (error !== "キャンセルされました") {
        setLastError(`Pick App Error: ${error}`);
      }
    }
  };

  const handleRemoveProcess = async (index: number) => {
    try {
      const current = processes
        .filter((_, i) => i !== index)
        .map(p => p.name);
      await invoke("set_procs", { procs: current });
      setProcesses(prev => prev.filter((_, i) => i !== index));
      setLastError(null);
    } catch (error) {
      setLastError(`Remove Error: ${error}`);
    }
  };

  const handleToggleAutostart = async () => {
    try {
      if (autostartEnabled) {
        await disable();
      } else {
        await enable();
      }
      setAutostartEnabled(!autostartEnabled);
    } catch (error) {
      setLastError(`Autostart Error: ${error}`);
    }
  };

  const filteredRunningProcesses = runningProcesses.filter(p =>
    p.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="app">
      <div className="card">
        <h1 className="title">☕ Caffei Native</h1>

        {lastError && (
          <div className="error-banner" style={{ background: '#ff4441', color: 'white', padding: '10px', borderRadius: '8px', marginBottom: '15px' }}>
            <strong>Error:</strong> {lastError}
          </div>
        )}

        <div className="control-section">
          <button
            className={`toggle-button ${status.is_on && status.is_manual ? 'active' : ''}`}
            onClick={handleToggle}
          >
            {status.is_on && status.is_manual ? "⏸️ 手動抑制を停止" : "▶️ 手動で抑制開始"}
          </button>

          <button
            className={`pause-button ${status.is_paused ? 'active' : ''}`}
            onClick={handleTogglePause}
            title={status.is_paused ? "監視を再開" : "監視を一時停止"}
          >
            {status.is_paused ? "🔄 監視を再開" : "⏸️ 監視を一時停止"}
          </button>
        </div>

        <div className={`status-display ${status.is_on ? 'active' : ''} ${status.is_paused ? 'paused' : ''}`}>
          <div className="status-dot"></div>
          <div className="status-text">
            <span className="label">現在の状態:</span>
            <span className="value">
              {status.is_on ? "スリープ抑制中" : (status.is_paused ? "一時停止中" : "通常（スリープ可能）")}
            </span>
          </div>
          {status.active_reason && (
            <div className="status-reason">
              <span className="label">理由:</span>
              <span className="value">{status.active_reason}</span>
            </div>
          )}
        </div>

        <div className="monitoring-section">
          <div className="section-header">
            <h2 className="monitoring-title">監視プロセス</h2>
            <div className="button-group">
              <button className="picker-btn secondary" onClick={() => setShowPicker(!showPicker)}>
                🔍 プロセスから選択
              </button>
              <button className="picker-btn" onClick={handlePickApp}>
                📂 アプリを直接選択
              </button>
            </div>
          </div>

          <p className="description">
            以下のプロセスが実行されている間、自動的にスリープ抑制を ON にします
          </p>

          {showPicker && (
            <div className="picker-overlay">
              <div className="picker-content">
                <input
                  type="text"
                  placeholder="プロセスを検索..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="search-input"
                  autoFocus
                />
                <div className="picker-list">
                  {filteredRunningProcesses.map(p => (
                    <div key={p} className="picker-item" onClick={() => handleAddProcess(p)}>
                      {p}
                    </div>
                  ))}
                  {filteredRunningProcesses.length === 0 && (
                    <div className="picker-empty">見つかりません</div>
                  )}
                </div>
                <button className="close-btn" onClick={() => setShowPicker(false)}>閉じる</button>
              </div>
            </div>
          )}

          <div className="process-list">
            {processes.map((proc, index) => {
              const isDetected = status.active_processes.includes(proc.name);
              return (
                <div key={index} className={`process-item ${isDetected ? 'detected' : ''} ${status.is_paused ? 'paused-item' : ''}`}>
                  <span className="process-name border-none">
                    {proc.name}
                    {isDetected && !status.is_paused && <span className="detected-badge">検出中</span>}
                    {isDetected && status.is_paused && <span className="detected-badge inactive">検出中</span>}
                  </span>
                  <button
                    className="remove-btn"
                    onClick={() => handleRemoveProcess(index)}
                  >
                    ×
                  </button>
                </div>
              );
            })}
            {processes.length === 0 && (
              <div className="empty-list">監視対象のプロセスはありません</div>
            )}
          </div>
        </div>

        <div className="settings-section" style={{ marginTop: '20px', padding: '15px', background: 'rgba(255,255,255,0.05)', borderRadius: '12px' }}>
          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer', gap: '10px' }}>
            <input
              type="checkbox"
              checked={autostartEnabled}
              onChange={handleToggleAutostart}
              style={{ width: '18px', height: '18px' }}
            />
            <span style={{ fontSize: '0.9rem' }}>ログイン時に自動起動する</span>
          </label>
        </div>

        <div className="version-info" style={{ marginTop: '20px', fontSize: '0.8rem', opacity: 0.6, textAlign: 'center' }}>
          Version: {packageJson.version}
        </div>
      </div>
    </div>
  );
}

export default App;
