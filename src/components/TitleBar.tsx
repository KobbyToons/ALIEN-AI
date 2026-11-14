import { useEffect, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getKillSwitch } from "../lib/tauri";

interface TitleBarProps {
  onOpenSettings: () => void;
  onCollapse?: () => void;
}

/**
 * Custom title bar for the frameless window with Black & Red aesthetics.
 * The `data-tauri-drag-region` attribute lets users drag the window.
 */
export default function TitleBar({ onOpenSettings, onCollapse }: TitleBarProps) {
  const [killSwitchActive, setKillSwitchActive] = useState(false);

  useEffect(() => {
    const check = () => {
      getKillSwitch().then(setKillSwitchActive).catch(() => {});
    };
    check();
    const interval = setInterval(check, 2000);
    return () => clearInterval(interval);
  }, []);

  const handleMinimize = async () => {
    try {
      const win = getCurrentWebviewWindow();
      await win.hide();
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div
      data-tauri-drag-region
      className="
        flex items-center justify-between
        h-10 px-4 select-none cursor-move
        text-[11px] font-semibold tracking-wider uppercase
        bg-[#060608] border-b border-red-950/80 text-neutral-400
      "
    >
      <div className="flex items-center gap-2" data-tauri-drag-region>
        <span data-tauri-drag-region className="text-red-500 font-black tracking-widest drop-shadow-[0_0_8px_rgba(239,68,68,0.6)]">
          ALIEN
        </span>
        {killSwitchActive && (
          <span className="bg-red-600 text-white text-[9px] px-1.5 py-0.5 rounded-full font-bold animate-pulse shadow-sm shadow-red-600/40">
            KILL SWITCH
          </span>
        )}
      </div>

      <div className="flex items-center gap-1.5" data-tauri-drag-region>
        {onCollapse && (
          <button
            onClick={onCollapse}
            className="p-1 rounded-md hover:bg-neutral-800 transition-colors text-neutral-400 hover:text-red-400"
            title="Collapse to Orb"
          >
            ⚪
          </button>
        )}

        {/* Settings button */}
        <button
          onClick={onOpenSettings}
          className="p-1 rounded-md hover:bg-neutral-800 transition-colors text-neutral-400 hover:text-red-400"
          title="Settings & Brain"
        >
          ⚙️
        </button>

        {/* Hide/Minimize button */}
        <button
          onClick={handleMinimize}
          className="p-1 px-1.5 rounded-md hover:bg-neutral-800 transition-colors text-neutral-400 hover:text-red-400 font-bold"
          title="Minimize to Tray (Ctrl+Space)"
        >
          ─
        </button>
      </div>
    </div>
  );
}
