# Brave Browser (Direct Native Browser in Rust)

A native desktop browser written in Rust with **direct native page loading** (NO iframes, NO proxy, NO CSRF/419 simulation issues).

## Fixed & Immutable Identity Information
This browser enforces 4 static parameters that **never change**, regardless of the page or website navigated:

| Parameter | Fixed Value |
|---|---|
| **Application Name** | `Brave Browser` |
| **Window Title** | `Bluevy Admin - Brave Browser` |
| **Application Vers** | `195.104` |
| **Extended Info** | `http://localhost:3001/workspace/DataList?id=9b0f6bcb-c5d1-4c86-a4` |

---

## Architecture & Features
- **Real Native Web Engine**: The browser view loads websites directly into native webviews without iframes or proxy wrappers. All cookies, sessions, authentication headers, and CSRF tokens work identically to standard Brave / Chromium.
- **Permanent Title Locking**: Window title is enforced to `Bluevy Admin - Brave Browser` and cannot be modified by any web page.
- **Top Multi-Tabs Bar**: Tab system supporting multiple concurrent tabs, closing tabs, and switching tabs.
- **Top Right Auto Control (`F8`)**:
  - Toggled via the top-right button or pressing **`F8`**.
  - Naturally glides the actual PC mouse cursor along human-like Bézier curves.
  - Smooth page scrolling, random text selection, and empty background clicking.
  - Synchronous human-cadence typing in the address bar (a-z, 0-9) from realistic dictionary queries without submitting / pressing Enter.
  - Randomized action pauses between **5 and 60 seconds**.
- **macOS Activity Tracker Compatibility**:
  - Bundled as `Brave.app` with `com.brave.Browser` bundle identifier and version `195.104` for RescueTime tracking.

---

## How to Run

```bash
./run.sh
```
or
```bash
cargo run
```
