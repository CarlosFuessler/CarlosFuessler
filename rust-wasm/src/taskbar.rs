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
            let style = menu.get_attribute("style").unwrap_or_default();
            let hidden = style.contains("display:none") || style.is_empty();
            menu.set_attribute("style", if hidden { "display:block" } else { "display:none" }).unwrap();
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
                    let style = menu.get_attribute("style").unwrap_or_default();
                    let hidden = style.contains("display:none") || style.is_empty();
                    menu.set_attribute("style", if hidden { "display:block" } else { "display:none" }).unwrap();
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
                    menu.set_attribute("style", "display:none").unwrap();
                }
            },
        );
        doc.add_event_listener_with_callback("click", close_cb.as_ref().unchecked_ref()).unwrap();
        crate::app_state::store_closure(close_cb);

        // Keyboard shortcut: Ctrl+Esc or Meta (Windows key) to toggle Start menu
        let d3 = doc.clone();
        let key_cb = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
            move |ev: web_sys::KeyboardEvent| {
                // Windows key (Meta) or Ctrl+Esc
                if ev.key() == "Meta" || (ev.key() == "Escape" && ev.ctrl_key()) {
                    ev.prevent_default();
                    if let Some(menu) = d3.get_element_by_id("start-menu") {
                        let style = menu.get_attribute("style").unwrap_or_default();
                        let hidden = style.contains("display:none") || style.is_empty();
                        menu.set_attribute("style", if hidden { "display:block" } else { "display:none" }).unwrap();
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
                                        menu.set_attribute("style", "display:none").unwrap();
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
        "shutdown" => trigger_shutdown(document),
        _ => { crate::app_state::create_window(app, app, 400, 300); }
    }
}

pub fn trigger_shutdown(document: &web_sys::Document) {
    let overlay = document.create_element("div").unwrap();
    overlay.set_attribute("id", "shutdown-overlay").unwrap();
    overlay.set_attribute("style",
        "position:fixed;inset:0;z-index:99999;background:#000080;\
         display:flex;align-items:center;justify-content:center;\
         flex-direction:column;gap:16px;"
    ).unwrap();
    overlay.set_inner_html(
        "<div style='color:white;font-size:18px;'>\
         Please wait while your computer shuts down.\
         </div>\
         <div id='shutdown-counter' style='color:white;font-size:24px;'>5</div>"
    );
    document.body().unwrap().append_child(&overlay).unwrap();

    let window = web_sys::window().unwrap();
    let doc = document.clone();
    let cb_holder: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let counter: Rc<RefCell<i32>> = Rc::new(RefCell::new(5));

    let closure = {
        let doc = doc.clone();
        let window = window.clone();
        let cb_holder = cb_holder.clone();
        let counter = counter.clone();
        Closure::<dyn FnMut()>::new(move || {
            let mut c = counter.borrow_mut();
            *c -= 1;
            let count = *c;
            if count > 0 {
                if let Some(el) = doc.get_element_by_id("shutdown-counter") {
                    el.set_inner_html(&count.to_string());
                }
                if let Some(cb_ref) = cb_holder.borrow().as_ref() {
                    let cb_fn = cb_ref.as_ref().unchecked_ref::<js_sys::Function>().clone();
                    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&cb_fn, 1000);
                }
            } else {
                if let Some(el) = doc.get_element_by_id("shutdown-overlay") {
                    el.set_inner_html(
                        "<div style='color:white;font-size:24px;'>\
                         It's now safe to turn off your computer.\
                         </div>"
                    );
                    let reload_cb = Closure::<dyn FnMut()>::new(move || {
                        if let Some(w) = web_sys::window() {
                            w.location().reload().ok();
                        }
                    });
                    el.add_event_listener_with_callback("click", reload_cb.as_ref().unchecked_ref()).unwrap();
                    crate::app_state::store_closure(reload_cb);
                }
                *cb_holder.borrow_mut() = None;
            }
        })
    };

    let cb_fn = closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
    *cb_holder.borrow_mut() = Some(closure);
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&cb_fn, 1000);
}
