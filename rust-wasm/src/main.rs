use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let app = document.get_element_by_id("app").unwrap();

    let el = document.create_element("div")?;
    el.set_inner_html("<h1>Win95 OS Desktop — loading...</h1>");
    app.append_child(&el)?;

    Ok(())
}
