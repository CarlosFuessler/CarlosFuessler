use crate::app_state;
use crate::desktop::{render_desktop_icons, DesktopIcon};
use crate::taskbar::Taskbar;
use crate::vfs::VirtualFS;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

pub struct LoginResult {
    pub username: String,
    pub is_guest: bool,
}

pub fn show_login(document: &web_sys::Document) {
    let overlay = document.get_element_by_id("login-overlay").unwrap();
    overlay.set_attribute("style", "").unwrap();

    // Auto-focus the password field
    if let Some(pass_el) = document.get_element_by_id("login-pass") {
        if let Some(input) = pass_el.dyn_ref::<web_sys::HtmlInputElement>() {
            input.focus().ok();
        }
    }

    // OK / Submit button
    let doc_ok = document.clone();
    let ok_btn = document.get_element_by_id("login-ok").unwrap();
    let ok_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
        move |_ev: web_sys::MouseEvent| {
            let pass_elem = doc_ok.get_element_by_id("login-pass").unwrap();
            let pass_input = pass_elem.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
            let password = pass_input.value();
            if password == "123" {
                start_desktop(&doc_ok, LoginResult { username: "Visitor".to_string(), is_guest: false });
            } else {
                if let Some(err_el) = doc_ok.get_element_by_id("login-error") {
                    err_el.set_inner_html("The password is incorrect. Please try again. (Hint: 123)");
                }
                pass_input.set_value("");
                pass_input.focus().ok();
            }
        },
    );
    ok_btn.add_event_listener_with_callback("click", ok_cb.as_ref().unchecked_ref()).unwrap();
    app_state::store_closure(ok_cb);

    // Enter key submits password
    let doc_enter = document.clone();
    let enter_cb = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
        move |ev: web_sys::KeyboardEvent| {
            if ev.key() == "Enter" {
                let pass_elem = doc_enter.get_element_by_id("login-pass").unwrap();
                let pass_input = pass_elem.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
                let password = pass_input.value();
                if password == "123" {
                    start_desktop(&doc_enter, LoginResult { username: "Visitor".to_string(), is_guest: false });
                } else {
                    if let Some(err_el) = doc_enter.get_element_by_id("login-error") {
                        err_el.set_inner_html("The password is incorrect. Please try again. (Hint: 123)");
                    }
                    pass_input.set_value("");
                    pass_input.focus().ok();
                }
            }
        },
    );
    overlay.add_event_listener_with_callback("keydown", enter_cb.as_ref().unchecked_ref()).unwrap();
    app_state::store_closure(enter_cb);

    // Shutdown button
    let doc_sd = document.clone();
    let sd_btn = document.get_element_by_id("login-shutdown").unwrap();
    let sd_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
        move |_ev: web_sys::MouseEvent| {
            if let Some(body) = doc_sd.body() {
                body.set_inner_html("");
                body.style().set_property("background", "#000").ok();
            }
        },
    );
    sd_btn.add_event_listener_with_callback("click", sd_cb.as_ref().unchecked_ref()).unwrap();
    app_state::store_closure(sd_cb);
}

fn start_desktop(document: &web_sys::Document, _result: LoginResult) {
    document.get_element_by_id("login-overlay").unwrap().set_attribute("style", "display:none").unwrap();
    document.get_element_by_id("desktop").unwrap().set_attribute("style", "").unwrap();
    document.get_element_by_id("taskbar").unwrap().set_attribute("style", "").unwrap();

    // Clock
    let now = js_sys::Date::new_0();
    let hours = now.get_hours();
    let minutes = now.get_minutes();
    let am_pm = if hours < 12 { "AM" } else { "PM" };
    let h12 = if hours % 12 == 0 { 12 } else { hours % 12 };
    let time_str = format!("{}:{:02} {}", h12, minutes, am_pm);
    if let Some(clock) = document.get_element_by_id("clock") {
        clock.set_inner_html(&time_str);
    }

    // Desktop icons
    let icons = vec![
        DesktopIcon { id: "my-pc".into(), title: "My PC".into(), icon: r##"<svg class="icon-svg" viewBox="0 0 16 16"><rect x="2" y="2" width="12" height="9" fill="#dfdfdf" stroke="#808080" stroke-width="1.5"/><path fill="#808080" d="M1,12 h14 v2 h-14 z M6,11 h4 v2 h-4 z"/><rect x="3" y="3" width="10" height="7" fill="#008080"/></svg>"## },
        DesktopIcon { id: "recycle-bin".into(), title: "Recycle Bin".into(), icon: r##"<svg class="icon-svg" viewBox="0 0 16 16"><path fill="#dfdfdf" stroke="#808080" stroke-width="1" d="M3,4 h10 l-1,10 h-8 z M2,2 h12 v2 h-12 z M6,2 v-1 h4 v1 z"/><line x1="6" y1="6" x2="6" y2="12" stroke="#808080" stroke-width="1"/><line x1="10" y1="6" x2="10" y2="12" stroke="#808080" stroke-width="1"/></svg>"## },
    ];
    render_desktop_icons(document, &icons);

    // Setup taskbar (wires Start button, menu items, keyboard shortcuts)
    let taskbar = Taskbar::new(document);
    taskbar.setup_start_button();
    app_state::set_taskbar(taskbar);

    // Start Clippy onboarding tour
    crate::clippy::init();

    // Load VFS async
    wasm_bindgen_futures::spawn_local(async move {
        web_sys::console::log_1(&"Loading virtual filesystem...".into());
        let vfs = VirtualFS::load().await;
        app_state::set_vfs(vfs);
        web_sys::console::log_1(&"Virtual filesystem loaded.".into());
    });

}
