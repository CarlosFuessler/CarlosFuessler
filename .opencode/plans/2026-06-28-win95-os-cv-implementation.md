# Win95 OS Desktop CV — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an interactive Windows 95-themed OS desktop CV website deployed on GitHub Pages.

**Architecture:** Rust WASM (wasm-bindgen + web-sys) handles all stateful logic — window manager, file system, terminal, login. Custom Win95 CSS provides the retro visual theme. Minimal JS glue for DOM events and WASM boot. Markdown files serve as content source.

**Tech Stack:** Rust + wasm-pack + wasm-bindgen + web-sys, Custom CSS, Vanilla JS, pulldown-cmark, serde, GitHub Pages

## Global Constraints

- No JavaScript/TypeScript framework (no React, Vue, Svelte, etc.)
- No CSS framework (no Tailwind, Bootstrap, etc.)
- No backend — entirely client-side static hosting
- All logic in Rust WASM; JS is only DOM event wiring
- Content in Markdown files under `content/`
- Must deploy to GitHub Pages with `wasm-pack build`

---

### Task 1: Project Scaffold + Build System

**Files:**
- Create: `index.html`
- Create: `style.css`
- Create: `js/main.js`
- Create: `rust-wasm/Cargo.toml`
- Create: `rust-wasm/src/main.rs`
- Create: `.gitignore`

**Interfaces:**
- Consumes: nothing
- Produces: WASM module exported as `init()` that logs to console. `index.html` loads WASM and shows a test message.

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "win95-desktop"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = [
    "Document", "Element", "Window", "HtmlElement", "MouseEvent",
    "KeyboardEvent", "HtmlDivElement", "Node", "EventTarget",
    "console", "DomRect", "HtmlInputElement",
    "HtmlButtonElement", "Storage",
] }
js-sys = "0.3"
```

- [ ] **Step 2: Create main.rs with minimal WASM entry**

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    let el = document.create_element("div")?;
    el.set_inner_html("<h1>Win95 OS Desktop — loading...</h1>");
    body.append_child(&el)?;

    Ok(())
}
```

- [ ] **Step 3: Create index.html**

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Carlos — CV</title>
  <link rel="stylesheet" href="style.css" />
</head>
<body>
  <div id="app"></div>
  <script type="module">
    import init from './pkg/win95_desktop.js';
    init();
  </script>
</body>
</html>
```

- [ ] **Step 4: Create style.css with minimal reset**

```css
*, *::before, *::after { margin: 0; padding: 0; box-sizing: border-box; }
html, body { width: 100%; height: 100%; overflow: hidden; background: #000; }
#app { width: 100%; height: 100%; position: relative; }
```

- [ ] **Step 5: Create js/main.js (placeholder)**

```js
// JS glue — will wire DOM events for WASM
export function setupEventListeners(elementId, eventType, callback) {
  const el = document.getElementById(elementId);
  if (el) el.addEventListener(eventType, callback);
}
```

- [ ] **Step 6: Create .gitignore**

```
/target
/pkg
.superpowers/
```

- [ ] **Step 7: Build and verify**

Run: `wasm-pack build rust-wasm --target web --out-dir ../pkg`
Verify: `ls pkg/win95_desktop_bg.wasm` exists
Open `index.html` in browser (via local server like `python3 -m http.server`) — page shows "Win95 OS Desktop — loading..."

- [ ] **Step 8: Commit**

```bash
git add -A && git commit -m "feat: initial project scaffold with Rust WASM"
```

---

### Task 2: Win95 CSS Design System

**Files:**
- Modify: `style.css`

**Interfaces:**
- Consumes: HTML elements with Win95 class names
- Produces: Complete Win95 visual language usable by all subsequent tasks

- [ ] **Step 1: Define CSS custom properties and global styles**

```css
:root {
  --desktop-bg: #008080;
  --silver: #c0c0c0;
  --silver-light: #dfdfdf;
  --silver-dark: #808080;
  --darkest: #000000;
  --white: #ffffff;
  --title-active: #000080;
  --title-active-text: #ffffff;
  --title-inactive: #808080;
  --title-inactive-text: #c0c0c0;
  --font: "MS Sans Serif", "Chicago", "Courier New", monospace;
  --font-size: 11px;
  --border-outset: 2px outset var(--white);
  --border-inset: 2px inset var(--silver-dark);
  --border-window: 2px outset var(--silver);
  --border-input: 2px inset var(--silver-dark);
  --taskbar-height: 40px;
  --titlebar-height: 22px;
}
```

- [ ] **Step 2: Add Win95 utility classes**

```css
/* Win95 3D borders */
.win95-outset { border: 2px outset var(--white); }
.win95-inset { border: 2px inset var(--silver-dark); }
.win95-sunken { border: 2px inset var(--silver-dark); background: var(--white); }
.win95-raised { border: 2px outset var(--white); background: var(--silver); }
.win95-btn {
  font-family: var(--font);
  font-size: var(--font-size);
  padding: 2px 12px;
  background: var(--silver);
  border: 2px outset var(--white);
  outline: none;
  cursor: pointer;
}
.win95-btn:active { border: 2px inset var(--silver-dark); }
.win95-titlebar {
  height: var(--titlebar-height);
  background: var(--title-active);
  color: var(--title-active-text);
  font-family: var(--font);
  font-size: var(--font-size);
  display: flex; align-items: center; padding: 0 4px;
  user-select: none; cursor: default;
}
.win95-titlebar.inactive { background: var(--title-inactive); color: var(--title-inactive-text); }
.win95-window {
  background: var(--silver);
  border: var(--border-window);
  box-shadow: 4px 4px 0 var(--darkest);
  position: absolute; display: flex; flex-direction: column;
  font-family: var(--font); font-size: var(--font-size);
}
.win95-title-btn {
  width: 16px; height: 14px; margin-left: 2px;
  border: 1px outset var(--white);
  background: var(--silver);
  font-size: 10px; line-height: 14px; text-align: center;
  cursor: pointer; display: flex; align-items: center; justify-content: center;
}
.win95-title-btn:active { border: 1px inset var(--silver-dark); }
```

- [ ] **Step 3: Add desktop and taskbar styles**

```css
#desktop {
  width: 100%; height: calc(100% - var(--taskbar-height));
  background: var(--desktop-bg);
  position: relative; overflow: hidden;
}
#taskbar {
  height: var(--taskbar-height);
  background: var(--silver);
  border-top: 2px outset var(--white);
  display: flex; align-items: center; padding: 0 4px;
  position: fixed; bottom: 0; left: 0; right: 0;
  z-index: 10000;
}
#start-btn {
  padding: 2px 8px; font-family: var(--font); font-size: var(--font-size);
  background: var(--silver); border: 2px outset var(--white);
  cursor: pointer; display: flex; align-items: center; gap: 4px;
  font-weight: bold; height: 30px;
}
#start-btn:active { border: 2px inset var(--silver-dark); }
.tb-app-btn {
  padding: 2px 8px; margin: 0 2px; height: 28px;
  font-family: var(--font); font-size: var(--font-size);
  background: var(--silver); border: 2px outset var(--white);
  cursor: pointer; display: flex; align-items: center; gap: 4px;
  max-width: 180px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.tb-app-btn.active { border: 2px inset var(--silver-dark); background: var(--silver-light); }
#sys-tray {
  margin-left: auto; display: flex; align-items: center; gap: 8px;
  padding: 0 8px; border-left: 1px inset var(--silver-dark);
  font-family: var(--font); font-size: var(--font-size); height: 28px;
}
```

- [ ] **Step 4: Add boot screen and login styles**

```css
#boot-screen {
  position: fixed; inset: 0; z-index: 99999;
  background: #000080; display: flex; flex-direction: column;
  align-items: center; justify-content: center; color: white;
  font-family: var(--font);
}
#boot-logo { font-size: 48px; margin-bottom: 40px; }
#boot-progress {
  width: 300px; height: 16px; border: 2px inset var(--silver-dark);
  background: var(--white); position: relative;
}
#boot-progress-fill {
  height: 100%; width: 0%; background: #000080;
  transition: width 0.3s linear;
}
#boot-text { margin-top: 8px; font-size: var(--font-size); color: #c0c0c0; }
#login-overlay {
  position: fixed; inset: 0; z-index: 99998;
  background: var(--desktop-bg); display: flex;
  align-items: center; justify-content: center;
}
#login-dialog {
  background: var(--silver); border: var(--border-window);
  box-shadow: 4px 4px 0 var(--darkest); width: 360px;
  font-family: var(--font); font-size: var(--font-size);
}
#login-body { padding: 16px; display: flex; flex-direction: column; gap: 8px; }
#login-body label { display: flex; align-items: center; gap: 8px; }
#login-body input {
  flex: 1; padding: 2px 4px; font-family: var(--font); font-size: var(--font-size);
  border: var(--border-input); background: var(--white);
}
#login-btns { display: flex; justify-content: flex-end; gap: 8px; margin-top: 8px; }
```

- [ ] **Step 5: Commit**

```bash
git add style.css && git commit -m "feat: add Win95 CSS design system"
```

---

### Task 3: Boot Screen + Login

**Files:**
- Modify: `rust-wasm/src/main.rs`
- Create: `rust-wasm/src/login.rs`
- Modify: `index.html`
- Create: `rust-wasm/templates/app.html`

**Interfaces:**
- Consumes: CSS classes from Task 2
- Produces: `LoginResult { username: String, is_guest: bool }` — emitted after login success

- [ ] **Step 1: Create HTML template for app shell (`rust-wasm/templates/app.html`)**

```html
<div id="boot-screen">
  <div id="boot-logo">⬛ Windows 95</div>
  <div id="boot-progress"><div id="boot-progress-fill"></div></div>
  <div id="boot-text">Starting Windows 95...</div>
</div>
<div id="login-overlay" style="display:none;">
  <div id="login-dialog">
    <div class="win95-titlebar">Enter Network Password</div>
    <div id="login-body">
      <p>Enter your name and password to log on.</p>
      <label>Username: <input id="login-user" type="text" /></label>
      <label>Password: <input id="login-pass" type="password" /></label>
      <div id="login-btns">
        <button id="login-ok" class="win95-btn">OK</button>
        <button id="login-guest" class="win95-btn">Guest</button>
      </div>
    </div>
  </div>
</div>
<div id="desktop" style="display:none;">
  <div id="desktop-icons"></div>
</div>
<div id="taskbar" style="display:none;">
  <button id="start-btn">⬛ Start</button>
  <div id="tb-apps"></div>
  <div id="sys-tray"><span id="clock"></span></div>
</div>
```

- [ ] **Step 2: Update main.rs with boot flow**

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod login;
use login::{show_login, LoginResult};

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    body.set_inner_html(include_str!("../templates/app.html"));
    show_boot_screen(&document);

    Ok(())
}

fn show_boot_screen(document: &web_sys::Document) {
    let boot = document.get_element_by_id("boot-screen").unwrap();
    let fill = document.get_element_by_id("boot-progress-fill").unwrap();
    let text = document.get_element_by_id("boot-text").unwrap();
    // Animate progress bar over ~3 seconds then show login
    // Use setTimeout chain
}
```

- [ ] **Step 3: Implement login.rs**

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub struct LoginResult {
    pub username: String,
    pub is_guest: bool,
}

pub fn show_login(document: &web_sys::Document) {
    let overlay = document.get_element_by_id("login-overlay").unwrap();
    overlay.set_attribute("style", "").unwrap();
    // Wire OK and Guest buttons
}

fn start_desktop(document: &web_sys::Document, result: LoginResult) {
    document.get_element_by_id("login-overlay").unwrap()
        .set_attribute("style", "display:none").unwrap();
    document.get_element_by_id("desktop").unwrap()
        .set_attribute("style", "").unwrap();
    document.get_element_by_id("taskbar").unwrap()
        .set_attribute("style", "").unwrap();
}
```

- [ ] **Step 4: Build and verify**

Build → browser → boot screen → progress → login → enter credentials → desktop appears

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: add boot screen and login flow"
```

---

### Task 4: Desktop Icons + Taskbar

**Files:**
- Modify: `rust-wasm/src/main.rs`
- Create: `rust-wasm/src/taskbar.rs`
- Create: `rust-wasm/src/desktop.rs`

**Interfaces:**
- Consumes: LoginResult from Task 3
- Produces: Taskbar with app buttons, desktop icons

- [ ] **Step 1: Create desktop.rs**

```rust
pub struct DesktopIcon {
    pub id: String,
    pub title: String,
    pub icon: &'static str,
}

pub fn render_desktop_icons(document: &web_sys::Document, icons: &[DesktopIcon]) {
    let container = document.get_element_by_id("desktop-icons").unwrap();
    container.set_inner_html("");
    for icon in icons {
        let el = document.create_element("div").unwrap();
        el.set_attribute("class", "desktop-icon").unwrap();
        el.set_inner_html(&format!(
            r#"<div class="desktop-icon-img">{}</div><div class="desktop-icon-label">{}</div>"#,
            icon.icon, icon.title
        ));
        container.append_child(&el).unwrap();
    }
}
```

- [ ] **Step 2: Create taskbar.rs**

```rust
pub struct Taskbar {
    document: web_sys::Document,
}

impl Taskbar {
    pub fn add_app_button(&self, id: &str, title: &str) {
        // Create button in #tb-apps
    }
    pub fn remove_app_button(&self, id: &str) {
        // Remove button by id
    }
    pub fn set_active(&self, id: &str) {
        // Toggle active class
    }
}
```

- [ ] **Step 3: Add desktop icon CSS**

```css
.desktop-icon {
  display: flex; flex-direction: column; align-items: center;
  width: 72px; padding: 4px; cursor: pointer; text-align: center;
  color: white; font-family: var(--font); font-size: var(--font-size);
  user-select: none;
}
.desktop-icon:hover { background: rgba(255,255,255,0.1); }
.desktop-icon-img { font-size: 32px; }
#desktop-icons {
  position: absolute; top: 16px; left: 16px;
  display: flex; flex-direction: column; flex-wrap: wrap; gap: 8px;
}
```

- [ ] **Step 4: Integrate in main.rs start_desktop**

```rust
fn start_desktop(document: &web_sys::Document, result: LoginResult) {
    // ... hide login ...
    let icons = vec![
        DesktopIcon { id: "my-pc".into(), title: "My PC".into(), icon: "💻" },
        DesktopIcon { id: "projects".into(), title: "Projects".into(), icon: "📁" },
        DesktopIcon { id: "terminal".into(), title: "Terminal".into(), icon: "⬛" },
    ];
    render_desktop_icons(&document, &icons);
}
```

- [ ] **Step 5: Build and verify**

Build → login → desktop with 3 icons + taskbar visible

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "feat: add desktop icons and taskbar"
```

---

### Task 5: Window Manager

**Files:**
- Create: `rust-wasm/src/window_manager.rs`
- Modify: `rust-wasm/src/main.rs`

**Interfaces:**
- Consumes: nothing
- Produces: WindowManager struct with create_window, close_window, focus_window

- [ ] **Step 1: Define Window and WindowManager structs**

```rust
#[derive(Clone, Copy, PartialEq)]
pub enum WindowState { Open, Minimized, Maximized, Closed }

pub struct Window {
    pub id: u32, pub app_id: String, pub title: String,
    pub x: i32, pub y: i32, pub w: u32, pub h: u32,
    pub prev_x: i32, pub prev_y: i32,
    pub prev_w: u32, pub prev_h: u32,
    pub z_index: u32, pub state: WindowState,
    pub element: web_sys::Element,
    pub content: web_sys::Element,
}

pub struct WindowManager {
    next_id: u32, next_z: u32,
    windows: Vec<Window>,
    document: web_sys::Document,
    desktop: web_sys::Element,
}
```

- [ ] **Step 2: Implement core methods**

Implement `new`, `create_window` (builds DOM element with title bar + content + title buttons), `focus_window` (updates z-index and title bar style), `close_window`, `minimize_window`, `maximize_window`.

- [ ] **Step 3: Implement drag behavior**

mousedown on titlebar tracks mousemove delta → updates left/top. mouseup stops tracking.

- [ ] **Step 4: Add window content area CSS**

```css
.win95-content {
  flex: 1; overflow: auto;
  background: var(--white);
  border: 2px inset var(--silver-dark);
  margin: 2px; padding: 4px;
}
```

- [ ] **Step 5: Test with a test window**

```rust
fn start_desktop(document: &web_sys::Document, result: LoginResult) {
    // ... existing code ...
    let mut wm = WindowManager::new(&document);
    wm.create_window("test", "Welcome", 500, 300);
}
```

- [ ] **Step 6: Build and verify**

Login → test window appears → drag it → click title buttons → minimize/maximize/close work

- [ ] **Step 7: Commit**

```bash
git add -A && git commit -m "feat: add window manager with drag and z-order"
```

---

### Task 6: Start Menu

**Files:**
- Modify: `rust-wasm/src/taskbar.rs`
- Modify: `rust-wasm/src/main.rs`

- [ ] **Step 1: Add start menu CSS**

```css
#start-menu {
  position: absolute; bottom: var(--taskbar-height); left: 0;
  background: var(--silver); border: var(--border-outset);
  box-shadow: 4px 4px 0 var(--darkest);
  z-index: 20000; min-width: 200px;
  font-family: var(--font); font-size: var(--font-size);
}
.start-menu-item { padding: 6px 16px; cursor: pointer; }
.start-menu-item:hover { background: var(--title-active); color: var(--title-active-text); }
.start-menu-sep { height: 2px; margin: 4px 8px; border-top: 1px inset var(--silver-dark); }
```

- [ ] **Step 2: Add start menu HTML to app.html template**

Inside taskbar div, add:
```html
<div id="start-menu" style="display:none;">
  <div class="start-menu-item" data-app="file-manager">📁 File Manager</div>
  <div class="start-menu-item" data-app="terminal">⬛ Terminal</div>
  <div class="start-menu-item" data-app="projects">📁 Projects</div>
  <div class="start-menu-item" data-app="about">📄 About Me</div>
  <div class="start-menu-item" data-app="contact">✉ Contact</div>
  <div class="start-menu-sep"></div>
  <div class="start-menu-item" data-app="shutdown">🔌 Shut Down...</div>
</div>
```

- [ ] **Step 3: Wire start button toggle and menu items**

Toggle display of #start-menu on click. Close when clicking outside. Each data-app value triggers the corresponding launch function or shutdown.

- [ ] **Step 4: Implement shutdown animation**

Countdown overlay → "It's now safe to turn off your computer" → click to reload

- [ ] **Step 5: Build and verify**

Start menu opens/closes → clicking items triggers placeholders → shutdown works

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "feat: add start menu with shutdown"
```

---

### Task 7: File Manager + Virtual File System

**Files:**
- Create: `rust-wasm/src/vfs.rs`
- Create: `rust-wasm/src/file_manager.rs`
- Create: `content/filesystem.json`
- Modify: `Cargo.toml` (add serde, serde_json)
- Modify: `rust-wasm/src/main.rs`

- [ ] **Step 1: Create content/filesystem.json**

```json
[
  { "name": "About Me", "path": "/about", "type": "dir", "children": [
    { "name": "about.md", "path": "/about/about.md", "type": "file", "children": null }
  ]},
  { "name": "CV & Experience", "path": "/cv", "type": "dir", "children": [
    { "name": "cv.md", "path": "/cv/cv.md", "type": "file", "children": null }
  ]},
  { "name": "Projects", "path": "/projects", "type": "dir", "children": [
    { "name": "project-1.md", "path": "/projects/project-1.md", "type": "file", "children": null }
  ]},
  { "name": "Skills", "path": "/skills", "type": "dir", "children": [
    { "name": "skills.md", "path": "/skills/skills.md", "type": "file", "children": null }
  ]},
  { "name": "Contact", "path": "/contact", "type": "dir", "children": [
    { "name": "contact.md", "path": "/contact/contact.md", "type": "file", "children": null }
  ]}
]
```

- [ ] **Step 2: Create vfs.rs with VirtualFS struct**

Load filesystem.json via fetch, parse with serde_json, expose `list_children(path)` to get directory entries.

- [ ] **Step 3: Create file_manager.rs with Explorer UI**

Window with toolbar (Back, Forward, Up), left folder tree pane, right file list pane. Double-click folder → navigate. Double-click .md → trigger MarkdownViewer::open.

- [ ] **Step 4: Wire launch_file_manager in main.rs**

Store WindowManager in Rc<RefCell<>> for shared access between modules.

- [ ] **Step 5: Build and verify**

Open File Manager → see folder tree → browse folders → double-click .md shows "opening..."

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "feat: add file manager with virtual filesystem"
```

---

### Task 8: Markdown Viewer

**Files:**
- Create: `rust-wasm/src/markdown.rs`
- Modify: `Cargo.toml` (add pulldown-cmark)
- Modify: `rust-wasm/src/main.rs`

- [ ] **Step 1: Add pulldown-cmark to Cargo.toml**

```toml
pulldown-cmark = "0.11"
```

- [ ] **Step 2: Create markdown.rs**

Fetches .md file via fetch, parses with pulldown-cmark, renders HTML into a markdown viewer window.

- [ ] **Step 3: Add markdown body CSS**

```css
.markdown-body { padding: 8px; line-height: 1.5; }
.markdown-body h1 { font-size: 18px; margin: 8px 0; }
.markdown-body h2 { font-size: 14px; margin: 8px 0; }
.markdown-body code { background: #eee; padding: 1px 4px; }
.markdown-body pre { background: #eee; padding: 8px; margin: 8px 0; }
```

- [ ] **Step 4: Wire file double-click to MarkdownViewer**

- [ ] **Step 5: Build and verify**

Double-click .md in file manager → viewer opens with rendered markdown

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "feat: add markdown viewer"
```

---

### Task 9: Terminal

**Files:**
- Create: `rust-wasm/src/terminal.rs`
- Modify: `rust-wasm/src/main.rs`

- [ ] **Step 1: Create terminal.rs**

Window with black background, green text, blinking cursor. Input line at bottom. Commands: help, whoami, projects, skills, email, clear, exit. Command parsing in Rust.

- [ ] **Step 2: Build and verify**

Open Terminal → type commands → see responses → clear → exit closes

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat: add terminal app"
```

---

### Task 10: Projects Gallery + Contact

**Files:**
- Create: `rust-wasm/src/projects_gallery.rs`
- Create: `rust-wasm/src/contact.rs`
- Modify: `rust-wasm/src/main.rs`

- [ ] **Step 1: Create projects_gallery.rs**

Card layout with project screenshots, descriptions, tech tags. Filter buttons by tag.

- [ ] **Step 2: Create contact.rs**

Window with email, GitHub, LinkedIn links.

- [ ] **Step 3: Add project card CSS**

```css
.project-card { width: 280px; border: 1px inset var(--silver-dark); background: var(--white); padding: 8px; }
```

- [ ] **Step 4: Build and verify**

Open Projects → see cards → filter works → Open Contact → see links

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: add projects gallery and contact apps"
```

---

### Task 11: Content Files + Auto-Open + Polish

**Files:**
- Create: `content/about.md`, `content/cv.md`, `content/skills.md`, `content/contact.md`
- Create: `content/projects/project-1.md`
- Modify: `rust-wasm/src/main.rs`

- [ ] **Step 1: Write markdown content files**

about.md, cv.md, skills.md, contact.md, project-1.md with real content.

- [ ] **Step 2: Auto-open About Me after login**

```rust
fn start_desktop(document: &web_sys::Document, result: LoginResult) {
    // ... existing code ...
    MarkdownViewer::open(&mut wm, &document, "About Me - Notepad", "content/about.md");
}
```

- [ ] **Step 3: Build and verify full flow**

Boot → login → desktop → About Me auto-opens → all apps work end-to-end

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: add content files and auto-open about me"
```

---

### Task 12: Final Polish + Deployment

**Files:**
- Modify: `index.html` (meta tags, favicon)

- [ ] **Step 1: Add meta tags, verify full build**

- [ ] **Step 2: Test locally with full reload**

- [ ] **Step 3: Commit and push**

```bash
git add -A && git commit -m "chore: final polish for deployment"
git push origin main
```

- [ ] **Step 4: Enable GitHub Pages in repo settings**
