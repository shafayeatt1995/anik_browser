use core_graphics::geometry::CGPoint;

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
pub const APP_NAME: &str = "Brave Browser";
pub const WINDOW_TITLE: &str = "Bluevy Admin - Brave Browser";
pub const APP_VERSION: &str = "195.104";
pub const EXTENDED_INFO: &str = "http://localhost:3001/workspace/DataList?id=9b0f6bcb-c5d1-4c86-a4";

mod toolbar;
use toolbar::build_toolbar_html;

mod mouse_control;
use mouse_control::{get_current_mouse_position, human_like_move_mouse, scroll_mouse};

// Real Brave Browser / Chrome User Agent for direct native browsing
const CHROME_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";

const TOOLBAR_HEIGHT: f64 = 88.0;

#[derive(Debug)]
pub enum UserBrowserEvent {
    CreateTab(String),
    ToggleAutoControl,
    UpdateCountdown(u32, String),
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

    // Window position & size shared for mouse bounds (in logical points matching macOS CoreGraphics mouse coordinates)
    let scale_factor = window.scale_factor();
    let initial_pos = window.outer_position().unwrap_or(tao::dpi::PhysicalPosition::new(100, 100));
    let initial_size = window.inner_size();
    let app_window_rect = Arc::new(Mutex::new(AppWindowRect {
        x: initial_pos.x as f64 / scale_factor,
        y: initial_pos.y as f64 / scale_factor,
        width: initial_size.width as f64 / scale_factor,
        height: initial_size.height as f64 / scale_factor,
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
            let mut next_action_choice = rng.gen_range(0..2);

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

                // Calculate browser webview viewport bounds strictly (safe inner screen area)
                // Margins: 60px away from sides and bottom, and safely below the top toolbar (88px + 60px)
                let min_x = rect.x + 60.0;
                let max_x = (rect.x + rect.width - 60.0).max(min_x + 50.0);
                let min_y = rect.y + TOOLBAR_HEIGHT + 60.0;
                let max_y = (rect.y + rect.height - 60.0).max(min_y + 50.0);

                // 1. Get ACTUAL current mouse position on the screen
                let current_mouse_pt = get_current_mouse_position();

                // 2. Execute scheduled action (only mouse move and scroll):
                let action_choice = next_action_choice;

                match action_choice {
                    0 => {
                        // Action 0: Mouse Movement strictly inside browser safe viewport
                        let target_x = rng.gen_range(min_x..max_x);
                        let target_y = rng.gen_range(min_y..max_y);
                        let target_pt = CGPoint::new(target_x, target_y);
                        println!("[Auto Control] Action: Mouse Move to ({:.0}, {:.0}) [safe viewport]", target_x, target_y);
                        human_like_move_mouse(current_mouse_pt, target_pt, &auto_active);
                    }
                    _ => {
                        // Action 1: Scroll Page up or down naturally
                        let scroll_dir = if rng.gen_bool(0.70) { -1 } else { 1 };
                        let total_ticks = rng.gen_range(4..14);
                        println!(
                            "[Auto Control] Action: Scroll Page {}",
                            if scroll_dir < 0 { "Down ⬇" } else { "Up ⬆" }
                        );
                        for _ in 0..total_ticks {
                            if !auto_active.load(Ordering::SeqCst) {
                                break;
                            }
                            scroll_mouse(scroll_dir * rng.gen_range(1..3));
                            thread::sleep(Duration::from_millis(rng.gen_range(30..80)));
                        }
                    }
                }

                if !auto_active.load(Ordering::SeqCst) {
                    continue;
                }

                // Random dynamic interval between 5 to 30 seconds generated fresh after every event/action
                let sleep_secs = rng.gen_range(5..=30);

                // Pick the upcoming action beforehand to display in terminal
                next_action_choice = rng.gen_range(0..2);
                let next_action_label = match next_action_choice {
                    0 => "mouse move",
                    _ => "scroll page",
                };

                println!(
                    "[Auto Control] Finished. Next action in {}s => {}",
                    sleep_secs, next_action_label
                );

                // Live countdown per second for UI badge display (e.g. 15s => mouse move)
                for remaining in (1..=sleep_secs).rev() {
                    if !auto_active.load(Ordering::SeqCst) {
                        break;
                    }
                    let _ = proxy_worker.send_event(UserBrowserEvent::UpdateCountdown(
                        remaining,
                        next_action_label.to_string(),
                    ));

                    // Sleep 1 second in 100ms slices for instant stop reaction
                    for _ in 0..10 {
                        if !auto_active.load(Ordering::SeqCst) {
                            break;
                        }
                        thread::sleep(Duration::from_millis(100));
                    }
                }

                // Reset badge on action execution
                if auto_active.load(Ordering::SeqCst) {
                    let _ = proxy_worker.send_event(UserBrowserEvent::UpdateCountdown(
                        0,
                        next_action_label.to_string(),
                    ));
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
            Event::UserEvent(UserBrowserEvent::UpdateCountdown(secs, action_name)) => {
                if let Ok(opt_tv) = toolbar_for_loop.lock() {
                    if let Some(tv) = opt_tv.as_ref() {
                        let json_action = serde_json::to_string(&action_name).unwrap_or_default();
                        let script = format!("if (window.updateAutoCountdown) window.updateAutoCountdown({}, {});", secs, json_action);
                        let _ = tv.evaluate_script(&script);
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
                let scale = window.scale_factor();
                if let Ok(mut r) = rect_for_loop.lock() {
                    r.x = new_pos.x as f64 / scale;
                    r.y = new_pos.y as f64 / scale;
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
                    r.width = logical_w;
                    r.height = logical_h;
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
