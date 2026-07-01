use wasm_bindgen::prelude::*;

mod app_state;
mod desktop;
mod file_manager;
mod markdown;
mod projects_gallery;
mod contact;
mod login;
mod taskbar;
mod vfs;
mod terminal;
mod window_manager;
mod apps;
mod clippy;
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    body.set_inner_html(include_str!("../templates/app.html"));
    app_state::init_app_state(document.clone());
    
    // Show login screen immediately
    login::show_login(&document);

    Ok(())
}
