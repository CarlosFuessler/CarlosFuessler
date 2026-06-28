use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::vfs::FsEntry;

pub struct FileManager;

impl FileManager {
    /// Open the File Manager application in a new window.
    pub fn open() {
        // Check whether the VFS has been loaded yet.
        let vfs_ready = crate::app_state::with_app(|app| app.vfs.is_some());
        if !vfs_ready {
            web_sys::console::log_1(
                &"File Manager: VFS not loaded yet — try again shortly.".into(),
            );
            return;
        }

        // Create the window via the WindowManager.
        let win_id = crate::app_state::with_app(|app| {
            app.window_manager
                .create_window("file-manager", "File Manager", 700, 450)
        });

        // Retrieve the content element of the new window.
        let content = crate::app_state::with_app(|app| {
            app.window_manager
                .get_content(win_id)
                .expect("window content not found")
        });

        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");

        // ---- State ----
        let current_path: Rc<RefCell<String>> = Rc::new(RefCell::new("/".into()));

        // ---- Clear the window content ----
        content.set_inner_html("");

        // ---- Build toolbar ----
        let toolbar = document
            .create_element("div")
            .unwrap();
        toolbar.set_attribute("class", "fm-toolbar").unwrap();

        let back_btn = Self::create_tb_btn(&toolbar, &document, "← Back");
        let fwd_btn = Self::create_tb_btn(&toolbar, &document, "→ Forward");
        let up_btn = Self::create_tb_btn(&toolbar, &document, "↑ Up");

        // Disable Back/Forward for now.
        let _ = back_btn.set_attribute("disabled", "disabled");
        let _ = fwd_btn.set_attribute("disabled", "disabled");

        content.append_child(&toolbar).unwrap();

        // ---- Build split body ----
        let body = document.create_element("div").unwrap();
        body.set_attribute("class", "fm-body").unwrap();

        let tree_pane = document.create_element("div").unwrap();
        tree_pane.set_attribute("class", "fm-tree").unwrap();

        let files_pane = document.create_element("div").unwrap();
        files_pane.set_attribute("class", "fm-files").unwrap();

        body.append_child(&tree_pane).unwrap();
        body.append_child(&files_pane).unwrap();
        content.append_child(&body).unwrap();

        // ---- Initial render ----
        Self::render_tree(&tree_pane, &current_path, &files_pane, &document);
        Self::render_files(&files_pane, &current_path, &tree_pane, &document);

        // ---- Wire Up button ----
        let up_cur = current_path.clone();
        let up_tree = tree_pane.clone();
        let up_files = files_pane.clone();
        let up_doc = document.clone();
        let up_cb = Closure::<dyn FnMut()>::new(move || {
            let path = up_cur.borrow().clone();
            let parent = Self::parent_path(&path);
            if parent != path {
                *up_cur.borrow_mut() = parent;
                Self::render_tree(&up_tree, &up_cur, &up_files, &up_doc);
                Self::render_files(&up_files, &up_cur, &up_tree, &up_doc);
            }
        });
        up_btn
            .add_event_listener_with_callback("click", up_cb.as_ref().unchecked_ref())
            .unwrap();
        up_cb.forget();
    }

    // ---------------------------------------------------------------
    // Toolbar helpers
    // ---------------------------------------------------------------
    fn create_tb_btn(
        parent: &web_sys::Element,
        doc: &web_sys::Document,
        label: &str,
    ) -> web_sys::Element {
        let btn = doc.create_element("button").unwrap();
        btn.set_attribute("class", "fm-tb-btn").unwrap();
        btn.set_inner_html(label);
        parent.append_child(&btn).unwrap();
        btn
    }

    fn parent_path(path: &str) -> String {
        if path == "/" {
            return "/".into();
        }
        let trimmed = path.trim_end_matches('/');
        if let Some(pos) = trimmed.rfind('/') {
            if pos == 0 {
                "/".into()
            } else {
                trimmed[..pos].into()
            }
        } else {
            "/".into()
        }
    }

    // ---------------------------------------------------------------
    // Tree pane
    // ---------------------------------------------------------------
    fn render_tree(
        tree_cont: &web_sys::Element,
        current_path: &Rc<RefCell<String>>,
        files_cont: &web_sys::Element,
        doc: &web_sys::Document,
    ) {
        tree_cont.set_inner_html("");
        crate::app_state::with_app(|app| {
            if let Some(vfs) = &app.vfs {
                if let Some(entries) = vfs.list_children("/") {
                    Self::build_tree_items(
                        tree_cont,
                        entries,
                        current_path,
                        files_cont,
                        doc,
                        0,
                    );
                }
            }
        });
    }

    fn build_tree_items(
        container: &web_sys::Element,
        entries: &[FsEntry],
        current_path: &Rc<RefCell<String>>,
        files_cont: &web_sys::Element,
        doc: &web_sys::Document,
        depth: usize,
    ) {
        let cur_path = current_path.borrow().clone();

        for entry in entries {
            if entry.entry_type != "dir" {
                continue;
            }

            let item = doc.create_element("div").unwrap();
            let padding = depth * 16 + 4;
            let is_active = entry.path == cur_path;

            // Style
            let mut style = format!(
                "padding-left:{}px;cursor:pointer;white-space:nowrap;\
                 font-family:var(--font);font-size:var(--font-size);",
                padding
            );
            if is_active {
                style.push_str(
                    "background:var(--title-active);color:var(--title-active-text);",
                );
            }
            item.set_attribute("style", &style).unwrap();
            item.set_attribute("data-path", &entry.path).unwrap();

            item.set_inner_html(&format!("📁 {}", entry.name));

            // Click handler
            let item_path = entry.path.clone();
            let cur = current_path.clone();
            let tree = container.clone();
            let files = files_cont.clone();
            let d = doc.clone();
            let click_cb = Closure::<dyn FnMut()>::new(move || {
                *cur.borrow_mut() = item_path.clone();
                Self::render_tree(&tree, &cur, &files, &d);
                Self::render_files(&files, &cur, &tree, &d);
            });
            item.add_event_listener_with_callback("click", click_cb.as_ref().unchecked_ref())
                .unwrap();
            click_cb.forget();

            container.append_child(&item).unwrap();

            // Show children if this entry is the current path or an ancestor
            let should_show = is_active
                || cur_path.starts_with(&format!("{}/", entry.path));
            if should_show {
                if let Some(children) = &entry.children {
                    Self::build_tree_items(
                        container,
                        children,
                        current_path,
                        files_cont,
                        doc,
                        depth + 1,
                    );
                }
            }
        }
    }

    // ---------------------------------------------------------------
    // Files pane (right side)
    // ---------------------------------------------------------------
    fn render_files(
        files_cont: &web_sys::Element,
        current_path: &Rc<RefCell<String>>,
        tree_cont: &web_sys::Element,
        doc: &web_sys::Document,
    ) {
        files_cont.set_inner_html("");
        let path = current_path.borrow().clone();

        crate::app_state::with_app(|app| {
            if let Some(vfs) = &app.vfs {
                let entries = match vfs.list_children(&path) {
                    Some(e) => e,
                    None => return,
                };

                for entry in entries {
                    let item = doc.create_element("div").unwrap();
                    item.set_attribute("class", "fm-file-item").unwrap();
                    item.set_attribute("data-path", &entry.path).unwrap();
                    item.set_attribute("data-type", &entry.entry_type).unwrap();

                    let icon = if entry.entry_type == "dir" {
                        "📁"
                    } else {
                        "📄"
                    };
                    item.set_inner_html(&format!("{} {}", icon, entry.name));

                    // Double-click handler
                    let item_path = entry.path.clone();
                    let item_type = entry.entry_type.clone();
                    let cur = current_path.clone();
                    let files = files_cont.clone();
                    let tree = tree_cont.clone();
                    let d = doc.clone();
                    let dbl_cb = Closure::<dyn FnMut()>::new(move || {
                        if item_type == "dir" {
                            *cur.borrow_mut() = item_path.clone();
                            Self::render_tree(&tree, &cur, &files, &d);
                            Self::render_files(&files, &cur, &tree, &d);
                        } else if item_type == "file" && item_path.ends_with(".md") {
                            // Markdown Viewer doesn't exist yet (Task 8);
                            // for now, create a simple placeholder window.
                            let name = item_path
                                .split('/')
                                .last()
                                .unwrap_or(&item_path);
                            crate::app_state::with_app(|app| {
                                app.window_manager.create_window(
                                    "md-viewer",
                                    &format!("Opening {}...", name),
                                    500,
                                    400,
                                );
                            });
                        }
                    });
                    item.add_event_listener_with_callback(
                        "dblclick",
                        dbl_cb.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                    dbl_cb.forget();

                    files_cont.append_child(&item).unwrap();
                }
            }
        });
    }
}
