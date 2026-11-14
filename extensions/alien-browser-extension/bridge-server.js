#!/usr/bin/env node
/**
 * ALIEN Assistant — Local Browser Bridge Server
 * Facilitates bidirectional real-time communication between ALIEN and Brave Browser.
 * Port: 4114
 */

import http from "node:http";
import url from "node:url";

const PORT = parseInt(process.env.ALIEN_PORT || process.argv[2] || "4114", 10);

let extensionConnected = false;
let lastExtensionSeen = null;
let browserMetadata = null;
let activeTabInfo = null;

// Command queues and waiters
const pendingCommands = new Map(); // id -> { resolve, reject, timer }
const waitingPollers = []; // array of { res, timer }
const queuedCommands = []; // array of { id, action, params }

function setCors(res) {
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");
}

function sendJson(res, statusCode, data) {
  setCors(res);
  res.writeHead(statusCode, { "Content-Type": "application/json" });
  res.end(JSON.stringify(data));
}

function parseJsonBody(req) {
  return new Promise((resolve, reject) => {
    let body = "";
    req.on("data", (chunk) => {
      body += chunk;
      if (body.length > 5 * 1024 * 1024) {
        reject(new Error("Payload too large"));
      }
    });
    req.on("end", () => {
      if (!body.trim()) return resolve({});
      try {
        resolve(JSON.parse(body));
      } catch (err) {
        reject(new Error("Invalid JSON: " + err.message));
      }
    });
    req.on("error", reject);
  });
}

const server = http.createServer(async (req, res) => {
  setCors(res);

  if (req.method === "OPTIONS") {
    res.writeHead(204);
    res.end();
    return;
  }

  const parsedUrl = url.parse(req.url, true);
  const pathname = parsedUrl.pathname;

  // ── GET /health & /status ──────────────────────────────────────────
  if (req.method === "GET" && (pathname === "/health" || pathname === "/status")) {
    const isRecentlyActive =
      extensionConnected &&
      lastExtensionSeen &&
      Date.now() - lastExtensionSeen < 45000;

    return sendJson(res, 200, {
      ok: true,
      service: "ALIEN Browser Bridge",
      port: PORT,
      extensionConnected: Boolean(isRecentlyActive),
      browser: browserMetadata ? browserMetadata.browser : "Brave",
      activeTab: activeTabInfo,
      queuedCommands: queuedCommands.length,
      waitingCommands: pendingCommands.size,
      lastSeenSecondsAgo: lastExtensionSeen
        ? Math.round((Date.now() - lastExtensionSeen) / 1000)
        : null,
    });
  }

  // ── POST /register (Extension reports in) ──────────────────────────
  if (req.method === "POST" && pathname === "/register") {
    try {
      const data = await parseJsonBody(req);
      extensionConnected = true;
      lastExtensionSeen = Date.now();
      browserMetadata = data;
      if (data.activeTab) activeTabInfo = data.activeTab;
      return sendJson(res, 200, { ok: true, registered: true });
    } catch (e) {
      return sendJson(res, 400, { ok: false, error: e.message });
    }
  }

  // ── GET /poll (Extension long-poll for incoming commands) ──────────
  if (req.method === "GET" && pathname === "/poll") {
    extensionConnected = true;
    lastExtensionSeen = Date.now();

    // If there is already a queued command, dispatch immediately
    if (queuedCommands.length > 0) {
      const cmd = queuedCommands.shift();
      return sendJson(res, 200, cmd);
    }

    // Otherwise, hold response open for up to 25s (long polling)
    const pollerObj = { res, timer: null };

    pollerObj.timer = setTimeout(() => {
      const idx = waitingPollers.indexOf(pollerObj);
      if (idx !== -1) waitingPollers.splice(idx, 1);
      sendJson(res, 200, { noop: true });
    }, 25000);

    waitingPollers.push(pollerObj);
    req.on("close", () => {
      clearTimeout(pollerObj.timer);
      const idx = waitingPollers.indexOf(pollerObj);
      if (idx !== -1) waitingPollers.splice(idx, 1);
    });
    return;
  }

  // ── POST /result (Extension delivers execution outcome) ────────────
  if (req.method === "POST" && pathname === "/result") {
    try {
      const result = await parseJsonBody(req);
      extensionConnected = true;
      lastExtensionSeen = Date.now();

      const { id } = result;
      if (id && pendingCommands.has(id)) {
        const handler = pendingCommands.get(id);
        clearTimeout(handler.timer);
        pendingCommands.delete(id);
        handler.resolve(result);
      }

      return sendJson(res, 200, { ok: true, received: true });
    } catch (e) {
      return sendJson(res, 400, { ok: false, error: e.message });
    }
  }

  // ── POST /command (ALIEN AI dispatches an action to Brave) ─────────
  if (req.method === "POST" && pathname === "/command") {
    try {
      const cmd = await parseJsonBody(req);
      const action = cmd.action;
      const params = cmd.params || {};
      const timeoutMs = cmd.timeoutMs || 20000;

      if (!action) {
        return sendJson(res, 400, { ok: false, error: "Action is required." });
      }

      const isRecentlyActive =
        extensionConnected &&
        lastExtensionSeen &&
        Date.now() - lastExtensionSeen < 45000;

      if (!isRecentlyActive && waitingPollers.length === 0) {
        return sendJson(res, 200, {
          success: false,
          error: "Brave Extension is not connected. Please ensure Brave Browser is running with the ALIEN Assistant extension installed and enabled.",
        });
      }

      const id = "cmd_" + Date.now() + "_" + Math.random().toString(36).slice(2, 7);
      const commandPayload = { id, action, params };

      // Dispatch to waiting extension poller or queue
      if (waitingPollers.length > 0) {
        const poller = waitingPollers.shift();
        clearTimeout(poller.timer);
        sendJson(poller.res, 200, commandPayload);
      } else {
        queuedCommands.push(commandPayload);
      }

      // Wait for extension result
      const resultPromise = new Promise((resolve) => {
        const timer = setTimeout(() => {
          pendingCommands.delete(id);
          resolve({
            success: false,
            error: `Browser action '${action}' timed out after ${timeoutMs}ms. Ensure Brave extension is active.`,
          });
        }, timeoutMs);

        pendingCommands.set(id, { resolve, reject: resolve, timer });
      });

      const outcome = await resultPromise;
      return sendJson(res, 200, outcome);
    } catch (cmdErr) {
      return sendJson(res, 200, {
        ok: false,
        success: false,
        error: cmdErr.message || String(cmdErr),
      });
    }
  }

  // Default 404
  sendJson(res, 404, { ok: false, error: "Not Found" });
});

server.listen(PORT, "127.0.0.1", () => {
  console.log(`[ALIEN Bridge] Listening on http://127.0.0.1:${PORT}`);
});

server.on("error", (err) => {
  if (err.code === "EADDRINUSE") {
    console.log(`[ALIEN Bridge] Port ${PORT} already in use. Checking if existing bridge is alive...`);
    http.get(`http://127.0.0.1:${PORT}/health`, (r) => {
      if (r.statusCode === 200) {
        console.log(`[ALIEN Bridge] Existing bridge server is healthy and running.`);
        process.exit(0);
      } else {
        console.error(`[ALIEN Bridge] Port ${PORT} occupied by unknown service.`);
        process.exit(1);
      }
    }).on("error", () => {
      console.error(`[ALIEN Bridge] Port ${PORT} occupied, could not connect.`);
      process.exit(1);
    });
  } else {
    console.error("[ALIEN Bridge] Server error:", err);
    process.exit(1);
  }
});
