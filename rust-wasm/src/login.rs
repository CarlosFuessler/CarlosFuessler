use crate::desktop::{render_desktop_icons, DesktopIcon};
use crate::taskbar::Taskbar;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

pub struct LoginResult {
    pub username: String,
    pub is_guest: bool,
}

pub fn show_login(document: &web_sys::Document) {
    let overlay = document.get_element_by_id("login-overlay").unwrap();
    overlay.set_attribute("style", "").unwrap();

    // OK button
    let doc_ok = document.clone();
    let ok_btn = document.get_element_by_id("login-ok").unwrap();
    let ok_cb = Closure::<dyn FnMut()>::new(move || {
        let user_elem = doc_ok
            .get_element_by_id("login-user")
            .unwrap();
        let user_input = user_elem
            .dyn_ref::<web_sys::HtmlInputElement>()
            .unwrap();
        let username = user_input.value();
        if !username.is_empty() {
            start_desktop(
                &doc_ok,
                LoginResult {
                    username,
                    is_guest: false,
                },
            );
        }
    });
    ok_btn
        .add_event_listener_with_callback("click", ok_cb.as_ref().unchecked_ref())
        .unwrap();
    ok_cb.forget();

    // Guest button
    let doc_guest = document.clone();
    let guest_btn = document.get_element_by_id("login-guest").unwrap();
    let guest_cb = Closure::<dyn FnMut()>::new(move || {
        start_desktop(
            &doc_guest,
            LoginResult {
                username: "Guest".to_string(),
                is_guest: true,
            },
        );
    });
    guest_btn
        .add_event_listener_with_callback("click", guest_cb.as_ref().unchecked_ref())
        .unwrap();
    guest_cb.forget();
}

fn start_desktop(document: &web_sys::Document, _result: LoginResult) {
    // Hide login
    document
        .get_element_by_id("login-overlay")
        .unwrap()
        .set_attribute("style", "display:none")
        .unwrap();

    // Show desktop and taskbar
    document
        .get_element_by_id("desktop")
        .unwrap()
        .set_attribute("style", "")
        .unwrap();
    document
        .get_element_by_id("taskbar")
        .unwrap()
        .set_attribute("style", "")
        .unwrap();

    // Set clock to current time
    let now = js_sys::Date::new_0();
    let hours = now.get_hours();
    let minutes = now.get_minutes();
    let am_pm = if hours < 12 { "AM" } else { "PM" };
    let h12 = if hours % 12 == 0 {
        12
    } else {
        hours % 12
    };
    let time_str = format!("{}:{:02} {}", h12, minutes, am_pm);
    if let Some(clock) = document.get_element_by_id("clock") {
        clock.set_inner_html(&time_str);
    }

    // Render desktop icons
    let icons = vec![
        DesktopIcon { id: "my-pc".into(), title: "My PC".into(), icon: "💻" },
        DesktopIcon { id: "projects".into(), title: "Projects".into(), icon: "📁" },
        DesktopIcon { id: "terminal".into(), title: "Terminal".into(), icon: "⬛" },
    ];
    render_desktop_icons(document, &icons);

    // Initialize taskbar
    let _taskbar = Taskbar::new(document);
}
