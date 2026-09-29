use std::io::Write;
use std::process::{Command, Stdio};
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::{Rect, WebViewBuilder};

// 4 Permanent & Static Metadata Values
pub const APP_NAME: &str = "Brave";
pub const WINDOW_TITLE: &str = "Bluevy Admin - Brave";
pub const APP_VERSION: &str = "8037.58";
pub const EXTENDED_INFO: &str = "http://localhost:3003/workspace/DataList?id=9b0f6bcb-c5d1-4c86-a4";

// Real Google Chrome User Agent for direct native browsing
const CHROME_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";

const TOOLBAR_HEIGHT: f64 = 88.0;

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

fn build_toolbar_html(current_url: &str) -> String {
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
      background: #202124;
      color: #e8eaed;
      height: 88px;
      overflow: hidden;
      display: flex;
      flex-direction: column;
      border-bottom: 1px solid #3c4043;
      user-select: none;
      -webkit-user-select: none;
    }}

    /* Top Static System Banner */
    .system-strip {{
      height: 32px;
      background: linear-gradient(90deg, #16181d 0%, #1f232b 100%);
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 14px;
      font-size: 11.5px;
      color: #9aa0a6;
      border-bottom: 1px solid #2d3139;
      flex-shrink: 0;
    }}
    .badge {{
      background: rgba(66, 133, 244, 0.22);
      color: #8ab4f8;
      padding: 2px 8px;
      border-radius: 4px;
      font-weight: 600;
      border: 1px solid rgba(66, 133, 244, 0.35);
    }}
    .version-badge {{
      background: #1a73e8;
      color: #ffffff;
      padding: 2px 8px;
      border-radius: 4px;
      font-weight: 700;
      font-size: 11.5px;
      letter-spacing: 0.3px;
      box-shadow: 0 1px 3px rgba(0,0,0,0.3);
    }}
    .stat-val {{
      color: #f1f3f4;
      font-weight: 600;
    }}
    .ext-info {{
      color: #8ab4f8;
      max-width: 480px;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      cursor: pointer;
      font-family: monospace;
      font-size: 11px;
    }}
    .ext-info:hover {{
      text-decoration: underline;
    }}

    /* Navigation Bar */
    .nav-bar {{
      height: 56px;
      display: flex;
      align-items: center;
      padding: 0 10px;
      gap: 8px;
      background: #2b2a33;
      flex-shrink: 0;
    }}
    .nav-btn {{
      width: 34px;
      height: 34px;
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
      width: 18px;
      height: 18px;
      fill: currentColor;
    }}
    .url-box-wrapper {{
      flex: 1;
      height: 38px;
      background: #1e1f22;
      border-radius: 19px;
      display: flex;
      align-items: center;
      padding: 0 12px;
      border: 1px solid #3c4043;
      transition: border-color 0.15s, box-shadow 0.15s;
    }}
    .url-box-wrapper:focus-within {{
      border-color: #4285f4;
      box-shadow: 0 0 0 2px rgba(66, 133, 244, 0.25);
    }}
    .lock-icon {{
      font-size: 13px;
      margin-right: 8px;
      color: #8ab4f8;
      user-select: none;
    }}
    .url-input {{
      flex: 1;
      height: 100%;
      background: transparent;
      border: none;
      outline: none;
      color: #ffffff;
      font-size: 13.5px;
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
    .action-pill {{
      background: #1a73e8;
      color: #ffffff;
      border: none;
      padding: 0 16px;
      height: 34px;
      border-radius: 17px;
      font-size: 12px;
      cursor: pointer;
      font-weight: 600;
      transition: background 0.15s;
      flex-shrink: 0;
    }}
    .action-pill:hover {{
      background: #1b66c9;
    }}
  </style>
</head>
<body>
  <div class="system-strip">
    <div style="display: flex; align-items: center; gap: 12px;">
      <span class="badge">Google Chrome</span>
      <span>Title: <span class="stat-val">{title}</span></span>
      <span>Application Vers: <span class="version-badge">{version}</span></span>
    </div>
    <div style="display: flex; align-items: center; gap: 8px;">
      <span>Extended Info:</span>
      <span class="ext-info" onclick="sendAction('navigate', '{extended}')" title="{extended}">{extended}</span>
      <button class="mini-btn" onclick="sendAction('navigate', '{extended}')">Load</button>
    </div>
  </div>

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
    <button class="nav-btn" title="Home (Google)" onclick="sendAction('navigate', 'https://www.google.com')">
      <svg viewBox="0 0 24 24"><path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/></svg>
    </button>

    <div class="url-box-wrapper">
      <span class="lock-icon">🔒</span>
      <input type="text" class="url-input" id="address" value="{url}" placeholder="Search Google or enter URL" spellcheck="false" autocomplete="off" />
      <div class="input-actions">
        <button class="mini-btn" title="Paste clipboard URL" onclick="handlePasteAction()">Paste</button>
        <button class="mini-btn" title="Copy current URL" onclick="handleCopyAction()">Copy</button>
      </div>
    </div>
    <button class="action-pill" onclick="submitUrl()">Go</button>
  </div>

  <script>
    const input = document.getElementById("address");

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

    // Support keyboard shortcuts Cmd+V, Cmd+C, Cmd+A, Cmd+X directly
    input.addEventListener("keydown", function(e) {{
      if (e.key === "Enter") {{
        submitUrl();
        return;
      }}

      if (e.metaKey || e.ctrlKey) {{
        if (e.key.toLowerCase() === "v") {{
          // Request paste from native macOS clipboard
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

    // Update url input box if URL changed
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
  </script>
</body>
</html>"#,
        title = WINDOW_TITLE,
        version = APP_VERSION,
        extended = EXTENDED_INFO,
        url = current_url
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("==================================================");
    println!("Application Name : {}", APP_NAME);
    println!("Window Title     : {}", WINDOW_TITLE);
    println!("Application Vers : {}", APP_VERSION);
    println!("Extended Info    : {}", EXTENDED_INFO);
    println!("Engine           : Direct Native Chromium/WebKit Engine");
    println!("Direct Browsing  : ENABLED (No iframes, No proxy)");
    println!("==================================================");

    let event_loop = EventLoop::new();

    let initial_width = 1280.0;
    let initial_height = 840.0;

    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_inner_size(LogicalSize::new(initial_width, initial_height))
        .with_min_inner_size(LogicalSize::new(700.0, 500.0))
        .build(&event_loop)?;

    let initial_url = "https://www.google.com";

    // 1. Create Native Web Content View (Loads real web pages directly, no iframes!)
    let content_bounds = Rect {
        position: LogicalPosition::new(0.0, TOOLBAR_HEIGHT).into(),
        size: LogicalSize::new(initial_width, initial_height - TOOLBAR_HEIGHT).into(),
    };

    let content_view = WebViewBuilder::new()
        .with_user_agent(CHROME_USER_AGENT)
        .with_bounds(content_bounds)
        .with_url(initial_url)
        .with_back_forward_navigation_gestures(true)
        .with_devtools(true)
        .with_clipboard(true)
        .with_document_title_changed_handler(|_| {
            // Window title is locked on window object and event loop
        })
        .build_as_child(&window)?;

    // 2. Create Native Chrome Toolbar Webview as a child at the top
    let toolbar_bounds = Rect {
        position: LogicalPosition::new(0.0, 0.0).into(),
        size: LogicalSize::new(initial_width, TOOLBAR_HEIGHT).into(),
    };

    let toolbar_html = build_toolbar_html(initial_url);

    let content_view_proxy = std::sync::Arc::new(std::sync::Mutex::new(content_view));
    let content_view_for_ipc = content_view_proxy.clone();

    let toolbar_view_cell: std::sync::Arc<std::sync::Mutex<Option<wry::WebView>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));
    let toolbar_view_for_ipc = toolbar_view_cell.clone();

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
                            if let Ok(cv) = content_view_for_ipc.lock() {
                                let _ = cv.load_url(&target);
                            }
                        }
                    }
                    "back" => {
                        if let Ok(cv) = content_view_for_ipc.lock() {
                            let _ = cv.evaluate_script("window.history.back();");
                        }
                    }
                    "forward" => {
                        if let Ok(cv) = content_view_for_ipc.lock() {
                            let _ = cv.evaluate_script("window.history.forward();");
                        }
                    }
                    "reload" => {
                        if let Ok(cv) = content_view_for_ipc.lock() {
                            let _ = cv.evaluate_script("window.location.reload();");
                        }
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

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                let scale = window.scale_factor();
                let logical_w = new_size.width as f64 / scale;
                let logical_h = new_size.height as f64 / scale;

                // Resize toolbar
                if let Ok(opt_tv) = toolbar_view_cell.lock() {
                    if let Some(tv) = opt_tv.as_ref() {
                        let _ = tv.set_bounds(Rect {
                            position: LogicalPosition::new(0.0, 0.0).into(),
                            size: LogicalSize::new(logical_w, TOOLBAR_HEIGHT).into(),
                        });
                    }
                }

                // Resize content view
                if let Ok(cv) = content_view_proxy.lock() {
                    let content_h = (logical_h - TOOLBAR_HEIGHT).max(10.0);
                    let _ = cv.set_bounds(Rect {
                        position: LogicalPosition::new(0.0, TOOLBAR_HEIGHT).into(),
                        size: LogicalSize::new(logical_w, content_h).into(),
                    });
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
