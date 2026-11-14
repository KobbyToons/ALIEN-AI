document.addEventListener("DOMContentLoaded", async () => {
  const statusDot = document.getElementById("statusDot");
  const statusText = document.getElementById("statusText");
  const activeTabTitle = document.getElementById("activeTabTitle");
  const activeTabUrl = document.getElementById("activeTabUrl");
  const toggleEnabled = document.getElementById("toggleEnabled");
  const btnPing = document.getElementById("btnPing");
  const btnRunTest = document.getElementById("btnRunTest");
  const btnClearActions = document.getElementById("btnClearActions");
  const actionsList = document.getElementById("actionsList");
  const inputPort = document.getElementById("inputPort");
  const btnSavePort = document.getElementById("btnSavePort");

  let currentPort = 4114;

  // Load stored settings
  const stored = await chrome.storage.local.get([
    "alien_port",
    "alien_enabled",
    "alien_actions",
  ]);

  if (stored.alien_port) {
    currentPort = stored.alien_port;
    inputPort.value = currentPort;
  }
  if (stored.alien_enabled !== undefined) {
    toggleEnabled.checked = stored.alien_enabled;
  }

  // Load Active Tab
  try {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (tab) {
      activeTabTitle.textContent = tab.title || "Untitled Tab";
      activeTabUrl.textContent = tab.url || "about:blank";
    }
  } catch (err) {
    activeTabTitle.textContent = "Unable to read active tab";
  }

  // Check Bridge Status
  async function checkBridgeStatus() {
    statusDot.className = "status-dot";
    statusText.textContent = "Connecting...";

    try {
      const start = performance.now();
      const resp = await fetch(`http://127.0.0.1:${currentPort}/health`, {
        signal: AbortSignal.timeout(2000),
      });

      if (resp.ok) {
        const pingMs = Math.round(performance.now() - start);
        statusDot.className = "status-dot connected";
        statusText.textContent = `Online (${pingMs}ms)`;
      } else {
        throw new Error("HTTP " + resp.status);
      }
    } catch (_) {
      statusDot.className = "status-dot disconnected";
      statusText.textContent = "Offline (Bridge Idle)";
    }
  }

  // Render Action Feed
  function renderActions(actions) {
    if (!actions || actions.length === 0) {
      actionsList.innerHTML = `<div class="empty-state">No recent actions. Ready for ALIEN commands.</div>`;
      return;
    }

    actionsList.innerHTML = actions
      .slice(0, 15)
      .map(
        (a) => `
        <div class="action-item ${a.success ? "" : "failed"}">
          <div class="action-header">
            <span>${a.success ? "⚡" : "❌"} ${escapeHtml(a.action)}</span>
            <span class="action-time">${a.timestamp}</span>
          </div>
          <div class="action-detail" title="${escapeHtml(a.details)}">${escapeHtml(a.details)}</div>
        </div>
      `
      )
      .join("");
  }

  function escapeHtml(str) {
    return String(str || "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }

  renderActions(stored.alien_actions || []);
  checkBridgeStatus();

  // Listen for storage changes to refresh feed live
  chrome.storage.onChanged.addListener((changes, area) => {
    if (area === "local" && changes.alien_actions) {
      renderActions(changes.alien_actions.newValue || []);
    }
  });

  // Toggle Automation
  toggleEnabled.addEventListener("change", async () => {
    await chrome.storage.local.set({ alien_enabled: toggleEnabled.checked });
  });

  // Save Port
  btnSavePort.addEventListener("click", async () => {
    const val = parseInt(inputPort.value, 10);
    if (val >= 1024 && val <= 65535) {
      currentPort = val;
      await chrome.storage.local.set({ alien_port: val });
      checkBridgeStatus();
    }
  });

  // Ping Button
  btnPing.addEventListener("click", () => {
    checkBridgeStatus();
  });

  // Run In-Browser Test Button
  btnRunTest.addEventListener("click", async () => {
    btnRunTest.disabled = true;
    btnRunTest.textContent = "Testing...";

    try {
      const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
      if (!tab || !tab.id) throw new Error("No active tab.");

      const [res] = await chrome.scripting.executeScript({
        target: { tabId: tab.id },
        func: () => {
          const start = performance.now();
          const mathTest = 10 * 5 + 4 === 54;
          const domTest = document.body !== null;
          const userAgent = navigator.userAgent;
          const dur = Math.round(performance.now() - start);

          return {
            passed: mathTest && domTest,
            userAgent,
            durationMs: dur,
          };
        },
      });

      const data = res.result;
      const msg = data.passed
        ? `Code test PASSED (${data.durationMs}ms)`
        : "Code test FAILED";

      // Log action
      const storedActions = (await chrome.storage.local.get(["alien_actions"])).alien_actions || [];
      storedActions.unshift({
        id: Date.now().toString(),
        action: "manual_code_test",
        details: msg,
        success: data.passed,
        timestamp: new Date().toLocaleTimeString(),
      });
      await chrome.storage.local.set({ alien_actions: storedActions });
      renderActions(storedActions);
      alert(`🧪 ${msg}\nBrowser: ${data.userAgent.includes("Brave") ? "Brave Browser" : "Chromium"}`);
    } catch (err) {
      alert("Test Error: " + err.message);
    } finally {
      btnRunTest.disabled = false;
      btnRunTest.textContent = "🧪 Run Code Test";
    }
  });

  // Clear Actions
  btnClearActions.addEventListener("click", async () => {
    await chrome.storage.local.set({ alien_actions: [] });
    renderActions([]);
  });
});
