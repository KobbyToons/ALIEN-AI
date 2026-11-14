# ALIEN Assistant — Brave Browser Extension

A Manifest V3 extension connecting **Brave Browser** directly to the **ALIEN AI Desktop Assistant**.

## Capabilities

With this extension installed, ALIEN can:
- **Navigate & Explore**: Visit any URL (`browser_navigate`), open new tabs, or switch tabs seamlessly.
- **Inspect Web Content**: Extract page text, markdown structure, links, forms, and buttons (`browser_get_content`).
- **Execute JavaScript**: Run arbitrary scripts inside active Brave tabs and capture return values (`browser_eval_js`).
- **Interactive Actions**: Click buttons, submit forms, and type text into inputs (`browser_click`, `browser_type`).
- **In-Browser Code Testing**: Run automated JavaScript, HTML, and web application code tests directly in Brave, measuring execution time and capturing console logs, assertion checks, and errors (`browser_run_code_test`).
- **Visual Capture**: Capture screenshots of visible viewports (`browser_screenshot`).

---

## Architecture

- **Extension**: Chromium Manifest V3 extension (`manifest.json`, `background.js`, `popup.html`, `content.js`).
- **Bridge Server**: Local low-latency HTTP bridge on `127.0.0.1:4114` (`bridge-server.js`).
- **Skill Engine**: Integrated into ALIEN via `buddy/skills/browser/manifest.json`.
- **Packaging**: Both unpacked folder and signed `.crx` (`alien-browser-extension.crx`) with stable RSA ID `afblndojgjjnemhbljhfbnobgnigbpoi`.

---

## Installation in Brave Browser

1. Run `install-to-brave.bat` (or `powershell -ExecutionPolicy Bypass -File install-to-brave.ps1`).
2. Brave will open `brave://extensions`.
3. Toggle **Developer mode** in the top right corner.
4. Click **Load unpacked** in the top left corner.
5. Paste the path (already copied to your clipboard) into the file dialog and click **Select Folder**.
6. The ALIEN extension icon will appear in your Brave toolbar with a glowing red indicator when connected!
