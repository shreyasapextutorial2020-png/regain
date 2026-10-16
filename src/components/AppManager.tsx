import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ShieldAlert, RefreshCw, Plus, Check } from "lucide-react";

interface InstalledApp {
  name: string;
  executable_name: string | null;
  is_distraction: boolean;
  category: string;
}

export function AppManager() {
  const [apps, setApps] = useState<InstalledApp[]>([]);
  const [scanning, setScanning] = useState(false);
  const [blockedTargets, setBlockedTargets] = useState<string[]>([]);

  const handleScan = async () => {
    setScanning(true);
    try {
      const result = await invoke<InstalledApp[]>("scan_installed_apps");
      setApps(result);
      const currentList = await invoke<string[]>("get_blocklist");
      setBlockedTargets(currentList.map((t) => t.toLowerCase()));
    } catch (err) {
      console.error("Failed to scan apps:", err);
    } finally {
      setScanning(false);
    }
  };

  const handleBlockAll = async () => {
    try {
      const updatedList = await invoke<string[]>("auto_block_all_distractions");
      setBlockedTargets(updatedList.map((t) => t.toLowerCase()));
    } catch (err) {
      console.error("Failed to auto-block distractions:", err);
    }
  };

  const toggleApp = async (app: InstalledApp) => {
    const target = (app.executable_name || app.name).toLowerCase();
    const isCurrentlyBlocked = blockedTargets.includes(target);

    const nextList = isCurrentlyBlocked
      ? blockedTargets.filter((item) => item !== target)
      : [...blockedTargets, target];

    await invoke("update_blocklist", { list: nextList });
    setBlockedTargets(nextList);
  };

  return (
    <div style={{ marginTop: "1.5rem" }}>
      <div style={{ display: "flex", gap: "0.75rem", marginBottom: "1rem" }}>
        <button
          onClick={handleScan}
          disabled={scanning}
          style={{
            padding: "0.5rem 1rem",
            borderRadius: "6px",
            background: "#21262d",
            color: "#fff",
            border: "1px solid #30363d",
            cursor: "pointer",
            display: "flex",
            alignItems: "center",
            gap: "0.5rem",
          }}
        >
          <RefreshCw size={16} className={scanning ? "animate-spin" : ""} />
          {scanning ? "Scanning System..." : "Scan Installed Apps"}
        </button>

        <button
          onClick={handleBlockAll}
          style={{
            padding: "0.5rem 1rem",
            borderRadius: "6px",
            background: "#ef4444",
            color: "#fff",
            border: "none",
            cursor: "pointer",
            display: "flex",
            alignItems: "center",
            gap: "0.5rem",
          }}
        >
          <ShieldAlert size={16} /> Block All Detected Distractions
        </button>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem", maxHeight: "360px", overflowY: "auto" }}>
        {apps.map((app) => {
          const target = (app.executable_name || app.name).toLowerCase();
          const isBlocked = blockedTargets.includes(target);

          return (
            <div
              key={app.name}
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                padding: "0.6rem 0.8rem",
                borderRadius: "8px",
                background: "rgba(255, 255, 255, 0.04)",
                border: "1px solid rgba(255, 255, 255, 0.08)",
              }}
            >
              <div>
                <strong style={{ marginRight: "0.5rem" }}>{app.name}</strong>
                <span style={{ fontSize: "0.75rem", opacity: 0.6, marginRight: "0.5rem" }}>
                  {app.category}
                </span>
                {app.is_distraction && (
                  <span
                    style={{
                      fontSize: "0.7rem",
                      padding: "0.1rem 0.4rem",
                      borderRadius: "4px",
                      background: "rgba(239, 68, 68, 0.2)",
                      color: "#ef4444",
                    }}
                  >
                    Distraction
                  </span>
                )}
              </div>

              <button
                onClick={() => toggleApp(app)}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "0.3rem",
                  padding: "0.3rem 0.6rem",
                  borderRadius: "6px",
                  cursor: "pointer",
                  border: "none",
                  background: isBlocked ? "#238636" : "#30363d",
                  color: "#fff",
                }}
              >
                {isBlocked ? (
                  <>
                    <Check size={14} /> Blocked
                  </>
                ) : (
                  <>
                    <Plus size={14} /> Block
                  </>
                )}
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
}