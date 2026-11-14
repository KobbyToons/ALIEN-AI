/**
 * ALIEN Assistant — Content Script
 * Injected into web pages to assist with DOM interaction and event handling.
 */

(() => {
  if (window.__ALIEN_EXTENSION_INJECTED__) return;
  window.__ALIEN_EXTENSION_INJECTED__ = true;

  chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    if (request.type === "ALIEN_PING") {
      sendResponse({ status: "alive", url: window.location.href, title: document.title });
      return true;
    }

    if (request.type === "ALIEN_HIGHLIGHT_ELEMENT") {
      const el = document.querySelector(request.selector);
      if (el) {
        const origOutline = el.style.outline;
        el.style.outline = "2px solid #ef4444";
        setTimeout(() => {
          el.style.outline = origOutline;
        }, 1500);
        sendResponse({ success: true });
      } else {
        sendResponse({ success: false, error: "Element not found" });
      }
      return true;
    }
  });
})();
