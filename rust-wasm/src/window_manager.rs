use crate::app_state;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, PartialEq)]
pub enum WindowState {
    Open,
    Minimized,
    Maximized,
    Closed,
}

pub struct Window {
    pub id: u32,
    pub app_id: String,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub prev_x: i32,
    pub prev_y: i32,
    pub prev_w: u32,
    pub prev_h: u32,
    pub z_index: u32,
    pub state: WindowState,
    pub element: web_sys::Element,
    pub content: web_sys::Element,
}

pub struct WindowManager {
    next_id: u32,
    next_z: u32,
    windows: Vec<Window>,
    document: web_sys::Document,
    desktop: web_sys::Element,
}

impl WindowManager {
    pub fn new(document: web_sys::Document) -> Self {
        let desktop = document
            .get_element_by_id("desktop")
            .expect("desktop element not found");
        WindowManager {
            next_id: 1,
            next_z: 1,
            windows: Vec::new(),
            document,
            desktop,
        }
    }

    pub fn get_content(&self, id: u32) -> Option<web_sys::Element> {
        self.windows
            .iter()
            .find(|w| w.id == id)
            .map(|w| w.content.clone())
    }

    pub fn create_window(&mut self, app_id: &str, title: &str, w: u32, h: u32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let z = self.next_z;
        self.next_z += 1;

        let offset = (id % 10) * 20;
        let x = 100 + offset as i32;
        let y = 60 + offset as i32;

        let window_el = self.document.create_element("div").unwrap();
        window_el.set_attribute("class", "win95-window").unwrap();
        window_el.set_attribute("id", &format!("win-{}", id)).unwrap();
        let style = format!(
            "left:{}px;top:{}px;width:{}px;height:{}px;z-index:{}",
            x, y, w, h, z
        );
        window_el.set_attribute("style", &style).unwrap();

        // Titlebar
        let titlebar = self.document.create_element("div").unwrap();
        titlebar.set_attribute("class", "win95-titlebar").unwrap();

        let title_text = self.document.create_element("span").unwrap();
        title_text.set_inner_html(title);
        titlebar.append_child(&title_text).unwrap();

        let spacer = self.document.create_element("span").unwrap();
        spacer.set_attribute("style", "flex:1").unwrap();
        titlebar.append_child(&spacer).unwrap();

        let min_btn = self.document.create_element("button").unwrap();
        min_btn.set_attribute("class", "win95-title-btn").unwrap();
        min_btn.set_inner_html("_");
        titlebar.append_child(&min_btn).unwrap();

        let max_btn = self.document.create_element("button").unwrap();
        max_btn.set_attribute("class", "win95-title-btn").unwrap();
        max_btn.set_inner_html("□");
        titlebar.append_child(&max_btn).unwrap();

        let close_btn = self.document.create_element("button").unwrap();
        close_btn.set_attribute("class", "win95-title-btn").unwrap();
        close_btn.set_inner_html("✕");
        titlebar.append_child(&close_btn).unwrap();

        window_el.append_child(&titlebar).unwrap();

        let content = self.document.create_element("div").unwrap();
        content.set_attribute("class", "win95-content").unwrap();
        window_el.append_child(&content).unwrap();

        // Resize handle
        let resize_handle = self.document.create_element("div").unwrap();
        resize_handle.set_attribute("style",
            "position:absolute;right:0;bottom:0;width:14px;height:14px;\
             cursor:se-resize;background:linear-gradient(135deg,transparent 0%,transparent 40%,\
             var(--silver-dark) 40%,var(--silver-dark) 50%,transparent 50%,transparent 80%,\
             var(--silver-dark) 80%,var(--silver-dark) 90%,transparent 90%);z-index:1;"
        ).unwrap();
        window_el.append_child(&resize_handle).unwrap();

        self.desktop.append_child(&window_el).unwrap();

        self.wire_drag(id, &titlebar);
        self.wire_resize(id, &resize_handle);
        self.wire_title_buttons(id, &close_btn, &min_btn, &max_btn);

        let win = Window {
            id,
            app_id: app_id.to_string(),
            title: title.to_string(),
            x, y, w, h,
            prev_x: x, prev_y: y,
            prev_w: w, prev_h: h,
            z_index: z,
            state: WindowState::Open,
            element: window_el,
            content,
        };
        self.windows.push(win);
        self.focus_window_inner(id);
        id
    }

    pub fn close_window(&mut self, id: u32) {
        if let Some(idx) = self.windows.iter().position(|w| w.id == id) {
            let win = &self.windows[idx];
            if let Some(parent) = win.element.parent_element() {
                let _ = parent.remove_child(&win.element);
            }
            self.windows.remove(idx);
        }
    }

    pub fn focus_window_inner(&mut self, id: u32) {
        let z = {
            self.next_z += 1;
            self.next_z
        };
        if let Some(win) = self.find_window_mut(id) {
            win.z_index = z;
            let style = format!("left:{}px;top:{}px;width:{}px;height:{}px;z-index:{}", win.x, win.y, win.w, win.h, win.z_index);
            let _ = win.element.set_attribute("style", &style);
        }
        for w in &self.windows {
            if let Some(titlebar) = w.element.children().item(0) {
                if w.id == id {
                    let _ = titlebar.class_list().remove_1("inactive");
                } else {
                    let _ = titlebar.class_list().add_1("inactive");
                }
            }
        }
    }

    pub fn minimize_window(&mut self, id: u32) {
        if let Some(win) = self.find_window_mut(id) {
            win.state = WindowState::Minimized;
            let _ = win.element.set_attribute("style", "display:none");
        }
    }

    pub fn maximize_window(&mut self, id: u32) {
        let desktop_w: u32;
        let desktop_h: u32;
        if let Some(html_el) = self.desktop.dyn_ref::<web_sys::HtmlElement>() {
            desktop_w = html_el.client_width() as u32;
            desktop_h = html_el.client_height() as u32;
        } else {
            let rect = self.desktop.get_bounding_client_rect();
            desktop_w = rect.width() as u32;
            desktop_h = rect.height() as u32;
        }
        if let Some(win) = self.find_window_mut(id) {
            match win.state {
                WindowState::Maximized => {
                    win.state = WindowState::Open;
                    win.x = win.prev_x;
                    win.y = win.prev_y;
                    win.w = win.prev_w;
                    win.h = win.prev_h;
                }
                _ => {
                    win.prev_x = win.x;
                    win.prev_y = win.y;
                    win.prev_w = win.w;
                    win.prev_h = win.h;
                    win.w = desktop_w;
                    win.h = desktop_h;
                    win.x = 0;
                    win.y = 0;
                    win.state = WindowState::Maximized;
                }
            }
            let style = format!("left:{}px;top:{}px;width:{}px;height:{}px;z-index:{}", win.x, win.y, win.w, win.h, win.z_index);
            let _ = win.element.set_attribute("style", &style);
        }
    }

    fn apply_style(&self, win: &Window) {
        let style = format!(
            "left:{}px;top:{}px;width:{}px;height:{}px;z-index:{}",
            win.x, win.y, win.w, win.h, win.z_index
        );
        let _ = win.element.set_attribute("style", &style);
    }

    // ---------------------------------------------------------------
    // Drag
    // ---------------------------------------------------------------
    fn wire_drag(&self, id: u32, titlebar: &web_sys::Element) {
        let is_dragging = Rc::new(RefCell::new(false));
        let start_x = Rc::new(RefCell::new(0i32));
        let start_y = Rc::new(RefCell::new(0i32));
        let win_start_x = Rc::new(RefCell::new(0i32));
        let win_start_y = Rc::new(RefCell::new(0i32));
        let doc = self.document.clone();
        let win_id = id;

        let d_down = is_dragging.clone();
        let d_sx = start_x.clone();
        let d_sy = start_y.clone();
        let d_wx = win_start_x.clone();
        let d_wy = win_start_y.clone();
        let d_doc = doc.clone();
        let d_wid = win_id;

        let cb_down = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |ev: web_sys::MouseEvent| {
                ev.prevent_default();
                ev.stop_propagation();
                *d_down.borrow_mut() = true;
                *d_sx.borrow_mut() = ev.client_x();
                *d_sy.borrow_mut() = ev.client_y();
                if let Some(el) = d_doc.get_element_by_id(&format!("win-{}", d_wid)) {
                    let rect = el.get_bounding_client_rect();
                    *d_wx.borrow_mut() = rect.left() as i32;
                    *d_wy.borrow_mut() = rect.top() as i32;
                }
                app_state::with_wm(|wm| wm.focus_window_inner(d_wid));
                app_state::with_taskbar(|tb| tb.set_active(&format!("win-{}", d_wid)));
            },
        );
        titlebar.add_event_listener_with_callback("mousedown", cb_down.as_ref().unchecked_ref()).unwrap();
        app_state::store_closure(cb_down);

        let m_down = is_dragging.clone();
        let m_sx = start_x.clone();
        let m_sy = start_y.clone();
        let m_wx = win_start_x.clone();
        let m_wy = win_start_y.clone();
        let m_wid = win_id;

        let cb_move = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |ev: web_sys::MouseEvent| {
                if !*m_down.borrow() { return; }
                let dx = ev.client_x() - *m_sx.borrow();
                let dy = ev.client_y() - *m_sy.borrow();
                let nx = *m_wx.borrow() + dx;
                let ny = *m_wy.borrow() + dy;
                app_state::with_wm(|wm| {
                    let _ = wm.set_window_position(m_wid, nx, ny);
                });
            },
        );
        doc.add_event_listener_with_callback("mousemove", cb_move.as_ref().unchecked_ref()).unwrap();
        app_state::store_closure(cb_move);

        let u_down = is_dragging.clone();
        let cb_up = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |_ev: web_sys::MouseEvent| {
                *u_down.borrow_mut() = false;
            },
        );
        doc.add_event_listener_with_callback("mouseup", cb_up.as_ref().unchecked_ref()).unwrap();
        app_state::store_closure(cb_up);
    }

    // ---------------------------------------------------------------
    // Resize
    // ---------------------------------------------------------------
    fn wire_resize(&self, id: u32, handle: &web_sys::Element) {
        let is_resizing = Rc::new(RefCell::new(false));
        let start_x = Rc::new(RefCell::new(0i32));
        let start_y = Rc::new(RefCell::new(0i32));
        let start_w = Rc::new(RefCell::new(0u32));
        let start_h = Rc::new(RefCell::new(0u32));
        let doc = self.document.clone();
        let win_id = id;

        let d_res = is_resizing.clone();
        let d_sx = start_x.clone();
        let d_sy = start_y.clone();
        let d_sw = start_w.clone();
        let d_sh = start_h.clone();
        let d_doc = doc.clone();
        let d_wid = win_id;

        let cb_down = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |ev: web_sys::MouseEvent| {
                ev.prevent_default();
                ev.stop_propagation();
                *d_res.borrow_mut() = true;
                *d_sx.borrow_mut() = ev.client_x();
                *d_sy.borrow_mut() = ev.client_y();
                if let Some(el) = d_doc.get_element_by_id(&format!("win-{}", d_wid)) {
                    let rect = el.get_bounding_client_rect();
                    *d_sw.borrow_mut() = rect.width() as u32;
                    *d_sh.borrow_mut() = rect.height() as u32;
                }
            },
        );
        handle.add_event_listener_with_callback("mousedown", cb_down.as_ref().unchecked_ref()).unwrap();
        app_state::store_closure(cb_down);

        let m_res = is_resizing.clone();
        let m_sx = start_x.clone();
        let m_sy = start_y.clone();
        let m_sw = start_w.clone();
        let m_sh = start_h.clone();
        let m_wid = win_id;

        let cb_move = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |ev: web_sys::MouseEvent| {
                if !*m_res.borrow() { return; }
                let dx = ev.client_x() - *m_sx.borrow();
                let dy = ev.client_y() - *m_sy.borrow();
                let nw = (*m_sw.borrow() as i32 + dx).max(200) as u32;
                let nh = (*m_sh.borrow() as i32 + dy).max(100) as u32;
                app_state::with_wm(|wm| wm.set_window_size(m_wid, nw, nh));
            },
        );
        doc.add_event_listener_with_callback("mousemove", cb_move.as_ref().unchecked_ref()).unwrap();
        app_state::store_closure(cb_move);

        let u_res = is_resizing.clone();
        let cb_up = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |_ev: web_sys::MouseEvent| {
                *u_res.borrow_mut() = false;
            },
        );
        doc.add_event_listener_with_callback("mouseup", cb_up.as_ref().unchecked_ref()).unwrap();
        app_state::store_closure(cb_up);
    }

    // ---------------------------------------------------------------
    // Title buttons
    // ---------------------------------------------------------------
    fn wire_title_buttons(&self, id: u32, close_btn: &web_sys::Element, min_btn: &web_sys::Element, max_btn: &web_sys::Element) {
        let cb_close = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |_ev: web_sys::MouseEvent| {
                app_state::close_window(id);
            },
        );
        close_btn.add_event_listener_with_callback("click", cb_close.as_ref().unchecked_ref()).unwrap();
        app_state::store_closure(cb_close);

        let cb_min = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |_ev: web_sys::MouseEvent| {
                app_state::minimize_window(id);
            },
        );
        min_btn.add_event_listener_with_callback("click", cb_min.as_ref().unchecked_ref()).unwrap();
        app_state::store_closure(cb_min);

        let cb_max = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |_ev: web_sys::MouseEvent| {
                app_state::maximize_window(id);
            },
        );
        max_btn.add_event_listener_with_callback("click", cb_max.as_ref().unchecked_ref()).unwrap();
        app_state::store_closure(cb_max);
    }

    fn find_window_mut(&mut self, id: u32) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == id)
    }

    fn set_window_size(&mut self, id: u32, w: u32, h: u32) {
        if let Some(win) = self.find_window_mut(id) {
            win.w = w;
            win.h = h;
            let style = format!(
                "left:{}px;top:{}px;width:{}px;height:{}px;z-index:{}",
                win.x, win.y, w, h, win.z_index
            );
            let _ = win.element.set_attribute("style", &style);
        }
    }

    fn set_window_position(&mut self, id: u32, x: i32, y: i32) -> Result<(), ()> {
        if let Some(win) = self.find_window_mut(id) {
            win.x = x;
            win.y = y;
            let style = format!(
                "left:{}px;top:{}px;width:{}px;height:{}px;z-index:{}",
                x, y, win.w, win.h, win.z_index
            );
            let _ = win.element.set_attribute("style", &style);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }
}
