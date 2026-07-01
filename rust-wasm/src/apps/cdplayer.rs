use std::cell::RefCell;
use wasm_bindgen::JsCast;

const TRACKS: &[&str] = &[
    "Track 1", "Track 2", "Track 3", "Track 4", "Track 5",
    "Track 6", "Track 7", "Track 8", "Track 9", "Track 10",
    "Track 11", "Track 12", "Track 13", "Track 14"
];

struct CdState {
    current_track: usize,
    playing: bool,
    elapsed_secs: u32,
    ejected: bool,
    timer_handle: Option<js_sys::Function>,
}

thread_local! {
    static CD: RefCell<CdState> = const { RefCell::new(CdState {
        current_track: 0,
        playing: false,
        elapsed_secs: 0,
        ejected: false,
        timer_handle: None,
    })};
}

pub struct CDPlayerApp;

impl CDPlayerApp {
    pub fn open(document: &web_sys::Document) {
        let id = crate::app_state::create_window("cdplayer", "CD Player", 300, 140);
        let content = crate::app_state::with_wm(|wm| wm.get_content(id)).unwrap();
        create(&content, 300, 140);
    }
}

pub fn create(parent: &web_sys::Element, _w: u32, _h: u32) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    // Reset state on open
    CD.with(|cd| {
        let mut s = cd.borrow_mut();
        s.current_track = 0;
        s.playing = false;
        s.elapsed_secs = 0;
        s.ejected = false;
        s.timer_handle = None;
    });

    parent.set_attribute("style", "padding:0;background:var(--silver);height:100%;box-sizing:border-box;").unwrap();
    parent.set_inner_html(r##"
<div style="display:flex;flex-direction:column;gap:8px;padding:8px;height:100%;box-sizing:border-box;justify-content:center;">
   <div style="display:flex;align-items:center;gap:8px;">
    <div id="cd-icon" style="display:flex;align-items:center;user-select:none;"><svg class="icon-svg" style="width:24px;height:24px;" viewBox="0 0 16 16"><circle cx="8" cy="8" r="7" fill="#dfdfdf" stroke="#808080" stroke-width="1.5"/><circle cx="8" cy="8" r="3" fill="#c0c0c0" stroke="#808080"/><circle cx="8" cy="8" r="1" fill="#fff"/></svg></div>
    <div id="cd-display" style="
      flex:1;background:#000;color:#00ff00;font-family:'Courier New',monospace;
      font-size:16px;padding:4px 8px;text-align:center;
      border:2px inset var(--silver-dark);min-height:24px;
      display:flex;align-items:center;justify-content:center;
      user-select:none;
    ">No Disc</div>
  </div>
  <div style="display:flex;gap:4px;justify-content:center;">
    <button class="cd-btn" data-action="eject">⏏ Eject</button>
    <button class="cd-btn" data-action="prev">⏮ Prev</button>
    <button class="cd-btn" data-action="play">▶ Play</button>
    <button class="cd-btn" data-action="stop">⏹ Stop</button>
    <button class="cd-btn" data-action="next">⏭ Next</button>
  </div>
</div>
"##);

    let doc = parent.owner_document().unwrap();

    let cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new({
        let d = doc.clone();
        move |ev: web_sys::MouseEvent| {
            ev.stop_propagation();
            if let Some(target) = ev.target() {
                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                    if let Some(action) = el.get_attribute("data-action") {
                        handle_cd_action(&action, &d);
                    }
                }
            }
        }
    });
    parent.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(cb);

    update_cd_display(&doc);
}

fn handle_cd_action(action: &str, doc: &web_sys::Document) {
    match action {
        "eject" => {
            CD.with(|cd| {
                let mut s = cd.borrow_mut();
                s.ejected = !s.ejected;
                if s.ejected {
                    s.playing = false;
                    s.timer_handle = None;
                } else {
                    s.current_track = 0;
                    s.elapsed_secs = 0;
                }
            });
        }
        "play" => {
            CD.with(|cd| {
                let mut s = cd.borrow_mut();
                if s.ejected { return; }
                s.playing = !s.playing;
                if s.playing {
                    start_cd_timer(doc);
                } else {
                    s.timer_handle = None;
                }
            });
        }
        "stop" => {
            CD.with(|cd| {
                let mut s = cd.borrow_mut();
                s.playing = false;
                s.elapsed_secs = 0;
                s.timer_handle = None;
            });
        }
        "prev" => {
            CD.with(|cd| {
                let mut s = cd.borrow_mut();
                if s.ejected { return; }
                if s.current_track > 0 {
                    s.current_track -= 1;
                } else {
                    s.current_track = TRACKS.len() - 1;
                }
                s.elapsed_secs = 0;
            });
        }
        "next" => {
            CD.with(|cd| {
                let mut s = cd.borrow_mut();
                if s.ejected { return; }
                if s.current_track < TRACKS.len() - 1 {
                    s.current_track += 1;
                } else {
                    s.current_track = 0;
                }
                s.elapsed_secs = 0;
            });
        }
        _ => {}
    }
    update_cd_display(doc);
}

fn update_cd_display(doc: &web_sys::Document) {
    if let Some(display) = doc.get_element_by_id("cd-display") {
        CD.with(|cd| {
            let s = cd.borrow();
            if s.ejected {
                display.set_inner_html("No Disc");
                if let Some(icon) = doc.get_element_by_id("cd-icon") {
                    icon.set_inner_html(r##"<svg class="icon-svg" style="width:24px;height:24px;" viewBox="0 0 16 16"><circle cx="8" cy="8" r="7" fill="#dfdfdf" stroke="#808080" stroke-width="1.5"/><circle cx="8" cy="8" r="3" fill="#c0c0c0" stroke="#808080"/><circle cx="8" cy="8" r="1" fill="#fff"/></svg>"##);
                }
            } else {
                let mins = s.elapsed_secs / 60;
                let secs = s.elapsed_secs % 60;
                let play_indicator = if s.playing { "▶" } else { "⏸" };
                display.set_inner_html(&format!(
                    "{} Track {:02} [{:02}:{:02}]",
                    play_indicator, s.current_track + 1, mins, secs
                ));
                if let Some(icon) = doc.get_element_by_id("cd-icon") {
                    icon.set_inner_html(r##"<svg class="icon-svg" style="width:24px;height:24px;" viewBox="0 0 16 16"><circle cx="8" cy="8" r="7" fill="#dfdfdf" stroke="#808080" stroke-width="1.5"/><circle cx="8" cy="8" r="3" fill="#c0c0c0" stroke="#808080"/><circle cx="8" cy="8" r="1" fill="#fff"/></svg>"##);
                }
            }
        });
    }
}

fn start_cd_timer(doc: &web_sys::Document) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    
    let window = web_sys::window().unwrap();
    let d = doc.clone();
    let cb = Closure::<dyn FnMut()>::new(move || {
        let should_continue = CD.with(|cd| {
            let mut s = cd.borrow_mut();
            if !s.playing || s.ejected { return false; }
            s.elapsed_secs += 1;
            
            // Advance tracks every 30 seconds for simulation purposes
            if s.elapsed_secs >= 30 {
                if s.current_track < TRACKS.len() - 1 {
                    s.current_track += 1;
                    s.elapsed_secs = 0;
                } else {
                    s.current_track = 0;
                    s.elapsed_secs = 0;
                }
            }
            true
        });
        update_cd_display(&d);
        if should_continue {
            CD.with(|cd| {
                let s = cd.borrow();
                if let Some(h) = &s.timer_handle {
                    if let Some(w) = web_sys::window() {
                        let fn_ref = h.clone();
                        let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                            fn_ref.unchecked_ref(), 1000
                        );
                    }
                }
            });
        }
    });
    let fn_ref = cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
    crate::app_state::store_closure(cb);
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&fn_ref, 1000);
    CD.with(|cd| {
        cd.borrow_mut().timer_handle = Some(fn_ref);
    });
}
