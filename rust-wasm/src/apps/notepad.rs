use std::cell::RefCell;

thread_local! {
    static SAVED_TEXT: RefCell<String> = const { RefCell::new(String::new()) };
}

pub struct NotepadApp;

impl NotepadApp {
    pub fn open(document: &web_sys::Document) {
        let id = crate::app_state::create_window("notepad", "Notepad", 500, 400);
        let content = crate::app_state::with_wm(|wm| wm.get_content(id)).unwrap();
        create(&content, 500, 400);
    }
}

pub fn create(parent: &web_sys::Element, _w: u32, _h: u32) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    // Use flex column layout for the parent
    if let Some(html_el) = parent.dyn_ref::<web_sys::HtmlElement>() {
        html_el.style().set_property("display", "flex").ok();
        html_el.style().set_property("flex-direction", "column").ok();
        html_el.style().set_property("height", "100%").ok();
    }

    // Menu bar
    let menu_bar = parent.owner_document().unwrap().create_element("div").unwrap();
    menu_bar.set_attribute("class", "notepad-menubar").unwrap();
    menu_bar.set_inner_html(
        "<span class='notepad-menu-item' data-menuitem='new'>New</span> \
         <span class='notepad-menu-item' data-menuitem='save'>Save</span> \
         <span style='flex:1'></span> \
         <span class='notepad-menu-item' data-menuitem='exit'>Exit</span>"
    );
    parent.append_child(&menu_bar).unwrap();

    // Textarea
    let textarea = parent.owner_document().unwrap().create_element("textarea").unwrap();
    textarea.set_attribute("class", "notepad-textarea").unwrap();
    textarea.set_attribute("id", "notepad-textarea").unwrap();
    
    // Restore saved text
    SAVED_TEXT.with(|s| {
        if let Some(ta_input) = textarea.dyn_ref::<web_sys::HtmlTextAreaElement>() {
            ta_input.set_value(&s.borrow());
        }
    });
    parent.append_child(&textarea).unwrap();

    // Wire menu items
    let doc = parent.owner_document().unwrap();
    let ta = textarea.clone();
    let menu_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
        move |ev: web_sys::MouseEvent| {
            ev.stop_propagation();
            if let Some(target) = ev.target() {
                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                    if let Some(item) = el.get_attribute("data-menuitem") {
                        match item.as_str() {
                            "new" => {
                                if let Some(ta_input) = ta.dyn_ref::<web_sys::HtmlTextAreaElement>() {
                                    ta_input.set_value("");
                                }
                                SAVED_TEXT.with(|s| *s.borrow_mut() = String::new());
                            }
                            "save" => {
                                if let Some(ta_input) = ta.dyn_ref::<web_sys::HtmlTextAreaElement>() {
                                    let text = ta_input.value();
                                    SAVED_TEXT.with(|s| *s.borrow_mut() = text.clone());
                                    // Also save to localStorage
                                    if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                                        let _ = storage.set_item("notepad_text", &text);
                                    }
                                }
                            }
                            "exit" => {
                                // Find parent window and close it
                                let mut current: Option<web_sys::Element> = Some(ta.clone());
                                while let Some(el) = current {
                                    if el.tag_name() == "DIV" && el.class_list().contains("win95-window") {
                                        let id_str = el.id();
                                        if id_str.starts_with("win-") {
                                            if let Ok(id) = id_str[4..].parse::<u32>() {
                                                crate::app_state::close_window(id);
                                            }
                                        }
                                        break;
                                    }
                                    current = el.parent_element();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    );
    menu_bar.add_event_listener_with_callback("click", menu_cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(menu_cb);

    // Load from localStorage
    if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
        if let Ok(Some(text)) = storage.get_item("notepad_text") {
            if let Some(ta_input) = textarea.dyn_ref::<web_sys::HtmlTextAreaElement>() {
                ta_input.set_value(&text);
            }
            SAVED_TEXT.with(|s| *s.borrow_mut() = text);
        }
    }
}
