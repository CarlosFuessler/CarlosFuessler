# Win95 OS Desktop CV — Design Spec

## Overview

A GitHub Pages-hosted interactive resume/CV website themed as a Windows 95 desktop operating system. The page boots into a retro OS environment with a login screen, window manager, taskbar, start menu, and themed apps including a file manager, markdown viewer, terminal, and projects gallery.

## Architecture

GitHub Pages static hosting. All logic client-side. Rust WASM (wasm-bindgen + web-sys) for stateful logic. Minimal JS glue for DOM events and WASM boot. Custom Windows 95 CSS theme. Markdown content as static files.

## Tech Stack

| Layer | Technology |
|---|---|
| Logic | Rust + wasm-bindgen + web-sys |
| DOM glue | Vanilla JS (minimal) |
| Styling | Custom Win95 CSS |
| Build | wasm-pack --target web |
| Content | Markdown (pulldown-cmark) |
| Hosting | GitHub Pages |

## Boot Sequence

1. Page loads → Windows 95 startup screen (flying Windows logo, loading bar)
2. Login dialog appears (username + password fields, Win95 network login style)
3. Correct credentials → full desktop; Guest → limited content
4. Desktop renders (teal #008080 background, desktop icons, taskbar)
5. About Me notepad opens automatically

## Window Manager

Windows tracked as Vec<Window> in Rust. Each window has id, title, app, x/y/w/h, z_index, state (Open/Minimized/Maximized/Closed).

Behaviors:
- Drag by title bar, resize from edges/corners (min 200x150)
- Z-order: click brings to front (max z_index + 1)
- Minimize: hide + taskbar button. Maximize: snap to desktop bounds
- Close: remove window + taskbar entry
- Double-click title bar: toggle maximize/restore
- Active title bar: #000080 with white text; inactive: gray

All windows children of #desktop div, positioned absolute. Rust diffs DOM via web-sys.

## File Manager & Virtual FS

Virtual FS mirrors content/ directory via filesystem.json manifest:

```
My PC/
├── About Me / about.md
├── CV & Experience / cv.md
├── Projects / project-alpha.md, project-beta.md, screenshots/
├── Skills / skills.md
└── Contact / contact.md
```

UI: Windows 95 Explorer — left pane folder tree, right pane file listing, toolbar (Back/Forward/Up). Double-click .md → Markdown Viewer. Double-click folder → navigate in.

## Apps

- Markdown Viewer: Opens .md files, renders via pulldown-cmark, scrollable window
- Terminal: Green-on-black faux terminal. Commands: help, ls, cat, whoami, projects, skills, email, neofetch, clear, shutdown. Input history with up/down arrow.
- Projects Gallery: Card layout with screenshots, descriptions, tech tags, GitHub links. Filterable by tag.
- About Me: Pre-opened on boot, renders about.md in a notepad window
- Contact: Email, GitHub, LinkedIn links with icons

## Start Menu & Desktop

Start Menu: shortcuts to all apps, separator, Shut Down... (fake shutdown animation).

Desktop icons: My PC, Projects, Terminal — CSS-drawn 32x32 pixel-art.

Taskbar: fixed bottom, Start button (green text + Windows flag), running app buttons, system tray clock.

## Login Screen

Win95 network login dialog: "Enter Network Password" title bar, Username + Password fields, OK/Cancel buttons, Guest access. Password validated in WASM (client-side).

## Visual Theme

| Element | Value |
|---|---|
| Desktop bg | #008080 (teal) |
| Taskbar/window bg | #c0c0c0 (silver) |
| Active title bar | #000080 navy, white text |
| 3D outset border | top/left: #fff, bottom/right: #000 |
| 3D inset border | top/left: #808080, bottom/right: #fff |
| Font | MS Sans Serif, Chicago, monospace |
| Font size | 11px body |

## Project Structure

```
github.io/
├── index.html
├── style.css
├── content/
│   ├── about.md, cv.md, skills.md, contact.md
│   ├── projects/ (project-*.md, screenshots/)
│   └── filesystem.json
├── rust-wasm/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs, login.rs, window_manager.rs
│       ├── file_manager.rs, markdown.rs, terminal.rs
│       ├── projects_gallery.rs, contact.rs
│       ├── taskbar.rs, desktop.rs, vfs.rs
├── js/
│   └── main.js
└── pkg/ (compiled WASM, gitignored)
```

## Key Dependencies

wasm-bindgen, web-sys (Document, Element, Window, MouseEvent, KeyboardEvent, etc.), js-sys, pulldown-cmark, serde, serde_json

## Deployment

wasm-pack build → push to GitHub → enable Pages on main branch.
