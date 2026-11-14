# Buddy

> Your personal AI assistant — lives in the system tray, summoned with **Ctrl+Space**.

## Prerequisites

| Tool | Install |
|------|---------|
| **Rust** (via rustup) | `rustup default stable` |
| **Node.js** ≥ 18 | [nodejs.org](https://nodejs.org) |
| **pnpm** | `npm install -g pnpm` |
| **Visual Studio 2022** Build Tools (MSVC + Windows SDK) | [visualstudio.microsoft.com](https://visualstudio.microsoft.com/downloads/) |
| **WebView2 Runtime** | Pre-installed on Windows 10/11 with Edge |

## Getting Started

```powershell
cd buddy
pnpm install
pnpm tauri dev
```

The first run takes a few minutes while Cargo compiles the Rust backend.
After that, hot-reload is near-instant for frontend changes.

## Generate Icons (before building installer)

Place a 1024×1024 PNG source image, then run:

```powershell
pnpm tauri icon path/to/your-icon.png
```

This creates all sizes in `src-tauri/icons/`.

## Build Installer

```powershell
pnpm tauri build
```

Outputs `.msi` and `.exe` installers in `src-tauri/target/release/bundle/`.

## Project Structure

```
buddy/
├── src/                  # React frontend (TypeScript + Tailwind)
│   ├── components/       # Widget, BuddyOrb, ChatPanel, TitleBar
│   ├── hooks/            # useGlobalShortcut, (useChat, useVoice — later)
│   ├── lib/              # Tauri IPC wrappers
│   └── types/            # Shared TypeScript types
├── src-tauri/            # Rust backend
│   ├── src/
│   │   ├── lib.rs        # App setup, plugins, global shortcut
│   │   ├── tray.rs       # System-tray icon + events
│   │   ├── commands/     # IPC commands (shell, fs, process — Phase 2)
│   │   ├── permissions/  # Permission gate + audit log  (Phase 2)
│   │   ├── services/     # MCP, keyring, Obsidian       (Phase 4–7)
│   │   └── skills/       # Plug-in skill loader          (Phase 9)
│   ├── capabilities/     # Tauri v2 permission scopes
│   └── tauri.conf.json   # Window, bundle, build config
└── package.json
```

