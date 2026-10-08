use crate::EXTENDED_INFO;

pub fn build_toolbar_html(initial_tab_id: u32, initial_url: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    * {{
      box-sizing: border-box;
      margin: 0;
      padding: 0;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    }}
    body {{
      background: #1e1f22;
      color: #e8eaed;
      height: 88px;
      overflow: hidden;
      display: flex;
      flex-direction: column;
      border-bottom: 1px solid #3c4043;
      user-select: none;
      -webkit-user-select: none;
    }}

    /* Top Tabs Bar */
    .tabs-bar {{
      height: 36px;
      background: #18191c;
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 10px 0 10px;
      border-bottom: 1px solid #2b2a33;
      flex-shrink: 0;
    }}

    .tabs-left-section {{
      display: flex;
      align-items: flex-end;
      gap: 4px;
      height: 100%;
      flex: 1;
      overflow-x: auto;
    }}
    .tabs-left-section::-webkit-scrollbar {{
      display: none;
    }}

    .tabs-list {{
      display: flex;
      align-items: flex-end;
      gap: 4px;
      height: 100%;
    }}

    .tab {{
      height: 31px;
      min-width: 140px;
      max-width: 220px;
      background: #232428;
      color: #9aa0a6;
      border-radius: 8px 8px 0 0;
      padding: 0 10px;
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 12px;
      cursor: pointer;
      position: relative;
      transition: background 0.15s, color 0.15s;
      border-top: 2px solid transparent;
    }}
    .tab:hover {{
      background: #2a2b30;
      color: #e8eaed;
    }}
    .tab.active {{
      background: #2b2a33;
      color: #ffffff;
      font-weight: 500;
      border-top: 2px solid #ff5a00; /* Brave orange accent */
    }}
    .tab-icon {{
      width: 14px;
      height: 14px;
      flex-shrink: 0;
      fill: currentColor;
      opacity: 0.8;
    }}
    .tab-title {{
      flex: 1;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      font-size: 11.5px;
    }}
    .tab-close {{
      width: 16px;
      height: 16px;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 11px;
      color: #9aa0a6;
      flex-shrink: 0;
    }}
    .tab-close:hover {{
      background: rgba(255, 255, 255, 0.15);
      color: #ffffff;
    }}

    .new-tab-btn {{
      width: 28px;
      height: 28px;
      border-radius: 50%;
      background: transparent;
      border: none;
      color: #9aa0a6;
      font-size: 18px;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      margin-bottom: 2px;
      transition: background 0.15s, color 0.15s;
      flex-shrink: 0;
    }}
    .new-tab-btn:hover {{
      background: rgba(255, 255, 255, 0.12);
      color: #ffffff;
    }}

    /* Top Right Controls & Auto Mode Button */
    .tabs-right-section {{
      display: flex;
      align-items: center;
      gap: 10px;
      margin-left: 12px;
      flex-shrink: 0;
    }}

    .auto-mode-btn {{
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 4px 12px;
      border-radius: 14px;
      font-size: 11.5px;
      font-weight: 600;
      cursor: pointer;
      border: 1px solid #4a4d52;
      background: #252830;
      color: #bdc1c6;
      transition: all 0.2s ease;
    }}
    .auto-mode-btn.active {{
      background: #1b5e20;
      border-color: #4caf50;
      color: #a5d6a7;
      box-shadow: 0 0 10px rgba(76, 175, 80, 0.4);
    }}
    .status-indicator {{
      width: 7px;
      height: 7px;
      border-radius: 50%;
      background: #757575;
    }}
    .auto-mode-btn.active .status-indicator {{
      background: #00e676;
      box-shadow: 0 0 6px #00e676;
    }}
    .key-badge {{
      background: rgba(0, 0, 0, 0.28);
      border: 1px solid rgba(255, 255, 255, 0.15);
      padding: 2px 7px;
      border-radius: 6px;
      font-size: 11px;
      font-weight: 500;
      color: #e8eaed;
      letter-spacing: 0.2px;
      white-space: nowrap;
    }}
    .extended-info-pill {{
      display: inline-flex;
      align-items: center;
      gap: 5px;
      background: #23252a;
      border: 1px solid #3c4043;
      padding: 3px 10px;
      border-radius: 12px;
      font-size: 11px;
      color: #9aa0a6;
      max-width: 420px;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }}
    .extended-info-pill strong {{
      color: #8ab4f8;
      font-weight: 600;
      flex-shrink: 0;
    }}
    .extended-info-pill span {{
      color: #bdc1c6;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      font-family: monospace;
    }}

    /* Navigation Bar */
    .nav-bar {{
      height: 52px;
      display: flex;
      align-items: center;
      padding: 0 10px;
      gap: 8px;
      background: #2b2a33;
      flex-shrink: 0;
    }}
    .nav-btn {{
      width: 32px;
      height: 32px;
      border-radius: 50%;
      background: transparent;
      border: none;
      color: #e8eaed;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      transition: background 0.15s;
      flex-shrink: 0;
    }}
    .nav-btn:hover {{
      background: rgba(255, 255, 255, 0.12);
    }}
    .nav-btn svg {{
      width: 17px;
      height: 17px;
      fill: currentColor;
    }}

    .url-box-wrapper {{
      flex: 1;
      height: 36px;
      background: #1e1f22;
      border-radius: 18px;
      display: flex;
      align-items: center;
      padding: 0 12px;
      border: 1px solid #3c4043;
      transition: border-color 0.15s, box-shadow 0.15s;
    }}
    .url-box-wrapper:focus-within {{
      border-color: #ff5a00;
      box-shadow: 0 0 0 2px rgba(255, 90, 0, 0.25);
    }}
    .lock-icon {{
      font-size: 13px;
      margin-right: 8px;
      color: #ff5a00;
      user-select: none;
    }}
    .url-input {{
      flex: 1;
      height: 100%;
      background: transparent;
      border: none;
      outline: none;
      color: #ffffff;
      font-size: 13px;
      font-family: inherit;
      user-select: text !important;
      -webkit-user-select: text !important;
    }}

    .input-actions {{
      display: flex;
      align-items: center;
      gap: 4px;
      margin-left: 6px;
    }}
    .mini-btn {{
      background: rgba(255, 255, 255, 0.08);
      color: #bdc1c6;
      border: 1px solid #3c4043;
      padding: 3px 8px;
      border-radius: 10px;
      font-size: 11px;
      cursor: pointer;
      font-weight: 500;
      transition: background 0.15s, color 0.15s;
    }}
    .mini-btn:hover {{
      background: rgba(255, 255, 255, 0.18);
      color: #fff;
    }}

    .go-btn {{
      background: #ff5a00;
      color: #ffffff;
      border: none;
      padding: 0 14px;
      height: 32px;
      border-radius: 16px;
      font-size: 12px;
      cursor: pointer;
      font-weight: 600;
      transition: background 0.15s;
      flex-shrink: 0;
    }}
    .go-btn:hover {{
      background: #e04e00;
    }}
  </style>
</head>
<body>
  <!-- Tabs Bar at Top -->
  <div class="tabs-bar">
    <div class="tabs-left-section">
      <div class="tabs-list" id="tabs-list">
        <div class="tab active" id="tab-{initial_tab_id}" onclick="switchTab({initial_tab_id})">
          <svg class="tab-icon" viewBox="0 0 24 24"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/></svg>
          <span class="tab-title" id="tab-title-{initial_tab_id}">New Tab</span>
          <span class="tab-close" onclick="closeTab({initial_tab_id}, event)">✕</span>
        </div>
      </div>
      <button class="new-tab-btn" title="New Tab" onclick="createNewTab()">+</button>
    </div>

    <!-- Top Right Corner Controls & Auto Mode Button -->
    <div class="tabs-right-section">
      <div class="extended-info-pill" title="Extended Info: {extended_info}">
        <strong>Extended Info:</strong>
        <span>{extended_info}</span>
      </div>
      <button class="auto-mode-btn" id="auto-btn" onclick="toggleAutoControl()" title="Toggle Auto Browsing (Keyboard shortcut: F8)">
        <span class="status-indicator"></span>
        <span id="auto-btn-text">Auto Control: OFF</span>
        <span class="key-badge" id="auto-countdown-badge" style="display: none;"></span>
      </button>
    </div>
  </div>

  <!-- Navigation Bar -->
  <div class="nav-bar">
    <button class="nav-btn" title="Back" onclick="sendAction('back')">
      <svg viewBox="0 0 24 24"><path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/></svg>
    </button>
    <button class="nav-btn" title="Forward" onclick="sendAction('forward')">
      <svg viewBox="0 0 24 24"><path d="M12 4l-1.41 1.41L16.17 11H4v2h12.17l-5.58 5.59L12 20l8-8z"/></svg>
    </button>
    <button class="nav-btn" title="Reload" onclick="sendAction('reload')">
      <svg viewBox="0 0 24 24"><path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/></svg>
    </button>
    <button class="nav-btn" title="Home" onclick="sendAction('navigate', 'https://www.google.com')">
      <svg viewBox="0 0 24 24"><path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/></svg>
    </button>

    <div class="url-box-wrapper">
      <span class="lock-icon">🔒</span>
      <input type="text" class="url-input" id="address" value="{url}" placeholder="Search or enter web address" spellcheck="false" autocomplete="off" />
      <div class="input-actions">
        <button class="mini-btn" title="Paste URL" onclick="handlePasteAction()">Paste</button>
        <button class="mini-btn" title="Copy URL" onclick="handleCopyAction()">Copy</button>
      </div>
    </div>
    <button class="go-btn" onclick="submitUrl()">Go</button>
  </div>

  <script>
    const input = document.getElementById("address");
    let activeTabId = {initial_tab_id};
    let isAutoActive = false;

    function sendAction(action, payload) {{
      if (window.ipc) {{
        window.ipc.postMessage(JSON.stringify({{ action: action, payload: payload }}));
      }}
    }}

    function submitUrl() {{
      const val = input.value.trim();
      if (val) {{
        sendAction("navigate", val);
      }}
    }}

    function handlePasteAction() {{
      sendAction("request_paste");
    }}

    function handleCopyAction() {{
      const val = input.value;
      if (val) {{
        sendAction("set_clipboard", val);
      }}
    }}

    function createNewTab() {{
      sendAction("create_tab", "https://www.google.com");
    }}

    function switchTab(tabId) {{
      activeTabId = tabId;
      document.querySelectorAll(".tab").forEach(t => t.classList.remove("active"));
      const el = document.getElementById("tab-" + tabId);
      if (el) el.classList.add("active");
      sendAction("switch_tab", tabId.toString());
    }}

    function closeTab(tabId, e) {{
      if (e) e.stopPropagation();
      sendAction("close_tab", tabId.toString());
    }}

    function toggleAutoControl() {{
      sendAction("toggle_auto_control", "");
    }}

    // Update UI state for Auto Mode
    window.setAutoControlState = function(enabled) {{
      isAutoActive = enabled;
      const btn = document.getElementById("auto-btn");
      const btnText = document.getElementById("auto-btn-text");
      const badge = document.getElementById("auto-countdown-badge");
      if (enabled) {{
        btn.classList.add("active");
        btnText.innerText = "Auto Control: ON";
      }} else {{
        btn.classList.remove("active");
        btnText.innerText = "Auto Control: OFF";
        if (badge) {{
          badge.style.display = "none";
          badge.innerText = "";
        }}
      }}
    }};

    // Update live countdown and upcoming event in badge (e.g. "11s => mouse click")
    window.updateAutoCountdown = function(remainingSecs, actionName) {{
      const badge = document.getElementById("auto-countdown-badge");
      if (!badge) return;
      if (remainingSecs > 0 && isAutoActive) {{
        const actionLabel = actionName ? ` (${{actionName}})` : "";
        badge.innerText = `${{remainingSecs}}s${{actionLabel}}`;
        badge.style.display = "inline-block";
      }} else {{
        badge.style.display = "none";
      }}
    }};

    // Called from Rust when tabs update
    window.renderTabs = function(tabsArray, activeId, currentUrl) {{
      activeTabId = activeId;
      const list = document.getElementById("tabs-list");
      list.innerHTML = "";

      tabsArray.forEach(tab => {{
        const div = document.createElement("div");
        div.className = "tab" + (tab.id === activeId ? " active" : "");
        div.id = "tab-" + tab.id;
        div.onclick = () => switchTab(tab.id);

        div.innerHTML = `
          <svg class="tab-icon" viewBox="0 0 24 24"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/></svg>
          <span class="tab-title" id="tab-title-${{tab.id}}">${{escapeHtml(tab.title)}}</span>
          <span class="tab-close" onclick="closeTab(${{tab.id}}, event)">✕</span>
        `;
        list.appendChild(div);
      }});

      if (currentUrl) {{
        input.value = currentUrl;
      }}
    }};

    window.setAddress = function(url) {{
      input.value = url;
    }};

    window.insertPaste = function(text) {{
      if (!text) return;
      const start = input.selectionStart || 0;
      const end = input.selectionEnd || 0;
      const current = input.value;
      input.value = current.substring(0, start) + text + current.substring(end);
      input.selectionStart = input.selectionEnd = start + text.length;
      input.focus();
    }};


    function escapeHtml(str) {{
      if (!str) return "New Tab";
      return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    }}

    // Global document listener for F8 shortcut and keys
    window.addEventListener("keydown", function(e) {{
      if (e.key === "F8") {{
        e.preventDefault();
        toggleAutoControl();
      }}
    }});

    input.addEventListener("keydown", function(e) {{
      if (e.key === "F8") {{
        e.preventDefault();
        toggleAutoControl();
        return;
      }}

      if (e.key === "Enter") {{
        submitUrl();
        return;
      }}

      if (e.metaKey || e.ctrlKey) {{
        if (e.key.toLowerCase() === "v") {{
          sendAction("request_paste");
        }} else if (e.key.toLowerCase() === "c") {{
          const sel = window.getSelection().toString();
          if (sel) {{
            sendAction("set_clipboard", sel);
          }} else {{
            sendAction("set_clipboard", input.value);
          }}
        }} else if (e.key.toLowerCase() === "a") {{
          input.select();
        }}
      }}
    }});
  </script>
</body>
</html>"#,
        initial_tab_id = initial_tab_id,
        url = initial_url,
        extended_info = EXTENDED_INFO
    )
}
