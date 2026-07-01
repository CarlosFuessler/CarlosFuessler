use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

pub struct DesktopIcon {
    pub id: String,
    pub title: String,
    pub icon: &'static str,
}

pub fn render_desktop_icons(document: &web_sys::Document, icons: &[DesktopIcon]) {
    let container = document.get_element_by_id("desktop-icons").unwrap();
    container.set_inner_html("");

    for (i, icon) in icons.iter().enumerate() {
        let el = document.create_element("div").unwrap();
        el.set_attribute("class", "desktop-icon").unwrap();
        
        let top = 16 + i * 90;
        el.set_attribute("style", &format!("position: absolute; top: {}px; left: 16px;", top)).unwrap();
        el.set_inner_html(&format!(
            r#"<div class="desktop-icon-img">{}</div><div class="desktop-icon-label">{}</div>"#,
            icon.icon, icon.title
        ));

        // Double-click to launch
        let doc = document.clone();
        let app_id = icon.id.clone();
        let dbl_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |_evt: web_sys::MouseEvent| {
                launch_desktop_app(&doc, &app_id);
            },
        );
        el.add_event_listener_with_callback("dblclick", dbl_cb.as_ref().unchecked_ref()).unwrap();
        crate::app_state::store_closure(dbl_cb);

        // Drag to reposition
        let is_dragging = Rc::new(RefCell::new(false));
        let offset_x = Rc::new(RefCell::new(0i32));
        let offset_y = Rc::new(RefCell::new(0i32));

        let d_down = is_dragging.clone();
        let d_ox = offset_x.clone();
        let d_oy = offset_y.clone();
        let d_el = el.clone();

        let cb_down = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |ev: web_sys::MouseEvent| {
                *d_down.borrow_mut() = true;
                let rect = d_el.get_bounding_client_rect();
                *d_ox.borrow_mut() = ev.client_x() - rect.left() as i32;
                *d_oy.borrow_mut() = ev.client_y() - rect.top() as i32;
                if let Some(html_el) = d_el.dyn_ref::<web_sys::HtmlElement>() {
                    html_el.style().set_property("position", "absolute").ok();
                    html_el.style().set_property("z-index", "9999").ok();
                }
            },
        );
        el.add_event_listener_with_callback("mousedown", cb_down.as_ref().unchecked_ref()).unwrap();
        crate::app_state::store_closure(cb_down);

        let m_down = is_dragging.clone();
        let m_ox = offset_x.clone();
        let m_oy = offset_y.clone();
        let m_el = el.clone();

        let cb_move = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |ev: web_sys::MouseEvent| {
                if !*m_down.borrow() { return; }
                if let Some(html_el) = m_el.dyn_ref::<web_sys::HtmlElement>() {
                    let x = ev.client_x() - *m_ox.borrow();
                    let y = ev.client_y() - *m_oy.borrow();
                    html_el.style().set_property("left", &format!("{}px", x)).ok();
                    html_el.style().set_property("top", &format!("{}px", y)).ok();
                }
            },
        );
        document.add_event_listener_with_callback("mousemove", cb_move.as_ref().unchecked_ref()).unwrap();
        crate::app_state::store_closure(cb_move);

        let u_down = is_dragging.clone();
        let u_el = el.clone();
        let cb_up = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |_ev: web_sys::MouseEvent| {
                *u_down.borrow_mut() = false;
                if let Some(html_el) = u_el.dyn_ref::<web_sys::HtmlElement>() {
                    html_el.style().set_property("z-index", "").ok();
                }
            },
        );
        document.add_event_listener_with_callback("mouseup", cb_up.as_ref().unchecked_ref()).unwrap();
        crate::app_state::store_closure(cb_up);

        container.append_child(&el).unwrap();
    }
}

fn launch_desktop_app(document: &web_sys::Document, id: &str) {
    match id {
        "my-pc" => {
            crate::markdown::MarkdownViewer::open(document, "About Me - Notepad", "content/about/about.md");
            crate::apps::sysinfo::SysInfoApp::open(document);
        }
        "recycle-bin" => {
            let _ = document;
            web_sys::console::log_1(&"Recycle Bin opened (empty)".into());
        }
        _ => {}
    }
}
