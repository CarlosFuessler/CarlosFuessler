use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

pub struct Taskbar {
    document: web_sys::Document,
}

impl Taskbar {
    pub fn new(document: &web_sys::Document) -> Self {
        Self { document: document.clone() }
    }

    pub fn add_app_button(&self, id: &str, title: &str) {
        let container = self.document.get_element_by_id("tb-apps").unwrap();
        let btn = self.document.create_element("button").unwrap();
        btn.set_attribute("id", &format!("tb-btn-{}", id)).unwrap();
        btn.set_attribute("class", "tb-app-btn").unwrap();
        btn.set_inner_html(title);
        container.append_child(&btn).unwrap();
    }

    pub fn remove_app_button(&self, id: &str) {
        let btn_id = format!("tb-btn-{}", id);
        if let Some(btn) = self.document.get_element_by_id(&btn_id) {
            if let Some(parent) = btn.parent_element() {
                parent.remove_child(&btn).unwrap();
            }
        }
    }

    pub fn set_active(&self, id: &str) {
        let container = self.document.get_element_by_id("tb-apps").unwrap();
        let children = container.children();
        for i in 0..children.length() {
            if let Some(child) = children.item(i) {
                child.class_list().remove_1("active").unwrap();
            }
        }
        let btn_id = format!("tb-btn-{}", id);
        if let Some(btn) = self.document.get_element_by_id(&btn_id) {
            btn.class_list().add_1("active").unwrap();
        }
    }

    /// Toggle the Start menu open/closed.
    pub fn toggle_start_menu(&self) {
        if let Some(menu) = self.document.get_element_by_id("start-menu") {
            menu.class_list().toggle("open").ok();
        }
    }

    pub fn setup_start_button(&self) {
        let doc = &self.document;

        // Start button click
        let d1 = doc.clone();
        let toggle_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |evt: web_sys::MouseEvent| {
                evt.stop_propagation();
                if let Some(menu) = d1.get_element_by_id("start-menu") {
                    menu.class_list().toggle("open").ok();
                }
            },
        );
        let start_btn = doc.get_element_by_id("start-btn").unwrap();
        start_btn.add_event_listener_with_callback("click", toggle_cb.as_ref().unchecked_ref()).unwrap();
        crate::app_state::store_closure(toggle_cb);

        // Document click: close menu
        let d2 = doc.clone();
        let close_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |_ev: web_sys::MouseEvent| {
                if let Some(menu) = d2.get_element_by_id("start-menu") {
                    menu.class_list().remove_1("open").ok();
                }
            },
        );
        doc.add_event_listener_with_callback("click", close_cb.as_ref().unchecked_ref()).unwrap();
        crate::app_state::store_closure(close_cb);

        // Stop click propagation inside Start Menu so clicking search input doesn't close it
        if let Some(menu_el) = doc.get_element_by_id("start-menu") {
            let menu_click_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
                move |evt: web_sys::MouseEvent| {
                    evt.stop_propagation();
                }
            );
            menu_el.add_event_listener_with_callback("click", menu_click_cb.as_ref().unchecked_ref()).unwrap();
            crate::app_state::store_closure(menu_click_cb);
        }


        // Keyboard shortcut: Ctrl+Esc or Meta (Windows key) to toggle Start menu
        let d3 = doc.clone();
        let key_cb = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
            move |ev: web_sys::KeyboardEvent| {
                // Windows key (Meta) or Ctrl+Esc
                if ev.key() == "Meta" || (ev.key() == "Escape" && ev.ctrl_key()) {
                    ev.prevent_default();
                    if let Some(menu) = d3.get_element_by_id("start-menu") {
                        menu.class_list().toggle("open").ok();
                    }
                }
            },
        );
        doc.add_event_listener_with_callback("keydown", key_cb.as_ref().unchecked_ref()).unwrap();
        crate::app_state::store_closure(key_cb);

        // Wire each start menu item (any element with data-app inside #start-menu)
        let d4 = doc.clone();
        if let Ok(list) = doc.query_selector_all("#start-menu [data-app]") {
            for i in 0..list.length() {
                if let Some(node) = list.item(i) {
                    if let Some(el) = node.dyn_ref::<web_sys::Element>() {
                        let item_doc = d4.clone();
                        let item_el: web_sys::Element = el.clone();
                        let item_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
                            move |evt: web_sys::MouseEvent| {
                                evt.stop_propagation();
                                if let Some(app) = item_el.get_attribute("data-app") {
                                    if let Some(menu) = item_doc.get_element_by_id("start-menu") {
                                        menu.class_list().remove_1("open").ok();
                                    }
                                    launch_app(&item_doc, &app);
                                }
                            },
                        );
                        el.add_event_listener_with_callback("click", item_cb.as_ref().unchecked_ref()).unwrap();
                        crate::app_state::store_closure(item_cb);
                    }
                }
            }
        }
    }
}

pub fn launch_app(document: &web_sys::Document, app: &str) {
    match app {
        "programs" => {} // hover-only category
        "file-manager" => crate::file_manager::FileManager::open(),
        "terminal" => crate::terminal::Terminal::open(document),
        "projects" => crate::projects_gallery::ProjectsGallery::open(document),
        "contact" => crate::contact::ContactApp::open(document),
        "about" => crate::markdown::MarkdownViewer::open(document, "About Me - Notepad", "content/about/about.md"),
        "cv" => crate::markdown::MarkdownViewer::open(document, "CV - Notepad", "content/cv/cv.md"),
        "skills" => crate::markdown::MarkdownViewer::open(document, "Skills - Notepad", "content/skills/skills.md"),
        "calculator" => crate::apps::calculator::CalculatorApp::open(document),
        "notepad" => crate::apps::notepad::NotepadApp::open(document),
        "calendar" => crate::apps::calendar::CalendarApp::open(document),
        "minesweeper" => crate::apps::minesweeper::MinesweeperApp::open(document),
        "snake" => crate::apps::snake::SnakeApp::open(document),
        "sysinfo" => crate::apps::sysinfo::SysInfoApp::open(document),
        "paint" => crate::apps::paint::PaintApp::open(document),
        "cdplayer" => crate::apps::cdplayer::CDPlayerApp::open(document),
        "help" => crate::markdown::MarkdownViewer::open(document, "Help", "content/about/about.md"),
        "lockout" => trigger_lockout(document),
        _ => { crate::app_state::create_window(app, app, 400, 300); }
    }
}

pub fn trigger_lockout(document: &web_sys::Document) {
    if let Some(overlay) = document.get_element_by_id("login-overlay") {
        overlay.set_attribute("style", "").unwrap();
    }
    if let Some(desktop) = document.get_element_by_id("desktop") {
        desktop.set_attribute("style", "display:none").unwrap();
    }
    if let Some(taskbar) = document.get_element_by_id("taskbar") {
        taskbar.set_attribute("style", "display:none").unwrap();
    }
    if let Some(pass_el) = document.get_element_by_id("login-pass") {
        if let Some(input) = pass_el.dyn_ref::<web_sys::HtmlInputElement>() {
            input.set_value("");
            input.focus().ok();
        }
    }
    if let Some(err_el) = document.get_element_by_id("login-error") {
        err_el.set_inner_html("");
    }
    if let Some(menu) = document.get_element_by_id("start-menu") {
        menu.class_list().remove_1("open").ok();
    }
}
