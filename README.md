# Google Chrome (Direct Native Browser in Rust)

A native desktop browser written in Rust with **direct native page loading** (NO iframes, NO proxy, NO CSRF/419 simulation issues).

## Fixed & Immutable Identity Information
This browser enforces 4 static parameters that **never change**, regardless of the page or website navigated:

| Parameter | Fixed Value |
|---|---|
| **Application Name** | `Brave` |
| **Window Title** | `Bluevy Admin - Brave` |
| **Application Vers** | `8037.58` |
| **Extended Info** | `http://localhost:3003/workspace/DataList?id=9b0f6bcb-c5d1-4c86-a4` |

---

## Architecture & Direct Browsing
- **Real Native Web Engine**: The browser view loads websites directly into a native webview without iframes or proxy wrappers. All cookies, sessions, authentication headers, and CSRF tokens work identically to standard Google Chrome.
- **Permanent Title Locking**: Window title is enforced to `Bluevy Admin - Google chrome` and cannot be modified by any web page.
- **Top System & Control Bar**: Displays the 4 static properties, navigation controls (Back, Forward, Reload, Home), and an address bar with direct URL and Google search capabilities.
- **Native User-Agent**: Configured with a modern Chrome User-Agent header.

---

## How to Run

```bash
cargo run
```
