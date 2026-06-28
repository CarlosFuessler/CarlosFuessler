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

    pub fn create_window(&mut self, app_id: &str, title: &str, w: u32, h: u32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let z = self.next_z;
        self.next_z += 1;

        let offset = (id % 10) * 20;
        let x = 100 + offset as i32;
        let y = 60 + offset as i32;

        // --- Build DOM ---
        let window_el = self.document.create_element("div").unwrap();
        window_el.set_attribute("class", "win95-window").unwrap();
        window_el
            .set_attribute("id", &format!("win-{}", id))
            .unwrap();
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

        // Minimize button
        let min_btn = self.document.create_element("button").unwrap();
        min_btn.set_attribute("class", "win95-title-btn").unwrap();
        min_btn.set_inner_html("_");
        titlebar.append_child(&min_btn).unwrap();

        // Maximize button
        let max_btn = self.document.create_element("button").unwrap();
        max_btn.set_attribute("class", "win95-title-btn").unwrap();
        max_btn.set_inner_html("□");
        titlebar.append_child(&max_btn).unwrap();

        // Close button
        let close_btn = self.document.create_element("button").unwrap();
        close_btn.set_attribute("class", "win95-title-btn").unwrap();
        close_btn.set_inner_html("✕");
        titlebar.append_child(&close_btn).unwrap();

        window_el.append_child(&titlebar).unwrap();

        // Content area
        let content = self.document.create_element("div").unwrap();
        content.set_attribute("class", "win95-content").unwrap();
        window_el.append_child(&content).unwrap();

        // Append to desktop *before* wiring events so element is in DOM
        self.desktop.append_child(&window_el).unwrap();

        // Wire drag on titlebar
        self.wire_drag(id, &titlebar);

        // Wire title buttons
        self.wire_title_buttons(id, &close_btn, &min_btn, &max_btn);

        // Store window struct
        let win = Window {
            id,
            app_id: app_id.to_string(),
            title: title.to_string(),
            x,
            y,
            w,
            h,
            prev_x: x,
            prev_y: y,
            prev_w: w,
            prev_h: h,
            z_index: z,
            state: WindowState::Open,
            element: window_el,
            content,
        };
        self.windows.push(win);

        // Focus the new window
        self.focus_window(id);

        // Add taskbar button
        app_state::with_taskbar(|tb| tb.add_app_button(&format!("win-{}", id), title));

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
        app_state::with_taskbar(|tb| tb.remove_app_button(&format!("win-{}", id)));
    }

    pub fn focus_window(&mut self, id: u32) {
        self.next_z += 1;
        let z = self.next_z;

        if let Some(win) = self.find_window_mut(id) {
            win.z_index = z;
            let style = format!(
                "left:{}px;top:{}px;width:{}px;height:{}px;z-index:{}",
                win.x, win.y, win.w, win.h, z
            );
            let _ = win.element.set_attribute("style", &style);
        }

        // Update titlebar active/inactive classes
        for w in &self.windows {
            if let Some(titlebar) = w.element.children().item(0) {
                if w.id == id {
                    let _ = titlebar.class_list().remove_1("inactive");
                } else {
                    let _ = titlebar.class_list().add_1("inactive");
                }
            }
        }

        app_state::with_taskbar(|tb| tb.set_active(&format!("win-{}", id)));
    }

    pub fn minimize_window(&mut self, id: u32) {
        if let Some(win) = self.find_window_mut(id) {
            win.state = WindowState::Minimized;
            let _ = win.element.set_attribute("style", "display:none");
        }
    }

    pub fn maximize_window(&mut self, id: u32) {
        // Read desktop dimensions first, before any mutable borrow on self.windows
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
                    // Restore to previous position/size
                    win.state = WindowState::Open;
                    win.x = win.prev_x;
                    win.y = win.prev_y;
                    win.w = win.prev_w;
                    win.h = win.prev_h;
                }
                _ => {
                    // Save current position/size
                    win.prev_x = win.x;
                    win.prev_y = win.y;
                    win.prev_w = win.w;
                    win.prev_h = win.h;

                    // Maximize to fill desktop
                    win.w = desktop_w;
                    win.h = desktop_h;
                    win.x = 0;
                    win.y = 0;
                    win.state = WindowState::Maximized;
                }
            }
            // Apply updated position/size
            let style = format!(
                "left:{}px;top:{}px;width:{}px;height:{}px;z-index:{}",
                win.x, win.y, win.w, win.h, win.z_index
            );
            let _ = win.element.set_attribute("style", &style);
        }
    }

    // ---------------------------------------------------------------
    // Drag: mousedown on titlebar, mousemove/mouseup on document
    // ---------------------------------------------------------------
    fn wire_drag(&self, id: u32, titlebar: &web_sys::Element) {
        let is_dragging = Rc::new(RefCell::new(false));
        let start_x = Rc::new(RefCell::new(0i32));
        let start_y = Rc::new(RefCell::new(0i32));
        let win_start_x = Rc::new(RefCell::new(0i32));
        let win_start_y = Rc::new(RefCell::new(0i32));
        let doc = self.document.clone();
        let win_id = id;

        // --- mousedown on titlebar ---
        let d_is_dragging = is_dragging.clone();
        let d_start_x = start_x.clone();
        let d_start_y = start_y.clone();
        let d_win_start_x = win_start_x.clone();
        let d_win_start_y = win_start_y.clone();
        let d_doc = doc.clone();
        let d_win_id = win_id;

        let mousedown_cb =
            Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
                ev.prevent_default();
                ev.stop_propagation();
                *d_is_dragging.borrow_mut() = true;
                *d_start_x.borrow_mut() = ev.client_x();
                *d_start_y.borrow_mut() = ev.client_y();
                if let Some(el) = d_doc.get_element_by_id(&format!("win-{}", d_win_id)) {
                    let rect = el.get_bounding_client_rect();
                    *d_win_start_x.borrow_mut() = rect.left() as i32;
                    *d_win_start_y.borrow_mut() = rect.top() as i32;
                }
                app_state::with_wm(|wm| wm.focus_window(d_win_id));
            });
        titlebar
            .add_event_listener_with_callback(
                "mousedown",
                mousedown_cb.as_ref().unchecked_ref(),
            )
            .unwrap();
        mousedown_cb.forget();

        // --- mousemove on document ---
        let m_is_dragging = is_dragging.clone();
        let m_start_x = start_x.clone();
        let m_start_y = start_y.clone();
        let m_win_start_x = win_start_x.clone();
        let m_win_start_y = win_start_y.clone();
        let m_win_id = win_id;

        let mousemove_cb =
            Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
                if !*m_is_dragging.borrow() {
                    return;
                }
                let dx = ev.client_x() - *m_start_x.borrow();
                let dy = ev.client_y() - *m_start_y.borrow();
                let new_x = *m_win_start_x.borrow() + dx;
                let new_y = *m_win_start_y.borrow() + dy;

                app_state::with_wm(|wm| {
                    if wm.set_window_position(m_win_id, new_x, new_y).is_err() {
                        // Window was closed during drag — stop tracking
                        *m_is_dragging.borrow_mut() = false;
                    }
                });
            });
        doc.add_event_listener_with_callback(
            "mousemove",
            mousemove_cb.as_ref().unchecked_ref(),
        )
        .unwrap();
        mousemove_cb.forget();

        // --- mouseup on document ---
        let u_is_dragging = is_dragging.clone();
        let mouseup_cb =
            Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |_ev: web_sys::MouseEvent| {
                *u_is_dragging.borrow_mut() = false;
            });
        doc.add_event_listener_with_callback(
            "mouseup",
            mouseup_cb.as_ref().unchecked_ref(),
        )
        .unwrap();
        mouseup_cb.forget();
    }

    // ---------------------------------------------------------------
    // Title button wiring
    // ---------------------------------------------------------------
    fn wire_title_buttons(
        &self,
        id: u32,
        close_btn: &web_sys::Element,
        min_btn: &web_sys::Element,
        max_btn: &web_sys::Element,
    ) {
        // Close
        let close_cb = Closure::<dyn FnMut()>::new({
            let win_id = id;
            move || {
                app_state::with_wm(|wm| wm.close_window(win_id));
            }
        });
        close_btn
            .add_event_listener_with_callback("click", close_cb.as_ref().unchecked_ref())
            .unwrap();
        close_cb.forget();

        // Minimize
        let min_cb = Closure::<dyn FnMut()>::new({
            let win_id = id;
            move || {
                app_state::with_wm(|wm| wm.minimize_window(win_id));
            }
        });
        min_btn
            .add_event_listener_with_callback("click", min_cb.as_ref().unchecked_ref())
            .unwrap();
        min_cb.forget();

        // Maximize
        let max_cb = Closure::<dyn FnMut()>::new({
            let win_id = id;
            move || {
                app_state::with_wm(|wm| wm.maximize_window(win_id));
            }
        });
        max_btn
            .add_event_listener_with_callback("click", max_cb.as_ref().unchecked_ref())
            .unwrap();
        max_cb.forget();
    }

    // ---------------------------------------------------------------
    // Internal helpers
    // ---------------------------------------------------------------
    fn find_window_mut(&mut self, id: u32) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == id)
    }

    /// Updates a window's x/y in-place (used by the drag handler).
    /// Returns Ok(()) on success, Err(()) if the window no longer exists.
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
}
