use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

pub struct Taskbar {
    document: web_sys::Document,
}

impl Taskbar {
    pub fn new(document: &web_sys::Document) -> Self {
        Self {
            document: document.clone(),
        }
    }

    pub fn add_app_button(&self, id: &str, title: &str) {
        let container = self.document.get_element_by_id("tb-apps").unwrap();
        let btn = self.document.create_element("button").unwrap();
        btn.set_attribute("id", &format!("tb-btn-{}", id)).unwrap();
        btn.set_attribute("class", "tb-app-btn").unwrap();
        btn.set_inner_html(&format!("⬜ {}", title));
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
                child
                    .class_list()
                    .remove_1("active")
                    .unwrap();
            }
        }
        let btn_id = format!("tb-btn-{}", id);
        if let Some(btn) = self.document.get_element_by_id(&btn_id) {
            btn.class_list().add_1("active").unwrap();
        }
    }

    /// Wire up the Start button toggle and start menu items.
    pub fn setup_start_button(&self) {
        let doc = &self.document;

        // --- Start button: toggle menu, stop propagation so document handler doesn't close it ---
        let doc1 = doc.clone();
        let start_btn = doc.get_element_by_id("start-btn").unwrap();
        let toggle_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |evt: web_sys::MouseEvent| {
                evt.stop_propagation();
                if let Some(menu) = doc1.get_element_by_id("start-menu") {
                    let style = menu.get_attribute("style").unwrap_or_default();
                    let hidden = style.contains("display:none") || style.is_empty();
                    menu.set_attribute(
                        "style",
                        if hidden { "display:block" } else { "display:none" },
                    )
                    .unwrap();
                }
            },
        );
        start_btn
            .add_event_listener_with_callback("click", toggle_cb.as_ref().unchecked_ref())
            .unwrap();
        toggle_cb.forget();

        // --- Document click: close menu when clicking outside ---
        let doc2 = doc.clone();
        let close_cb = Closure::<dyn FnMut()>::new(move || {
            if let Some(menu) = doc2.get_element_by_id("start-menu") {
                menu.set_attribute("style", "display:none").unwrap();
            }
        });
        doc.add_event_listener_with_callback("click", close_cb.as_ref().unchecked_ref())
            .unwrap();
        close_cb.forget();

        // --- Wire each .start-menu-item by iterating children of #start-menu ---
        let doc3 = doc.clone();
        if let Some(start_menu) = doc.get_element_by_id("start-menu") {
            let children = start_menu.children();
            for i in 0..children.length() {
                if let Some(child) = children.item(i) {
                    // Only handle items with the start-menu-item class
                    if child.class_list().contains("start-menu-item") {
                        let item_doc = doc3.clone();
                        let item_el = child.clone();
                        let item_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
                            move |evt: web_sys::MouseEvent| {
                                evt.stop_propagation();
                                if let Some(app) = item_el.get_attribute("data-app") {
                                    // Close the menu first
                                    if let Some(menu) = item_doc.get_element_by_id("start-menu") {
                                        menu.set_attribute("style", "display:none").unwrap();
                                    }
                                    launch_app(&item_doc, &app);
                                }
                            },
                        );
                        child
                            .add_event_listener_with_callback(
                                "click",
                                item_cb.as_ref().unchecked_ref(),
                            )
                            .unwrap();
                        item_cb.forget();
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------
// App launcher
// ---------------------------------------------------------------
pub fn launch_app(document: &web_sys::Document, app: &str) {
    match app {
        "file-manager" => {
            crate::file_manager::FileManager::open();
        }
        "terminal" => {
            crate::terminal::Terminal::open(document);
        }
        "projects" => {
            crate::projects_gallery::ProjectsGallery::open(document);
        }
        "contact" => {
            crate::contact::ContactApp::open(document);
        }
        "shutdown" => trigger_shutdown(document),
        _ => {
            // Placeholder: create a simple window for the app
            crate::app_state::with_wm(|wm| {
                wm.create_window(app, app, 400, 300);
            });
        }
    }
}

// ---------------------------------------------------------------
// Shutdown animation
// ---------------------------------------------------------------
pub fn trigger_shutdown(document: &web_sys::Document) {
    // Create full-screen overlay
    let overlay = document.create_element("div").unwrap();
    overlay.set_attribute("id", "shutdown-overlay").unwrap();
    overlay
        .set_attribute(
            "style",
            "position:fixed;inset:0;z-index:99999;background:#000080;\
             display:flex;align-items:center;justify-content:center;\
             flex-direction:column;gap:16px;",
        )
        .unwrap();
    overlay.set_inner_html(
        "<div style='color:white;font-size:18px;'>\
         Please wait while your computer shuts down.\
         </div>\
         <div id='shutdown-counter' style='color:white;font-size:24px;'>5</div>",
    );
    document.body().unwrap().append_child(&overlay).unwrap();

    // Countdown timer
    let window = web_sys::window().unwrap();
    let doc = document.clone();

    let cb_holder: Rc<RefCell<Option<Closure<dyn FnMut()>>>> =
        Rc::new(RefCell::new(None));
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
                // Update the counter display
                if let Some(el) = doc.get_element_by_id("shutdown-counter") {
                    el.set_inner_html(&count.to_string());
                }
                // Schedule the next tick
                if let Some(cb_ref) = cb_holder.borrow().as_ref() {
                    let cb_fn = cb_ref
                        .as_ref()
                        .unchecked_ref::<js_sys::Function>()
                        .clone();
                    let _ = window
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            &cb_fn, 1000,
                        );
                }
            } else {
                // Show "safe to turn off" screen
                if let Some(el) = doc.get_element_by_id("shutdown-overlay") {
                    el.set_inner_html(
                        "<div style='color:white;font-size:24px;'>\
                         It's now safe to turn off your computer.\
                         </div>",
                    );
                    // Click anywhere to reload
                    let reload_cb = Closure::<dyn FnMut()>::new(move || {
                        if let Some(w) = web_sys::window() {
                            w.location().reload().ok();
                        }
                    });
                    el.add_event_listener_with_callback(
                        "click",
                        reload_cb.as_ref().unchecked_ref(),
                    )
                    .unwrap();
                    reload_cb.forget();
                }
                // Release the closure so it can be dropped
                *cb_holder.borrow_mut() = None;
            }
        })
    };

    let cb_fn = closure
        .as_ref()
        .unchecked_ref::<js_sys::Function>()
        .clone();
    *cb_holder.borrow_mut() = Some(closure);
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&cb_fn, 1000);
}
