# Carlos' CV & Portfolio (Windows 95 Desktop Simulation)

Welcome to my personal CV and portfolio website, designed to look and feel like a classic Windows 95 desktop simulation. 

This project is built using a modern high-performance tech stack: **Rust** compiled to **WebAssembly (WASM)**, making direct DOM calls using `web-sys` and `wasm-bindgen` (without any heavy JavaScript framework like React, Vue, or Angular). Styling is strictly controlled by custom vanilla CSS matching the retro theme.

---

## 🚀 Tech Stack

- **Frontend Core**: Rust (2021 edition) compiled to `wasm32-unknown-unknown`.
- **JS/DOM Bindings**: `wasm-bindgen` (0.2), `wasm-bindgen-futures` (0.4), `web-sys` (0.3), and `js-sys` (0.3).
- **Markdown Parsing**: `pulldown-cmark` (0.11) for rendering markdown files on the fly.
- **Serialization**: `serde` & `serde_json` (1.0) for loading config and virtual filesystem files.
- **Build System**: Zig (`build.zig`) orchestrating the WASM build and local HTTP server execution.
- **Development Server**: Minimal Rust-based static file server (`serve` crate) utilizing standard `TcpListener` that serves files with appropriate MIME types (specifically supporting `.wasm` files).
- **Styling**: Vanilla CSS (`style.css`) providing authentic Windows 95 borders, beveled buttons, `#008080` teal desktop background, and MS Sans Serif-like typography.

---

## 📂 Project Structure

```
├── .github/workflows/   # CI/CD deployment pipelines
│   └── deploy.yml       # GitHub Actions workflow for building & deploying to Pages
├── assets/              # Static media assets (wallpaper, images)
├── content/             # Markdown files (CV sections) & Virtual File System (VFS) map
│   ├── about.md
│   ├── contact.md
│   ├── cv.md
│   ├── projects.md
│   ├── skills.md
│   └── filesystem.json  # Structure definitions for the simulated filesystem
├── js/                  # JS bootstrap scripts
│   └── main.js          # Main script loading the WebAssembly module
├── pkg/                 # Compiled WASM artifacts (gitignored)
├── rust-wasm/           # Rust source code of the frontend desktop environment
│   ├── src/
│   │   ├── main.rs            # Entry point; handles login flow and initialization
│   │   ├── app_state.rs       # Central application state and safe thread-local accessors
│   │   ├── desktop.rs         # Desktop shortcuts rendering and drag-and-drop
│   │   ├── file_manager.rs    # Windows Explorer-like directory explorer
│   │   ├── markdown.rs        # Markdown to HTML conversion routines
│   │   ├── taskbar.rs         # Taskbar, window buttons, and Start Menu controller
│   │   ├── terminal.rs        # Interactive command prompt window with custom commands
│   │   ├── vfs.rs             # Virtual File System logic
│   │   └── window_manager.rs  # Dragging, resizing, z-indexing, and window state manager
│   │   └── apps/              # Desktop applications (minesweeper, paint, notepad, etc.)
│   └── Cargo.toml             # Cargo manifest for the Rust frontend
├── serve/               # Lightweight developer server in Rust
│   ├── src/main.rs        # Port listener and file serving logic
│   └── Cargo.toml         # Cargo manifest for the local dev server
├── index.html           # Main page entry point
├── style.css            # Windows 95 retro stylesheet
├── build.zig            # Zig build file orchestrating the development workflows
├── Cargo.toml           # Root workspace Cargo manifest
└── README.md            # You are here!
```

---

## 🛠️ Local Development

### Prerequisites

To build and run this project locally, you need the following tools installed:
- **Rust**: Install via [rustup.rs](https://rustup.rs/) (targets `wasm32-unknown-unknown`).
- **wasm-pack**: Install via `cargo install wasm-pack` or via curl.
- **Zig**: Install Zig (latest stable version) to run orchestrator commands.

### Build and Run commands

All main development tasks are orchestrated using **Zig** build steps:

1. **Build WASM & Start Dev Server** (Default):
   ```bash
   zig build
   # or simply
   zig
   ```
   This compiles the WebAssembly frontend and starts the local Rust dev server (typically serving on `http://localhost:8080` or the port output in the terminal).

2. **Build WebAssembly only**:
   ```bash
   zig build wasm
   ```
   This runs the development build using `wasm-pack` inside `rust-wasm`, outputting artifacts to `/pkg`.

3. **Run Dev Server only**:
   ```bash
   zig build run
   ```
   This starts the `serve` Cargo crate to host the current `/pkg` and workspace static assets.

4. **Production Build** (Manual):
   ```bash
   cd rust-wasm
   wasm-pack build --target web --out-dir ../pkg --release
   ```

---

## 📝 Code Conventions & Architectural Patterns

When modifying the Rust frontend, please strictly follow these rules to maintain stability:

### 1. Safe State Borrowing & Re-entrancy
* **Problem**: The global `APP_STATE` is stored inside a thread-local `RefCell`. Nesting calls to `with_app` or `with_wm` will cause a runtime panic if they attempt to borrow the cell when it is already borrowed.
* **Pattern**: Avoid nested borrows of `APP_STATE`. If a window manager operation needs to update the taskbar, do not do both borrows simultaneously. Instead, use high-level helper functions in [app_state.rs](file:///Users/carlos/Developer/Projects/github.io/rust-wasm/src/app_state.rs) (e.g. `create_window`, `close_window`, `focus_window`). These helpers safely borrow the window manager, extract the necessary data, drop the borrow, and then borrow the taskbar.

### 2. Event Listeners & Closure Memory Management
* **Problem**: When passing a Rust closure to JavaScript using `add_event_listener_with_callback`, JavaScript only holds a weak reference. If the closure is dropped in Rust, calling it from a DOM event will crash.
* **Pattern**: Always store the closure in the global state using `crate::app_state::store_closure(closure)` immediately after registering it to prevent it from being garbage collected.
* **Example**:
  ```rust
  let doc = document.clone();
  let cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |_evt| {
      // Event logic here
  });
  el.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref()).unwrap();
  crate::app_state::store_closure(cb);
  ```

### 3. DOM Manipulation & Element Creation
* Always query and cast elements using `web_sys` methods safely. Check for `Option` and `Result` values.
* To clear elements before re-rendering, use:
  ```rust
  let container = document.get_element_by_id("container-id").unwrap();
  container.set_inner_html("");
  ```

### 4. Dynamic Content
To add a new document or folder to the site:
1. Create/add the markdown file in `content/` (e.g., `content/projects/new-project.md`).
2. Add its reference path to the virtual filesystem registry in `content/filesystem.json`.

---

## 🌐 Deployment to GitHub Pages

The repository contains a GitHub Actions workflow that automates the deployment to GitHub Pages. On every push to the `main` branch, the workflow:
1. Checks out the code.
2. Configures a stable Rust toolchain with the WASM target.
3. Installs `wasm-pack`.
4. Caches cargo dependencies for faster runs.
5. Builds the WebAssembly bundle in `--release` mode.
6. Packages the static site assets (`index.html`, `style.css`, `pkg/`, `js/`, `assets/`, `content/`) into a `dist/` folder.
7. Deploys the package to GitHub Pages.

### Setup Instructions
To set up GitHub Pages for this repository:
1. Push this code to your GitHub repository.
2. Go to **Settings** -> **Pages** in your GitHub repository.
3. Under **Build and deployment** -> **Source**, select **GitHub Actions**.
4. The workflow will run automatically on the next commit and publish the site!
