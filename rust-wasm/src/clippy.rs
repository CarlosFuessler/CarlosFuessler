use std::cell::RefCell;
use std::rc::Rc;
use serde::Deserialize;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

#[derive(Deserialize, Clone, Debug)]
pub struct ClippyStep {
    pub text: String,
    pub btn: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ClippyDialogues {
    pub step0: ClippyStep,
    pub step1: ClippyStep,
    pub step2: ClippyStep,
}

struct ClippyState {
    step: u32,
    closed: bool,
    dialogues: Option<ClippyDialogues>,
}

thread_local! {
    static STATE: RefCell<ClippyState> = const { RefCell::new(ClippyState {
        step: 0,
        closed: false,
        dialogues: None,
    })};
}

pub fn init() {
    let document = web_sys::window().unwrap().document().unwrap();
    if document.get_element_by_id("clippy").is_some() {
        return;
    }

    // Load dialogues asynchronously
    wasm_bindgen_futures::spawn_local(async move {
        let dialogues = fetch_dialogues().await;
        STATE.with(|s| {
            s.borrow_mut().dialogues = dialogues;
        });
        
        // Render Clippy once dialogues are loaded
        render_clippy();
    });
}

async fn fetch_dialogues() -> Option<ClippyDialogues> {
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window()?;
    let promise = window.fetch_with_str("/content/clippy.json");
    let resp_val = JsFuture::from(promise).await.ok()?;
    let resp: web_sys::Response = resp_val.dyn_into().ok()?;
    
    let text_promise = resp.text().ok()?;
    let text_val = JsFuture::from(text_promise).await.ok()?;
    let text = text_val.as_string()?;
    
    serde_json::from_str(&text).ok()
}

fn render_clippy() {
    let document = web_sys::window().unwrap().document().unwrap();
    let desktop = match document.get_element_by_id("desktop") {
        Some(d) => d,
        None => return,
    };

    let clippy_el = document.create_element("div").unwrap();
    clippy_el.set_attribute("id", "clippy").unwrap();
    
    // Initial position: Next to My PC icon (top-left)
    clippy_el.set_attribute("style", "\
      position:absolute;z-index:99999;display:flex;align-items:flex-end;gap:12px;pointer-events:none;\
      top:30px;left:110px;\
      transition: top 0.8s cubic-bezier(0.25, 0.8, 0.25, 1), \
                  left 0.8s cubic-bezier(0.25, 0.8, 0.25, 1), \
                  bottom 0.8s cubic-bezier(0.25, 0.8, 0.25, 1), \
                  transform 0.8s cubic-bezier(0.25, 0.8, 0.25, 1);\
    ").unwrap();

    let initial_text = STATE.with(|s| {
        s.borrow().dialogues.as_ref()
            .map(|d| d.step0.text.clone())
            .unwrap_or_else(|| "Hi! I'm Clippy. Let's explore Carlos' portfolio! Double-click the <b>My PC</b> icon on the desktop to read the <b>About Me</b> section.".to_string())
    });

    let initial_btn = STATE.with(|s| {
        s.borrow().dialogues.as_ref()
            .map(|d| d.step0.btn.clone())
            .unwrap_or_else(|| "Skip Tour".to_string())
    });

    clippy_el.set_inner_html(&format!(r#"
<img src="/assets/clippy.png" id="clippy-img" style="width:60px;height:auto;pointer-events:auto;image-rendering:pixelated;user-select:none;cursor:grab;" />
<div id="clippy-bubble" style="
  background:#ffffcc;border:1px solid #000;padding:8px 12px;
  border-radius:8px;font-family:var(--font);font-size:11px;
  box-shadow:2px 2px 0 rgba(0,0,0,0.2);max-width:180px;position:relative;
  pointer-events:auto;line-height:1.4;display:flex;flex-direction:column;gap:6px;
  user-select:none;
">
  <div id="clippy-text">{}</div>
  <div style="display:flex;justify-content:flex-end;">
    <button class="win95-btn" id="clippy-action-btn" style="padding:2px 8px;font-size:10px;cursor:pointer;">{}</button>
  </div>
  <div id="clippy-bubble-arrow" style="
    position:absolute;left:-8px;bottom:12px;
    width:0;height:0;
    border-top:6px solid transparent;
    border-bottom:6px solid transparent;
    border-right:8px solid #ffffcc;
  "></div>
  <div style="
    position:absolute;left:-9px;bottom:12px;z-index:-1;
    width:0;height:0;
    border-top:6px solid transparent;
    border-bottom:6px solid transparent;
    border-right:8px solid #000;
  "></div>
</div>
"#, initial_text, initial_btn));

    desktop.append_child(&clippy_el).unwrap();

    // Wire up the button
    let doc = document.clone();
    let btn = doc.get_element_by_id("clippy-action-btn").unwrap();
    let btn_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        close_clippy(&doc);
    });
    btn.add_event_listener_with_callback("click", btn_cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(btn_cb);

    // ---------------------------------------------------------------
    // Drag-to-reposition logic
    // ---------------------------------------------------------------
    let is_dragging = Rc::new(RefCell::new(false));
    let offset_x = Rc::new(RefCell::new(0i32));
    let offset_y = Rc::new(RefCell::new(0i32));

    let d_down = is_dragging.clone();
    let d_ox = offset_x.clone();
    let d_oy = offset_y.clone();
    let d_el = clippy_el.clone();
    let cb_down = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
        // Only trigger drag if clicking on the image or the bubble, not the button
        if let Some(target) = ev.target() {
            if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                if el.id() == "clippy-action-btn" {
                    return;
                }
            }
        }
        
        *d_down.borrow_mut() = true;
        let rect = d_el.get_bounding_client_rect();
        *d_ox.borrow_mut() = ev.client_x() - rect.left() as i32;
        *d_oy.borrow_mut() = ev.client_y() - rect.top() as i32;
        
        // Disable transitions during drag so it follows mouse instantly
        if let Some(html_el) = d_el.dyn_ref::<web_sys::HtmlElement>() {
            html_el.style().set_property("transition", "none").ok();
            if let Some(img) = html_el.query_selector("#clippy-img").unwrap() {
                if let Some(img_el) = img.dyn_ref::<web_sys::HtmlElement>() {
                    img_el.style().set_property("cursor", "grabbing").ok();
                }
            }
        }
    });
    clippy_el.add_event_listener_with_callback("mousedown", cb_down.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(cb_down);

    let m_down = is_dragging.clone();
    let m_ox = offset_x.clone();
    let m_oy = offset_y.clone();
    let m_el = clippy_el.clone();
    let cb_move = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
        if *m_down.borrow() {
            let new_x = ev.client_x() - *m_ox.borrow();
            let new_y = ev.client_y() - *m_oy.borrow();
            if let Some(html_el) = m_el.dyn_ref::<web_sys::HtmlElement>() {
                // When dragging, we must remove 'bottom' so 'top' positioning works
                html_el.style().remove_property("bottom").ok();
                html_el.style().set_property("left", &format!("{}px", new_x)).ok();
                html_el.style().set_property("top", &format!("{}px", new_y)).ok();
            }
        }
    });
    document.add_event_listener_with_callback("mousemove", cb_move.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(cb_move);

    let u_down = is_dragging.clone();
    let u_el = clippy_el.clone();
    let cb_up = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |_ev: web_sys::MouseEvent| {
        if *u_down.borrow() {
            *u_down.borrow_mut() = false;
            
            // Re-enable smooth transition upon release
            if let Some(html_el) = u_el.dyn_ref::<web_sys::HtmlElement>() {
                html_el.style().set_property("transition", "\
                  top 0.8s cubic-bezier(0.25, 0.8, 0.25, 1), \
                  left 0.8s cubic-bezier(0.25, 0.8, 0.25, 1), \
                  bottom 0.8s cubic-bezier(0.25, 0.8, 0.25, 1), \
                  transform 0.8s cubic-bezier(0.25, 0.8, 0.25, 1);\
                ").ok();
                if let Some(img) = html_el.query_selector("#clippy-img").unwrap() {
                    if let Some(img_el) = img.dyn_ref::<web_sys::HtmlElement>() {
                        img_el.style().set_property("cursor", "grab").ok();
                    }
                }
            }
        }
    });
    document.add_event_listener_with_callback("mouseup", cb_up.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(cb_up);
}

pub fn on_app_open(app_id: &str) {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        if state.closed {
            return;
        }

        let document = web_sys::window().unwrap().document().unwrap();
        if state.step == 0 && (app_id == "file-manager" || app_id == "markdown") {
            state.step = 1;
            
            // Update text and button from JSON if available
            let text = state.dialogues.as_ref()
                .map(|d| d.step1.text.clone())
                .unwrap_or_else(|| "Excellent! You opened My PC. Here you can read about Carlos.<br/><br/>Next, let's see his work! Open the <b>Projects Gallery</b> from the Start Menu.".to_string());
            let btn_txt = state.dialogues.as_ref()
                .map(|d| d.step1.btn.clone())
                .unwrap_or_else(|| "Skip Tour".to_string());

            if let Some(text_el) = document.get_element_by_id("clippy-text") {
                text_el.set_inner_html(&text);
            }
            if let Some(btn_el) = document.get_element_by_id("clippy-action-btn") {
                btn_el.set_inner_html(&btn_txt);
            }
            
            // Fly Clippy to the Start button (bottom-left)
            if let Some(clippy) = document.get_element_by_id("clippy") {
                if let Some(html_el) = clippy.dyn_ref::<web_sys::HtmlElement>() {
                    let style = html_el.style();
                    style.remove_property("top").ok();
                    style.set_property("bottom", "55px").ok();
                    style.set_property("left", "20px").ok();
                }
            }
        } else if state.step == 1 && app_id == "projects" {
            state.step = 2;
            
            // Update text and button from JSON if available
            let text = state.dialogues.as_ref()
                .map(|d| d.step2.text.clone())
                .unwrap_or_else(|| "Wow, look at all these cool projects! Carlos has built systems tooling, web apps, and this retro desktop.<br/><br/>Now you're ready to explore on your own. Have fun!".to_string());
            let btn_txt = state.dialogues.as_ref()
                .map(|d| d.step2.btn.clone())
                .unwrap_or_else(|| "Close".to_string());

            if let Some(text_el) = document.get_element_by_id("clippy-text") {
                text_el.set_inner_html(&text);
            }
            if let Some(btn_el) = document.get_element_by_id("clippy-action-btn") {
                btn_el.set_inner_html(&btn_txt);
            }
            
            // Fly Clippy to the center of the screen
            if let Some(clippy) = document.get_element_by_id("clippy") {
                if let Some(html_el) = clippy.dyn_ref::<web_sys::HtmlElement>() {
                    let style = html_el.style();
                    style.remove_property("bottom").ok();
                    style.set_property("top", "50%").ok();
                    style.set_property("left", "50%").ok();
                    style.set_property("transform", "translate(-50%, -50%)").ok();
                }
            }
        }
    });
}

fn close_clippy(document: &web_sys::Document) {
    STATE.with(|s| {
        s.borrow_mut().closed = true;
    });
    if let Some(clippy) = document.get_element_by_id("clippy") {
        clippy.remove();
    }
}
