/**
 * ALIEN Assistant — Brave Browser Extension Service Worker
 * Communicates with the local ALIEN AI desktop bridge on port 4114.
 */

const DEFAULT_PORT = 4114;
let bridgePort = DEFAULT_PORT;
let isPolling = false;
let extensionEnabled = true;

// Initialize configuration from storage
chrome.storage.local.get(["alien_port", "alien_enabled", "alien_actions"], (res) => {
  if (res.alien_port) bridgePort = res.alien_port;
  if (res.alien_enabled !== undefined) extensionEnabled = res.alien_enabled;
  initBridge();
});

// Watch for settings changes from popup
chrome.storage.onChanged.addListener((changes, area) => {
  if (area === "local") {
    if (changes.alien_port) bridgePort = changes.alien_port.newValue;
    if (changes.alien_enabled !== undefined) {
      extensionEnabled = changes.alien_enabled.newValue;
      if (extensionEnabled && !isPolling) {
        startLongPoll();
      }
    }
  }
});

// Keepalive alarm to ensure service worker stays responsive in Manifest V3
chrome.alarms.create("alien_keepalive", { periodInMinutes: 0.5 });
chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === "alien_keepalive" && extensionEnabled && !isPolling) {
    startLongPoll();
  }
});

function getBridgeUrl(endpoint) {
  return `http://127.0.0.1:${bridgePort}${endpoint}`;
}

async function logAction(actionName, details, success = true) {
  const item = {
    id: Date.now().toString(),
    action: actionName,
    details: typeof details === "string" ? details : JSON.stringify(details),
    success,
    timestamp: new Date().toLocaleTimeString(),
  };

  const stored = await chrome.storage.local.get(["alien_actions"]);
  const actions = stored.alien_actions || [];
  actions.unshift(item);
  if (actions.length > 30) actions.pop();
  await chrome.storage.local.set({ alien_actions: actions, alien_last_active: Date.now() });
}

async function getActiveTab() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (tab) return tab;
  const all = await chrome.tabs.query({ active: true });
  return all[0] || null;
}

// Register browser presence with desktop bridge
async function registerWithBridge() {
  try {
    const activeTab = await getActiveTab();
    await fetch(getBridgeUrl("/register"), {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        browser: "Brave",
        userAgent: navigator.userAgent,
        activeTab: activeTab
          ? { id: activeTab.id, title: activeTab.title, url: activeTab.url }
          : null,
      }),
    });
  } catch (_) {
    // Bridge might not be started yet
  }
}

function initBridge() {
  registerWithBridge();
  startLongPoll();
}

// Main Long-Polling Loop
async function startLongPoll() {
  if (isPolling || !extensionEnabled) return;
  isPolling = true;

  while (extensionEnabled) {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 28000);

      const resp = await fetch(getBridgeUrl("/poll"), {
        signal: controller.signal,
      });
      clearTimeout(timeoutId);

      if (!resp.ok) {
        await new Promise((r) => setTimeout(r, 2000));
        continue;
      }

      const payload = await resp.json();

      if (payload && payload.id && payload.action) {
        handleCommand(payload);
      }
    } catch (err) {
      // Back off on connection refusal or timeout
      await new Promise((r) => setTimeout(r, 2000));
    }
  }

  isPolling = false;
}

// Dispatch incoming command to handlers
async function handleCommand(cmd) {
  const { id, action, params = {} } = cmd;
  let result = null;
  let isSuccess = true;
  let errorMsg = null;

  try {
    switch (action) {
      case "navigate":
        result = await actionNavigate(params);
        break;
      case "get_content":
        result = await actionGetContent(params);
        break;
      case "eval_js":
        result = await actionEvalJs(params);
        break;
      case "click":
        result = await actionClick(params);
        break;
      case "type":
        result = await actionType(params);
        break;
      case "run_code_test":
        result = await actionRunCodeTest(params);
        break;
      case "list_tabs":
        result = await actionListTabs();
        break;
      case "switch_tab":
        result = await actionSwitchTab(params);
        break;
      case "close_tab":
        result = await actionCloseTab(params);
        break;
      case "screenshot":
        result = await actionScreenshot();
        break;
      default:
        throw new Error(`Unknown browser action: ${action}`);
    }
    await logAction(action, result);
  } catch (err) {
    isSuccess = false;
    errorMsg = err.message || String(err);
    await logAction(action, errorMsg, false);
  }

  // Report result back to desktop bridge
  try {
    await fetch(getBridgeUrl("/result"), {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        id,
        success: isSuccess,
        data: result,
        error: errorMsg,
      }),
    });
  } catch (sendErr) {
    console.error("Failed to send command result to bridge:", sendErr);
  }
}

// ── Action Handlers ──────────────────────────────────────────────────

async function actionNavigate(params) {
  let { url, newTab = true } = params;
  if (!url) throw new Error("URL is required.");
  if (!/^https?:\/\//i.test(url) && !/^brave:\/\//i.test(url) && !/^file:\/\//i.test(url)) {
    url = "https://" + url;
  }

  let targetTabId = null;

  if (newTab) {
    const tab = await chrome.tabs.create({ url, active: true });
    targetTabId = tab.id;
  } else {
    const active = await getActiveTab();
    if (!active) {
      const tab = await chrome.tabs.create({ url, active: true });
      targetTabId = tab.id;
    } else {
      await chrome.tabs.update(active.id, { url });
      targetTabId = active.id;
    }
  }

  // Wait for the tab to reach 'complete' status
  await new Promise((resolve) => {
    const timer = setTimeout(resolve, 12000); // Max 12s wait
    const listener = (tabId, info) => {
      if (tabId === targetTabId && info.status === "complete") {
        clearTimeout(timer);
        chrome.tabs.onUpdated.removeListener(listener);
        resolve();
      }
    };
    chrome.tabs.onUpdated.addListener(listener);
  });

  const updatedTab = await chrome.tabs.get(targetTabId);
  return {
    tabId: updatedTab.id,
    url: updatedTab.url,
    title: updatedTab.title,
    navigated: true,
  };
}

async function actionGetContent(params) {
  const activeTab = await getActiveTab();
  if (!activeTab || !activeTab.id) throw new Error("No active tab found.");

  const maxLength = params.max_length || 6000;

  // Execute extraction in active tab
  const [execResult] = await chrome.scripting.executeScript({
    target: { tabId: activeTab.id },
    func: (limit) => {
      const clone = document.body ? document.body.cloneNode(true) : null;
      if (!clone) return { title: document.title, url: window.location.href, text: "" };

      // Remove non-content elements
      const badTags = clone.querySelectorAll("script, style, noscript, svg, canvas, iframe");
      badTags.forEach((el) => el.remove());

      const rawText = clone.innerText || clone.textContent || "";
      const cleanedText = rawText.replace(/\s+/g, " ").trim();

      // Collect key links
      const links = Array.from(document.querySelectorAll("a[href]"))
        .slice(0, 25)
        .map((a) => ({
          text: a.innerText.trim().slice(0, 80),
          href: a.href,
        }))
        .filter((l) => l.text.length > 0 && !l.href.startsWith("javascript:"));

      // Collect buttons
      const buttons = Array.from(document.querySelectorAll("button, [role='button'], input[type='button'], input[type='submit']"))
        .slice(0, 20)
        .map((b) => ({
          text: (b.innerText || b.value || "").trim().slice(0, 60),
          id: b.id || "",
          className: b.className || "",
        }))
        .filter((b) => b.text.length > 0);

      // Collect inputs
      const inputs = Array.from(document.querySelectorAll("input:not([type='hidden']), textarea, select"))
        .slice(0, 15)
        .map((i) => ({
          name: i.name || i.id || "",
          type: i.type || "text",
          placeholder: i.placeholder || "",
        }));

      return {
        title: document.title,
        url: window.location.href,
        text: cleanedText.slice(0, limit),
        totalLength: cleanedText.length,
        links,
        buttons,
        inputs,
      };
    },
    args: [maxLength],
  });

  return execResult.result;
}

async function actionEvalJs(params) {
  const activeTab = await getActiveTab();
  if (!activeTab || !activeTab.id) throw new Error("No active tab found.");
  if (!params.code) throw new Error("Code parameter is required.");

  const [execResult] = await chrome.scripting.executeScript({
    target: { tabId: activeTab.id },
    world: "MAIN",
    func: async (codeStr) => {
      try {
        const fn = new Function(`return (async () => { ${codeStr} })()`);
        const val = await fn();
        return {
          success: true,
          result: val !== undefined ? JSON.parse(JSON.stringify(val)) : null,
        };
      } catch (err) {
        return {
          success: false,
          error: err.message,
          stack: err.stack,
        };
      }
    },
    args: [params.code],
  });

  return execResult.result;
}

async function actionClick(params) {
  const activeTab = await getActiveTab();
  if (!activeTab || !activeTab.id) throw new Error("No active tab found.");
  const { selector, text } = params;

  const [execResult] = await chrome.scripting.executeScript({
    target: { tabId: activeTab.id },
    func: (sel, txt) => {
      let el = null;
      if (sel) {
        el = document.querySelector(sel);
      }
      if (!el && txt) {
        const lower = txt.toLowerCase();
        const candidates = document.querySelectorAll("button, a, input[type='button'], input[type='submit'], [role='button'], div, span");
        for (const c of candidates) {
          if (c.textContent && c.textContent.trim().toLowerCase().includes(lower)) {
            el = c;
            break;
          }
        }
      }

      if (!el) {
        return { success: false, error: `Element not found for selector='${sel}' or text='${txt}'` };
      }

      el.scrollIntoView({ behavior: "instant", block: "center" });

      const opts = { bubbles: true, cancelable: true, view: window };
      el.dispatchEvent(new MouseEvent("pointerdown", opts));
      el.dispatchEvent(new MouseEvent("mousedown", opts));
      el.dispatchEvent(new MouseEvent("pointerup", opts));
      el.dispatchEvent(new MouseEvent("mouseup", opts));
      el.click();

      return {
        success: true,
        clickedTag: el.tagName,
        clickedText: (el.textContent || el.value || "").trim().slice(0, 80),
      };
    },
    args: [selector || null, text || null],
  });

  if (!execResult.result.success) {
    throw new Error(execResult.result.error);
  }
  return execResult.result;
}

async function actionType(params) {
  const activeTab = await getActiveTab();
  if (!activeTab || !activeTab.id) throw new Error("No active tab found.");
  const { selector, text, clear = true } = params;
  if (!selector) throw new Error("Selector is required for typing.");

  const [execResult] = await chrome.scripting.executeScript({
    target: { tabId: activeTab.id },
    func: (sel, val, shouldClear) => {
      const el = document.querySelector(sel);
      if (!el) return { success: false, error: `Input element not found: ${sel}` };

      el.focus();
      if (shouldClear) {
        el.value = "";
        el.dispatchEvent(new Event("input", { bubbles: true }));
      }

      el.value = (el.value || "") + val;
      el.dispatchEvent(new Event("input", { bubbles: true }));
      el.dispatchEvent(new Event("change", { bubbles: true }));
      el.dispatchEvent(new Event("blur", { bubbles: true }));

      return { success: true, selector: sel, currentValue: el.value };
    },
    args: [selector, text || "", clear],
  });

  if (!execResult.result.success) {
    throw new Error(execResult.result.error);
  }
  return execResult.result;
}

// ── In-Browser Code Testing Engine ──────────────────────────────────
async function actionRunCodeTest(params) {
  const { code, test_name = "Anonymous Code Test" } = params;
  if (!code) throw new Error("Code parameter is required.");

  const activeTab = await getActiveTab();
  if (!activeTab || !activeTab.id) throw new Error("No active tab found for test execution.");

  const [execResult] = await chrome.scripting.executeScript({
    target: { tabId: activeTab.id },
    world: "MAIN",
    func: async (codeToRun, testTitle) => {
      const logs = [];
      const errors = [];
      const origLog = console.log;
      const origWarn = console.warn;
      const origErr = console.error;

      // Intercept console output during test execution
      console.log = (...args) => {
        logs.push("[LOG] " + args.map((a) => (typeof a === "object" ? JSON.stringify(a) : String(a))).join(" "));
        origLog.apply(console, args);
      };
      console.warn = (...args) => {
        logs.push("[WARN] " + args.map((a) => (typeof a === "object" ? JSON.stringify(a) : String(a))).join(" "));
        origWarn.apply(console, args);
      };
      console.error = (...args) => {
        errors.push("[ERROR] " + args.map((a) => (typeof a === "object" ? JSON.stringify(a) : String(a))).join(" "));
        origErr.apply(console, args);
      };

      const startTime = performance.now();
      let testPassed = true;
      let returnedValue = null;
      let failureReason = null;

      try {
        const fn = new Function(`
          return (async () => {
            ${codeToRun}
          })()
        `);
        returnedValue = await fn();
      } catch (err) {
        testPassed = false;
        failureReason = err.message;
        errors.push("[EXCEPTION] " + (err.stack || err.message));
      } finally {
        console.log = origLog;
        console.warn = origWarn;
        console.error = origErr;
      }

      const durationMs = Math.round((performance.now() - startTime) * 100) / 100;

      return {
        testTitle,
        passed: testPassed,
        failureReason,
        returnedValue: returnedValue !== undefined ? JSON.parse(JSON.stringify(returnedValue)) : null,
        logs,
        errors,
        durationMs,
      };
    },
    args: [code, test_name],
  });

  return execResult.result;
}

async function actionListTabs() {
  const tabs = await chrome.tabs.query({});
  return tabs.map((t) => ({
    id: t.id,
    title: t.title,
    url: t.url,
    active: t.active,
    windowId: t.windowId,
  }));
}

async function actionSwitchTab(params) {
  const { tabId } = params;
  if (!tabId) throw new Error("tabId is required.");
  const tab = await chrome.tabs.update(Number(tabId), { active: true });
  if (tab.windowId) {
    await chrome.windows.update(tab.windowId, { focused: true });
  }
  return { switched: true, tabId: tab.id, title: tab.title };
}

async function actionCloseTab(params) {
  const { tabId } = params;
  if (!tabId) throw new Error("tabId is required.");
  await chrome.tabs.remove(Number(tabId));
  return { closed: true, tabId };
}

async function actionScreenshot() {
  const dataUrl = await chrome.tabs.captureVisibleTab(null, { format: "png" });
  return {
    success: true,
    dataUrl,
    sizeBytes: dataUrl ? dataUrl.length : 0,
  };
}
