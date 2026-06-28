use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod app_state;
mod desktop;
mod file_manager;
mod markdown;
mod login;
mod taskbar;
mod vfs;
mod terminal;
mod window_manager;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    body.set_inner_html(include_str!("../templates/app.html"));
    show_boot_screen(&document);

    Ok(())
}

fn show_boot_screen(document: &web_sys::Document) {
    let fill = document.get_element_by_id("boot-progress-fill").unwrap();
    let text = document.get_element_by_id("boot-text").unwrap();
    let window = web_sys::window().unwrap();
    let doc = document.clone();

    // Hold the Closure in an Rc so it stays alive for recursive setTimeout calls
    let cb_holder: Rc<RefCell<Option<Closure<dyn FnMut()>>>> =
        Rc::new(RefCell::new(None));
    let progress: Rc<RefCell<f64>> = Rc::new(RefCell::new(0.0));

    let closure = {
        let fill = fill.clone();
        let text = text.clone();
        let doc = doc.clone();
        let window = window.clone();
        let cb_holder = cb_holder.clone();
        let progress = progress.clone();

        Closure::<dyn FnMut()>::new(move || {
            let mut p = progress.borrow_mut();
            *p += 20.0;
            if *p > 100.0 {
                *p = 100.0;
            }

            let _ = fill.set_attribute(
                "style",
                &format!(
                    "width:{}%;background:#000080;height:100%;transition:width 0.3s linear",
                    *p
                ),
            );

            if *p >= 100.0 {
                text.set_inner_html("Preparing desktop...");
                login::show_login(&doc);
                // Release the closure so it can be dropped
                *cb_holder.borrow_mut() = None;
            } else {
                // Re-schedule next step
                if let Some(cb_ref) = cb_holder.borrow().as_ref() {
                    let cb_fn =
                        cb_ref.as_ref().unchecked_ref::<js_sys::Function>().clone();
                    let _ = window
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            &cb_fn, 600,
                        );
                }
            }
        })
    };

    let cb_fn = closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
    *cb_holder.borrow_mut() = Some(closure);
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&cb_fn, 600);
}
