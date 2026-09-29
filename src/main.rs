use core_graphics::event::{
    CGEvent, CGEventTapLocation, CGEventType, CGMouseButton,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWarpMouseCursorPosition(new_cursor_position: CGPoint) -> i32;
    fn CGEventCreate(source: *const std::ffi::c_void) -> *mut std::ffi::c_void;
    fn CGEventGetLocation(event: *const std::ffi::c_void) -> CGPoint;
    fn CGEventCreateScrollWheelEvent2(
        source: *const std::ffi::c_void,
        units: u32,
        wheel_count: u32,
        wheel1: i32,
        wheel2: i32,
        wheel3: i32,
    ) -> *mut std::ffi::c_void;
    fn CGEventPost(tap: u32, event: *const std::ffi::c_void);
    fn CFRelease(cf: *const std::ffi::c_void);
}
use rand::Rng;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    keyboard::KeyCode,
    window::WindowBuilder,
};
use wry::{Rect, WebView, WebViewBuilder};

// 4 Permanent & Static Metadata Values
pub const APP_NAME: &str = "Google Chrome";
pub const WINDOW_TITLE: &str = "Bluevy Admin - Google Chrome";
pub const APP_VERSION: &str = "8037.58";
pub const EXTENDED_INFO: &str = "http://localhost:3001/workspace/DataList?id=9b0f6bcb-c5d1-4c86-a4";

// Real Brave Browser / Chrome User Agent for direct native browsing
const CHROME_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";

const TOOLBAR_HEIGHT: f64 = 88.0;

#[derive(Debug)]
pub enum UserBrowserEvent {
    CreateTab(String),
    ToggleAutoControl,
    TriggerAutoAction,
    TypeChar(char),
    ClearAddressbar,
}

fn get_macos_clipboard() -> String {
    let output = Command::new("pbpaste").output();
    if let Ok(out) = output {
        String::from_utf8_lossy(&out.stdout).to_string()
    } else {
        String::new()
    }
}

fn set_macos_clipboard(text: &str) {
    if let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

// Global window rect storage for auto mouse movement bounds
#[derive(Clone, Copy, Debug)]
pub struct AppWindowRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

fn build_toolbar_html(initial_tab_id: u32, initial_url: &str) -> String {
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
      background: rgba(255, 255, 255, 0.12);
      padding: 1px 5px;
      border-radius: 4px;
      font-size: 10px;
      font-family: monospace;
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
        <span class="key-badge">F8</span>
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
      if (enabled) {{
        btn.classList.add("active");
        btnText.innerText = "Auto Control: ON";
      }} else {{
        btn.classList.remove("active");
        btnText.innerText = "Auto Control: OFF";
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

    window.clearAddressbar = function() {{
      input.value = "";
      input.focus();
    }};

    window.appendChar = function(ch) {{
      input.value += ch;
      input.focus();
      // Ensure cursor stays at end without selecting or submitting
      input.selectionStart = input.selectionEnd = input.value.length;
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

struct TabState {
    id: u32,
    title: String,
    url: String,
    webview: WebView,
}

fn sync_tabs_to_toolbar(
    map: &HashMap<u32, TabState>,
    active_id: u32,
    toolbar_view_for_ipc: &Arc<Mutex<Option<WebView>>>,
) {
    #[derive(serde::Serialize)]
    struct TabInfo<'a> {
        id: u32,
        title: &'a str,
        url: &'a str,
    }

    let mut list: Vec<TabInfo> = map
        .values()
        .map(|t| TabInfo {
            id: t.id,
            title: &t.title,
            url: &t.url,
        })
        .collect();
    list.sort_by_key(|t| t.id);

    let active_url = map.get(&active_id).map(|t| t.url.as_str()).unwrap_or("");

    if let Ok(json_tabs) = serde_json::to_string(&list) {
        if let Ok(opt_tv) = toolbar_view_for_ipc.lock() {
            if let Some(tv) = opt_tv.as_ref() {
                let script = format!(
                    "window.renderTabs({}, {}, {});",
                    json_tabs,
                    active_id,
                    serde_json::to_string(active_url).unwrap_or_default()
                );
                let _ = tv.evaluate_script(&script);
            }
        }
    }
}

// Injected helper script into web pages for safe text selection and clicking empty space
const AUTO_INJECT_SCRIPT: &str = r#"
window.__autoRandomAction = function() {
    try {
        const rand = Math.random();
        if (rand < 0.6) {
            // Select random text on the page
            const textNodes = [];
            const walker = document.createTreeWalker(document.body || document.documentElement, NodeFilter.SHOW_TEXT);
            let n;
            while ((n = walker.nextNode())) {
                const txt = n.textContent.trim();
                if (txt.length > 5) {
                    textNodes.push(n);
                    if (textNodes.length > 100) break;
                }
            }
            if (textNodes.length > 0) {
                const targetNode = textNodes[Math.floor(Math.random() * textNodes.length)];
                const range = document.createRange();
                const textLen = targetNode.textContent.length;
                const start = Math.floor(Math.random() * Math.max(1, textLen - 4));
                const end = Math.min(textLen, start + Math.floor(Math.random() * 12) + 2);
                range.setStart(targetNode, start);
                range.setEnd(targetNode, end);
                const sel = window.getSelection();
                sel.removeAllRanges();
                sel.addRange(range);
            }
        } else {
            // Clear selection and click empty background / container
            const sel = window.getSelection();
            if (sel) sel.removeAllRanges();
            const bg = document.body || document.documentElement;
            if (bg) {
                const evt = new MouseEvent('click', { bubbles: true, cancelable: true, view: window });
                bg.dispatchEvent(evt);
            }
        }
    } catch(e) {}
};
"#;

// Get actual current mouse location on screen
fn get_current_mouse_position() -> CGPoint {
    unsafe {
        let ev = CGEventCreate(std::ptr::null());
        if !ev.is_null() {
            let loc = CGEventGetLocation(ev);
            CFRelease(ev);
            return loc;
        }
    }
    CGPoint::new(400.0, 400.0)
}

// Move mouse like a real human: Bézier curvature, speed variations, and slight overshoot
fn human_like_move_mouse(
    start: CGPoint,
    end: CGPoint,
    auto_active: &Arc<AtomicBool>,
) {
    let mut rng = rand::thread_rng();
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < 2.0 {
        return;
    }

    // Determine movement style randomly:
    // 0: Fast flick, 1: Smooth leisurely glide, 2: Wandering/hesitant curve
    let style = rng.gen_range(0..3);

    // Number of intermediate steps based on distance & style
    let steps = match style {
        0 => (dist / 14.0).clamp(18.0, 45.0) as usize, // fast move
        1 => (dist / 7.0).clamp(35.0, 90.0) as usize,  // smooth, steady move
        _ => (dist / 5.0).clamp(50.0, 130.0) as usize, // relaxed human exploration
    };

    // Calculate perpendicular offset for realistic natural hand curvature
    let perp_x = -dy / dist;
    let perp_y = dx / dist;

    // Random curve intensity (sometimes almost straight, sometimes prominently curved)
    let curve_mag = if rng.gen_bool(0.25) {
        // Nearly straight line with subtle micro-deviations
        rng.gen_range(-15.0..15.0)
    } else {
        // Curvy arc
        rng.gen_range(-1.0..1.0) * dist.min(180.0) * 0.45
    };

    // Control point 1 (around 25% - 40% of the path)
    let cp1_t = rng.gen_range(0.25..0.45);
    let cp1_x = start.x + dx * cp1_t + perp_x * curve_mag + rng.gen_range(-15.0..15.0);
    let cp1_y = start.y + dy * cp1_t + perp_y * curve_mag + rng.gen_range(-15.0..15.0);

    // Control point 2 (around 60% - 80% of the path, with natural counter-balance or continuation)
    let cp2_t = rng.gen_range(0.60..0.85);
    let counter_curv = if rng.gen_bool(0.4) { -curve_mag * 0.5 } else { curve_mag * 0.7 };
    let cp2_x = start.x + dx * cp2_t + perp_x * counter_curv + rng.gen_range(-15.0..15.0);
    let cp2_y = start.y + dy * cp2_t + perp_y * counter_curv + rng.gen_range(-15.0..15.0);

    let base_sleep_ms = match style {
        0 => rng.gen_range(5..10),  // faster updates
        1 => rng.gen_range(10..18), // standard smooth updates
        _ => rng.gen_range(14..24), // slow, deliberate movement
    };

    if let Ok(source) = CGEventSource::new(CGEventSourceStateID::CombinedSessionState) {
        for i in 1..=steps {
            if !auto_active.load(Ordering::SeqCst) {
                break;
            }

            let t = i as f64 / steps as f64;

            // Human acceleration curve: slow start, quick sweep, decelerate at destination
            let eased_t = if style == 0 {
                // Quick start then deceleration
                t * (2.0 - t)
            } else {
                // Smooth bell-shaped acceleration (SmoothStep / Sigmoidal)
                t * t * (3.0 - 2.0 * t)
            };

            // Cubic Bézier calculation: B(t) = (1-t)^3*P0 + 3(1-t)^2*t*P1 + 3(1-t)*t^2*P2 + t^3*P3
            let u = 1.0 - eased_t;
            let tt = eased_t * eased_t;
            let uu = u * u;
            let uuu = uu * u;
            let ttt = tt * eased_t;

            let cur_x = uuu * start.x
                + 3.0 * uu * eased_t * cp1_x
                + 3.0 * u * tt * cp2_x
                + ttt * end.x;

            let cur_y = uuu * start.y
                + 3.0 * uu * eased_t * cp1_y
                + 3.0 * u * tt * cp2_y
                + ttt * end.y;

            // Add subtle micro-jitter like a real human hand
            let jitter_x = if i < steps - 3 { rng.gen_range(-0.7..0.7) } else { 0.0 };
            let jitter_y = if i < steps - 3 { rng.gen_range(-0.7..0.7) } else { 0.0 };

            let pt = CGPoint::new(cur_x + jitter_x, cur_y + jitter_y);

            unsafe {
                let _ = CGWarpMouseCursorPosition(pt);
            }

            if let Ok(move_ev) = CGEvent::new_mouse_event(
                source.clone(),
                CGEventType::MouseMoved,
                pt,
                CGMouseButton::Left,
            ) {
                move_ev.post(CGEventTapLocation::HID);
            }

            // Variable sleep for non-robotic timing
            let jitter_sleep = rng.gen_range(0..4);
            thread::sleep(Duration::from_millis((base_sleep_ms + jitter_sleep).max(2)));
        }

        // Ensure final exact target position
        unsafe {
            let _ = CGWarpMouseCursorPosition(end);
        }
    }
}

// Simulate mouse click at given screen coordinates
fn click_mouse_at(point: CGPoint) {
    if let Ok(source) = CGEventSource::new(CGEventSourceStateID::CombinedSessionState) {
        if let Ok(down) = CGEvent::new_mouse_event(
            source.clone(),
            CGEventType::LeftMouseDown,
            point,
            CGMouseButton::Left,
        ) {
            down.post(CGEventTapLocation::HID);
        }
        thread::sleep(Duration::from_millis(40));
        if let Ok(up) = CGEvent::new_mouse_event(
            source,
            CGEventType::LeftMouseUp,
            point,
            CGMouseButton::Left,
        ) {
            up.post(CGEventTapLocation::HID);
        }
    }
}

// Simulate scroll wheel event using macOS CoreGraphics API
fn scroll_mouse(delta_y: i32) {
    unsafe {
        let scroll_ev = CGEventCreateScrollWheelEvent2(
            std::ptr::null(),
            0, // kCGScrollEventUnitPixel = 0, or kCGScrollEventUnitLine = 1
            1,
            delta_y,
            0,
            0,
        );
        if !scroll_ev.is_null() {
            CGEventPost(0 /* kCGHIDEventTap */, scroll_ev);
            CFRelease(scroll_ev);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("==================================================");
    println!("Application Name : {}", APP_NAME);
    println!("Window Title     : {}", WINDOW_TITLE);
    println!("Application Vers : {}", APP_VERSION);
    println!("Extended Info    : {}", EXTENDED_INFO);
    println!("Auto Control     : F8 Shortcut & Top-Right Button ENABLED");
    println!("Engine           : Direct Native Chromium/WebKit Engine");
    println!("==================================================");

    let event_loop: EventLoop<UserBrowserEvent> = EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let initial_width = 1280.0;
    let initial_height = 840.0;

    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_inner_size(LogicalSize::new(initial_width, initial_height))
        .with_min_inner_size(LogicalSize::new(700.0, 500.0))
        .build(&event_loop)?;

    let initial_url = "https://www.google.com";
    let first_tab_id = 1u32;

    // Content bounds for tab views
    let content_bounds = Rect {
        position: LogicalPosition::new(0.0, TOOLBAR_HEIGHT).into(),
        size: LogicalSize::new(initial_width, initial_height - TOOLBAR_HEIGHT).into(),
    };

    // Create the first tab webview
    let first_webview = WebViewBuilder::new()
        .with_user_agent(CHROME_USER_AGENT)
        .with_bounds(content_bounds)
        .with_url(initial_url)
        .with_initialization_script(AUTO_INJECT_SCRIPT)
        .with_back_forward_navigation_gestures(true)
        .with_devtools(true)
        .with_clipboard(true)
        .with_visible(true)
        .build_as_child(&window)?;

    let tabs: Arc<Mutex<HashMap<u32, TabState>>> = Arc::new(Mutex::new(HashMap::new()));
    let active_tab_id: Arc<Mutex<u32>> = Arc::new(Mutex::new(first_tab_id));
    let next_tab_id: Arc<Mutex<u32>> = Arc::new(Mutex::new(2u32));

    tabs.lock().unwrap().insert(
        first_tab_id,
        TabState {
            id: first_tab_id,
            title: "Google".to_string(),
            url: initial_url.to_string(),
            webview: first_webview,
        },
    );

    // Toolbar webview on top
    let toolbar_bounds = Rect {
        position: LogicalPosition::new(0.0, 0.0).into(),
        size: LogicalSize::new(initial_width, TOOLBAR_HEIGHT).into(),
    };

    let toolbar_html = build_toolbar_html(first_tab_id, initial_url);

    let toolbar_view_cell: Arc<Mutex<Option<WebView>>> = Arc::new(Mutex::new(None));
    let toolbar_view_for_ipc = toolbar_view_cell.clone();

    let tabs_for_ipc = tabs.clone();
    let active_id_for_ipc = active_tab_id.clone();
    let proxy_for_ipc = proxy.clone();

    // Auto Control State flags
    let auto_control_active = Arc::new(AtomicBool::new(false));

    // Window position & size shared for mouse bounds
    let initial_pos = window.outer_position().unwrap_or(tao::dpi::PhysicalPosition::new(100, 100));
    let initial_size = window.inner_size();
    let app_window_rect = Arc::new(Mutex::new(AppWindowRect {
        x: initial_pos.x as f64,
        y: initial_pos.y as f64,
        width: initial_size.width as f64,
        height: initial_size.height as f64,
    }));

    let toolbar_view = WebViewBuilder::new()
        .with_bounds(toolbar_bounds)
        .with_html(toolbar_html)
        .with_clipboard(true)
        .with_ipc_handler(move |msg| {
            let body = msg.body();
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
                let action = val.get("action").and_then(|a| a.as_str()).unwrap_or("");
                let payload = val.get("payload").and_then(|p| p.as_str()).unwrap_or("");

                match action {
                    "navigate" => {
                        let cur_id = *active_id_for_ipc.lock().unwrap();
                        let mut target = payload.trim().to_string();
                        if !target.is_empty() {
                            if !target.starts_with("http://") && !target.starts_with("https://") {
                                if target.contains('.') && !target.contains(' ') {
                                    target = format!("https://{}", target);
                                } else {
                                    target = format!("https://www.google.com/search?q={}", target);
                                }
                            }
                            println!("[Direct Navigation] Loading: {}", target);
                            let mut map = tabs_for_ipc.lock().unwrap();
                            if let Some(tab) = map.get_mut(&cur_id) {
                                tab.url = target.clone();
                                let _ = tab.webview.load_url(&target);
                            }
                        }
                    }
                    "back" => {
                        let cur_id = *active_id_for_ipc.lock().unwrap();
                        let map = tabs_for_ipc.lock().unwrap();
                        if let Some(tab) = map.get(&cur_id) {
                            let _ = tab.webview.evaluate_script("window.history.back();");
                        }
                    }
                    "forward" => {
                        let cur_id = *active_id_for_ipc.lock().unwrap();
                        let map = tabs_for_ipc.lock().unwrap();
                        if let Some(tab) = map.get(&cur_id) {
                            let _ = tab.webview.evaluate_script("window.history.forward();");
                        }
                    }
                    "reload" => {
                        let cur_id = *active_id_for_ipc.lock().unwrap();
                        let map = tabs_for_ipc.lock().unwrap();
                        if let Some(tab) = map.get(&cur_id) {
                            let _ = tab.webview.evaluate_script("window.location.reload();");
                        }
                    }
                    "create_tab" => {
                        let target_url = if payload.is_empty() {
                            "https://www.google.com".to_string()
                        } else {
                            payload.to_string()
                        };
                        let _ = proxy_for_ipc.send_event(UserBrowserEvent::CreateTab(target_url));
                    }
                    "switch_tab" => {
                        if let Ok(id) = payload.parse::<u32>() {
                            let map = tabs_for_ipc.lock().unwrap();
                            let old_id = *active_id_for_ipc.lock().unwrap();
                            if old_id != id {
                                if let Some(old_tab) = map.get(&old_id) {
                                    let _ = old_tab.webview.set_visible(false);
                                }
                                if let Some(new_tab) = map.get(&id) {
                                    let _ = new_tab.webview.set_visible(true);
                                    let _ = new_tab.webview.focus();
                                    *active_id_for_ipc.lock().unwrap() = id;
                                    sync_tabs_to_toolbar(&map, id, &toolbar_view_for_ipc);
                                }
                            }
                        }
                    }
                    "close_tab" => {
                        if let Ok(id) = payload.parse::<u32>() {
                            let mut map = tabs_for_ipc.lock().unwrap();
                            if map.len() <= 1 {
                                // If last tab, just reset to google
                                if let Some(tab) = map.get_mut(&id) {
                                    tab.url = "https://www.google.com".to_string();
                                    tab.title = "Google".to_string();
                                    let _ = tab.webview.load_url("https://www.google.com");
                                    sync_tabs_to_toolbar(&map, id, &toolbar_view_for_ipc);
                                }
                                return;
                            }

                            if let Some(removed) = map.remove(&id) {
                                let _ = removed.webview.set_visible(false);
                            }

                            let cur_active = *active_id_for_ipc.lock().unwrap();
                            if cur_active == id {
                                // Select first remaining tab
                                if let Some(&next_active) = map.keys().next() {
                                    *active_id_for_ipc.lock().unwrap() = next_active;
                                    if let Some(tab) = map.get(&next_active) {
                                        let _ = tab.webview.set_visible(true);
                                        let _ = tab.webview.focus();
                                    }
                                    sync_tabs_to_toolbar(&map, next_active, &toolbar_view_for_ipc);
                                }
                            } else {
                                sync_tabs_to_toolbar(&map, cur_active, &toolbar_view_for_ipc);
                            }
                        }
                    }
                    "toggle_auto_control" => {
                        let _ = proxy_for_ipc.send_event(UserBrowserEvent::ToggleAutoControl);
                    }
                    "request_paste" => {
                        let clip = get_macos_clipboard();
                        if let Ok(opt_tv) = toolbar_view_for_ipc.lock() {
                            if let Some(tv) = opt_tv.as_ref() {
                                let json_str = serde_json::to_string(&clip).unwrap_or_default();
                                let script = format!("window.insertPaste({});", json_str);
                                let _ = tv.evaluate_script(&script);
                            }
                        }
                    }
                    "set_clipboard" => {
                        set_macos_clipboard(payload);
                    }
                    _ => {}
                }
            }
        })
        .build_as_child(&window)?;

    // Store toolbar_view in cell for IPC callbacks
    if let Ok(mut cell) = toolbar_view_cell.lock() {
        *cell = Some(toolbar_view);
    }

    // Permanently lock the native OS window title
    window.set_title(WINDOW_TITLE);

    // Spawn Background Auto Controller Thread
    {
        let auto_active = auto_control_active.clone();
        let win_rect = app_window_rect.clone();
        let proxy_worker = proxy.clone();

        thread::spawn(move || {
            let mut rng = rand::thread_rng();

            loop {
                thread::sleep(Duration::from_millis(500));

                if !auto_active.load(Ordering::SeqCst) {
                    continue;
                }

                // Get current window bounds
                let rect = if let Ok(r) = win_rect.lock() {
                    *r
                } else {
                    continue;
                };

                // Browser content area on screen
                let min_x = rect.x + 35.0;
                let max_x = (rect.x + rect.width - 35.0).max(min_x + 50.0);
                let min_y = rect.y + TOOLBAR_HEIGHT + 35.0;
                let max_y = (rect.y + rect.height - 35.0).max(min_y + 50.0);

                // 1. Get ACTUAL current mouse position on the screen
                let current_mouse_pt = get_current_mouse_position();

                // 2. Select action type:
                // Either Human Mouse Motion + Page Interaction OR Natural Keyboard Typing into Addressbar
                // These are strictly synchronous and never execute at the same time.
                let mode_choice = rng.gen_range(0..10);

                if mode_choice < 7 {
                    // MODE A: Human Mouse Movement & Page Interactions
                    // 1. Move real mouse to random position within browser web content area
                    let target_x = rng.gen_range(min_x..max_x);
                    let target_y = rng.gen_range(min_y..max_y);
                    let target_pt = CGPoint::new(target_x, target_y);

                    // Move mouse seamlessly from its current real position along a human-like Bézier curve
                    human_like_move_mouse(current_mouse_pt, target_pt, &auto_active);

                    if !auto_active.load(Ordering::SeqCst) {
                        continue;
                    }

                    let action_choice = rng.gen_range(0..10);
                    if action_choice < 4 {
                        // Smooth human-like scrolling (multiple small ticks)
                        let total_ticks = rng.gen_range(3..12);
                        let scroll_dir = if rng.gen_bool(0.75) { -1 } else { 1 };
                        for _ in 0..total_ticks {
                            if !auto_active.load(Ordering::SeqCst) {
                                break;
                            }
                            scroll_mouse(scroll_dir * rng.gen_range(1..3));
                            thread::sleep(Duration::from_millis(rng.gen_range(35..90)));
                        }
                    } else if action_choice < 8 {
                        // Select random text on the webpage
                        let _ = proxy_worker.send_event(UserBrowserEvent::TriggerAutoAction);
                    } else {
                        // Click an empty space on the webpage
                        click_mouse_at(target_pt);
                    }
                } else {
                    // MODE B: Synchronous Natural Human Keyboard Typing in Address Bar (a-z, 0-9)
                    // Real dictionary words / search queries
                    const DICTIONARY_WORDS: &[&str] = &[
                        "weather today", "github trending", "rust programming", "coolify dashboard",
                        "news updates", "crypto market 2026", "travel destinations", "top movies",
                        "tech gadgets", "hrm workspace", "chatgpt 5", "ai developments",
                        "recipe ideas", "world clock", "sports scores", "flight tickets",
                        "online shop 24", "system monitor", "finance tracker", "developer tools",
                        "fast network", "cloud storage", "smart search", "browser speed",
                        "music playlist 99", "best laptop 2026", "coffee shops near me", "code refactor"
                    ];

                    let word = DICTIONARY_WORDS[rng.gen_range(0..DICTIONARY_WORDS.len())];
                    println!("[Auto Control] Synchronous Human Typing query: \"{}\"", word);

                    // Clear address bar first for clean typing
                    let _ = proxy_worker.send_event(UserBrowserEvent::ClearAddressbar);
                    thread::sleep(Duration::from_millis(rng.gen_range(250..450)));

                    // Type character by character at natural human typing cadence (80ms - 240ms)
                    for ch in word.chars() {
                        if !auto_active.load(Ordering::SeqCst) {
                            break;
                        }

                        // Send character to address bar without pressing Enter
                        let _ = proxy_worker.send_event(UserBrowserEvent::TypeChar(ch));

                        // Realistic human typing cadence with occasional pause between words
                        let key_delay = if ch == ' ' {
                            rng.gen_range(160..320)
                        } else {
                            rng.gen_range(75..195)
                        };
                        thread::sleep(Duration::from_millis(key_delay));
                    }
                }

                // 3. User requested random interval of 5 to 60 seconds between actions
                let sleep_secs = rng.gen_range(5..=60);
                println!(
                    "[Auto Control] Action finished. Next random human action in {}s (range: 5-60s)",
                    sleep_secs
                );

                // Sleep in 200ms increments so toggling OFF (F8) responds immediately without waiting up to 60s
                for _ in 0..(sleep_secs * 5) {
                    if !auto_active.load(Ordering::SeqCst) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(200));
                }
            }
        });
    }

    let tabs_for_loop = tabs.clone();
    let toolbar_for_loop = toolbar_view_cell.clone();
    let active_id_for_loop = active_tab_id.clone();
    let next_id_for_loop = next_tab_id.clone();
    let rect_for_loop = app_window_rect.clone();
    let auto_control_for_loop = auto_control_active.clone();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserBrowserEvent::TypeChar(ch)) => {
                if let Ok(opt_tv) = toolbar_for_loop.lock() {
                    if let Some(tv) = opt_tv.as_ref() {
                        let json_char = serde_json::to_string(&ch.to_string()).unwrap_or_default();
                        let script = format!("if (window.appendChar) window.appendChar({});", json_char);
                        let _ = tv.evaluate_script(&script);
                    }
                }
            }
            Event::UserEvent(UserBrowserEvent::ClearAddressbar) => {
                if let Ok(opt_tv) = toolbar_for_loop.lock() {
                    if let Some(tv) = opt_tv.as_ref() {
                        let _ = tv.evaluate_script("if (window.clearAddressbar) window.clearAddressbar();");
                    }
                }
            }
            Event::UserEvent(UserBrowserEvent::TriggerAutoAction) => {
                let cur_id = *active_id_for_loop.lock().unwrap();
                if let Ok(map) = tabs_for_loop.lock() {
                    if let Some(tab) = map.get(&cur_id) {
                        let _ = tab.webview.evaluate_script("if (window.__autoRandomAction) window.__autoRandomAction();");
                    }
                }
            }
            Event::UserEvent(UserBrowserEvent::ToggleAutoControl) => {
                let current = auto_control_for_loop.load(Ordering::SeqCst);
                let new_state = !current;
                auto_control_for_loop.store(new_state, Ordering::SeqCst);
                println!(
                    "[Auto Control] Toggled: {}",
                    if new_state { "ACTIVE (Moving mouse & browsing)" } else { "STOPPED" }
                );

                if let Ok(opt_tv) = toolbar_for_loop.lock() {
                    if let Some(tv) = opt_tv.as_ref() {
                        let script = format!("window.setAutoControlState({});", new_state);
                        let _ = tv.evaluate_script(&script);
                    }
                }
            }
            Event::UserEvent(UserBrowserEvent::CreateTab(target_url)) => {
                let size = window.inner_size();
                let scale = window.scale_factor();
                let logical_w = size.width as f64 / scale;
                let logical_h = size.height as f64 / scale;
                let content_h = (logical_h - TOOLBAR_HEIGHT).max(10.0);

                let cur_bounds = Rect {
                    position: LogicalPosition::new(0.0, TOOLBAR_HEIGHT).into(),
                    size: LogicalSize::new(logical_w, content_h).into(),
                };

                let mut n_lock = next_id_for_loop.lock().unwrap();
                let new_id = *n_lock;
                *n_lock += 1;

                if let Ok(new_wv) = WebViewBuilder::new()
                    .with_user_agent(CHROME_USER_AGENT)
                    .with_bounds(cur_bounds)
                    .with_url(&target_url)
                    .with_initialization_script(AUTO_INJECT_SCRIPT)
                    .with_back_forward_navigation_gestures(true)
                    .with_devtools(true)
                    .with_clipboard(true)
                    .with_visible(true)
                    .build_as_child(&window)
                {
                    let mut map = tabs_for_loop.lock().unwrap();
                    let old_id = *active_id_for_loop.lock().unwrap();
                    if let Some(old_tab) = map.get(&old_id) {
                        let _ = old_tab.webview.set_visible(false);
                    }

                    *active_id_for_loop.lock().unwrap() = new_id;

                    map.insert(
                        new_id,
                        TabState {
                            id: new_id,
                            title: "New Tab".to_string(),
                            url: target_url,
                            webview: new_wv,
                        },
                    );

                    sync_tabs_to_toolbar(&map, new_id, &toolbar_for_loop);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event: key_event, .. },
                ..
            } => {
                // Global F8 toggle shortcut detection
                if key_event.state == ElementState::Pressed && key_event.physical_key == KeyCode::F8 {
                    let current = auto_control_for_loop.load(Ordering::SeqCst);
                    let new_state = !current;
                    auto_control_for_loop.store(new_state, Ordering::SeqCst);
                    println!(
                        "[Auto Control - F8 Shortcut] Toggled: {}",
                        if new_state { "ACTIVE" } else { "STOPPED" }
                    );

                    if let Ok(opt_tv) = toolbar_for_loop.lock() {
                        if let Some(tv) = opt_tv.as_ref() {
                            let script = format!("window.setAutoControlState({});", new_state);
                            let _ = tv.evaluate_script(&script);
                        }
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                auto_control_for_loop.store(false, Ordering::SeqCst);
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Moved(new_pos),
                ..
            } => {
                if let Ok(mut r) = rect_for_loop.lock() {
                    r.x = new_pos.x as f64;
                    r.y = new_pos.y as f64;
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                let scale = window.scale_factor();
                let logical_w = new_size.width as f64 / scale;
                let logical_h = new_size.height as f64 / scale;

                if let Ok(mut r) = rect_for_loop.lock() {
                    r.width = new_size.width as f64;
                    r.height = new_size.height as f64;
                }

                // Resize toolbar
                if let Ok(opt_tv) = toolbar_for_loop.lock() {
                    if let Some(tv) = opt_tv.as_ref() {
                        let _ = tv.set_bounds(Rect {
                            position: LogicalPosition::new(0.0, 0.0).into(),
                            size: LogicalSize::new(logical_w, TOOLBAR_HEIGHT).into(),
                        });
                    }
                }

                // Resize all tab webviews
                let content_h = (logical_h - TOOLBAR_HEIGHT).max(10.0);
                let new_bounds = Rect {
                    position: LogicalPosition::new(0.0, TOOLBAR_HEIGHT).into(),
                    size: LogicalSize::new(logical_w, content_h).into(),
                };

                if let Ok(map) = tabs_for_loop.lock() {
                    for tab in map.values() {
                        let _ = tab.webview.set_bounds(new_bounds);
                    }
                }

                window.set_title(WINDOW_TITLE);
            }
            Event::WindowEvent {
                event: WindowEvent::Focused(true) | WindowEvent::ReceivedImeText(_),
                ..
            } => {
                // Ensure window title is strictly locked at all times
                window.set_title(WINDOW_TITLE);
            }
            Event::MainEventsCleared => {
                window.set_title(WINDOW_TITLE);
            }
            _ => (),
        }
    });
}
