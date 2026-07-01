use std::cell::RefCell;

const COLORS: &[&str] = &[
    "#000000", "#ffffff", "#ff0000", "#00ff00", "#0000ff",
    "#ffff00", "#ff00ff", "#00ffff", "#808080", "#c0c0c0",
    "#800000", "#008000", "#000080", "#808000", "#800080", "#008080"
];

thread_local! {
    static PAINT_TOOL: RefCell<String> = const { RefCell::new(String::new()) };
    static PAINT_COLOR: RefCell<String> = const { RefCell::new(String::new()) };
    static PAINT_SIZE: RefCell<u32> = const { RefCell::new(1) };
    static IS_DRAWING: RefCell<bool> = const { RefCell::new(false) };
}

pub struct PaintApp;

impl PaintApp {
    pub fn open(document: &web_sys::Document) {
        let id = crate::app_state::create_window("paint", "Paint", 600, 450);
        let content = crate::app_state::with_wm(|wm| wm.get_content(id)).unwrap();
        create(&content, 600, 450);
    }
}

pub fn create(parent: &web_sys::Element, _w: u32, _h: u32) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    parent.set_attribute("style", "padding:0;display:flex;flex-direction:column;background:var(--silver);height:100%;box-sizing:border-box;overflow:hidden;").unwrap();

    // Reset state on open
    PAINT_TOOL.with(|t| *t.borrow_mut() = "pencil".to_string());
    PAINT_COLOR.with(|c| *c.borrow_mut() = "#000000".to_string());
    PAINT_SIZE.with(|s| *s.borrow_mut() = 1);
    IS_DRAWING.with(|d| *d.borrow_mut() = false);

    // Toolbar
    let toolbar = parent.owner_document().unwrap().create_element("div").unwrap();
    toolbar.set_attribute("class", "paint-toolbar").unwrap();
    toolbar.set_inner_html(
        "<button class='paint-tbtn paint-tool-btn active' data-tool='pencil'><svg class='icon-svg' viewBox='0 0 16 16'><path fill='#fff' stroke='#000' stroke-width='1.2' d='M11,2 l3,3 l-9,9 l-3,1 l1,-3 z'/></svg> Pencil</button> \
         <button class='paint-tbtn paint-tool-btn' data-tool='brush'><svg class='icon-svg' viewBox='0 0 16 16'><path fill='#fff' stroke='#000' d='M12,2 l2,2 l-4,4 l-2,-2 z M8,8 l-4,4 c-1,1 -2,1 -2,2 c0,0 1,-1 2,-2 l4,-4 z'/></svg> Brush</button> \
         <button class='paint-tbtn paint-tool-btn' data-tool='eraser'><svg class='icon-svg' viewBox='0 0 16 16'><rect x='2' y='2' width='12' height='12' rx='2' fill='none' stroke='#000' stroke-width='1.5'/><line x1='2' y1='8' x2='14' y2='8' stroke='#000'/></svg> Eraser</button> \
         <button class='paint-tbtn' data-tool='clear'><svg class='icon-svg' viewBox='0 0 16 16'><path fill='none' stroke='#000' stroke-width='1.5' d='M3,4 h10 M5,4 v10 h6 v-10 M7,2 h2'/></svg> Clear</button> \
         <span style='border-left:1px inset var(--silver-dark);margin:0 4px;height:20px;'></span> \
         <button class='paint-tbtn paint-size-btn active' data-size='1'>1px</button> \
         <button class='paint-tbtn paint-size-btn' data-size='3'>3px</button> \
         <button class='paint-tbtn paint-size-btn' data-size='5'>5px</button>"
    );
    parent.append_child(&toolbar).unwrap();

    // Color palette
    let palette = parent.owner_document().unwrap().create_element("div").unwrap();
    palette.set_attribute("class", "paint-palette").unwrap();
    let mut pal_html = String::new();
    for (i, color) in COLORS.iter().enumerate() {
        let selected = if i == 0 { " selected" } else { "" };
        pal_html.push_str(&format!(
            r#"<div class="paint-color{}" data-color="{}" style="background:{};"></div>"#,
            selected, color, color
        ));
    }
    palette.set_inner_html(&pal_html);
    parent.append_child(&palette).unwrap();

    // Canvas container
    let canvas_el = parent.owner_document().unwrap().create_element("canvas").unwrap();
    canvas_el.set_attribute("id", "paint-canvas").unwrap();
    canvas_el.set_attribute("class", "paint-canvas").unwrap();
    parent.append_child(&canvas_el).unwrap();

    let canvas: web_sys::HtmlCanvasElement = canvas_el.dyn_into().unwrap();
    let ctx = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

    // Fill canvas white initially
    ctx.set_fill_style(&"#ffffff".into());
    ctx.fill_rect(0.0, 0.0, 800.0, 600.0); // Use large default bounds, set_size will adjust

    let doc = parent.owner_document().unwrap();

    // Tool/color/size click handler
    let c = canvas.clone();
    let ct = ctx.clone();
    let tb = toolbar.clone();
    let tl_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
        move |ev: web_sys::MouseEvent| {
            ev.stop_propagation();
            if let Some(target) = ev.target() {
                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                    if el.class_list().contains("paint-tbtn") {
                        if let Some(tool) = el.get_attribute("data-tool") {
                            match tool.as_str() {
                                "clear" => {
                                    ct.set_fill_style(&"#ffffff".into());
                                    ct.fill_rect(0.0, 0.0, c.width() as f64, c.height() as f64);
                                }
                                _ => {
                                    PAINT_TOOL.with(|t| *t.borrow_mut() = tool.clone());
                                    // Update active tool button styling
                                    let children = tb.children();
                                    for i in 0..children.length() {
                                        if let Some(child) = children.item(i) {
                                            if child.class_list().contains("paint-tool-btn") {
                                                if child.get_attribute("data-tool").as_deref() == Some(&tool) {
                                                    child.class_list().add_1("active").ok();
                                                } else {
                                                    child.class_list().remove_1("active").ok();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else if let Some(size_str) = el.get_attribute("data-size") {
                            if let Ok(sz) = size_str.parse::<u32>() {
                                PAINT_SIZE.with(|s| *s.borrow_mut() = sz);
                                // Update active size button styling
                                let children = tb.children();
                                for i in 0..children.length() {
                                    if let Some(child) = children.item(i) {
                                        if child.class_list().contains("paint-size-btn") {
                                            if child.get_attribute("data-size").as_deref() == Some(&size_str) {
                                                child.class_list().add_1("active").ok();
                                            } else {
                                                child.class_list().remove_1("active").ok();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if el.class_list().contains("paint-color") {
                        if let Some(color) = el.get_attribute("data-color") {
                            PAINT_COLOR.with(|c| *c.borrow_mut() = color.to_string());
                            // Update selected class in palette
                            if let Some(pal) = el.parent_element() {
                                let children = pal.children();
                                for i in 0..children.length() {
                                    if let Some(child) = children.item(i) {
                                        child.class_list().remove_1("selected").ok();
                                    }
                                }
                            }
                            el.class_list().add_1("selected").ok();
                        }
                    }
                }
            }
        }
    );
    parent.add_event_listener_with_callback("click", tl_cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(tl_cb);

    // Mouse handlers on canvas
    let c2 = canvas.clone();
    let ctx2 = ctx.clone();
    let md_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
        move |ev: web_sys::MouseEvent| {
            IS_DRAWING.with(|d| *d.borrow_mut() = true);
            let rect = c2.get_bounding_client_rect();
            let x = ev.client_x() as f64 - rect.left();
            let y = ev.client_y() as f64 - rect.top();
            let is_eraser = PAINT_TOOL.with(|t| t.borrow().clone() == "eraser");
            let size = PAINT_SIZE.with(|s| *s.borrow());
            
            ctx2.begin_path();
            if is_eraser {
                ctx2.set_fill_style(&"#ffffff".into());
                ctx2.fill_rect(x - size as f64 / 2.0, y - size as f64 / 2.0, size as f64, size as f64);
            } else {
                let color = PAINT_COLOR.with(|c| c.borrow().clone());
                ctx2.set_fill_style(&(&color).into());
                ctx2.set_stroke_style(&(&color).into());
                let tool = PAINT_TOOL.with(|t| t.borrow().clone());
                let radius = if tool == "brush" { size as f64 * 1.5 } else { size as f64 * 0.5 };
                let _ = ctx2.arc(x, y, radius, 0.0, std::f64::consts::TAU);
                ctx2.fill();
            }
        }
    );
    canvas.add_event_listener_with_callback("mousedown", md_cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(md_cb);

    let c3 = canvas.clone();
    let ctx3 = ctx.clone();
    let mm_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
        move |ev: web_sys::MouseEvent| {
            if !IS_DRAWING.with(|d| *d.borrow()) { return; }
            let rect = c3.get_bounding_client_rect();
            let x = ev.client_x() as f64 - rect.left();
            let y = ev.client_y() as f64 - rect.top();
            let is_eraser = PAINT_TOOL.with(|t| t.borrow().clone() == "eraser");
            let size = PAINT_SIZE.with(|s| *s.borrow());
            
            if is_eraser {
                ctx3.set_fill_style(&"#ffffff".into());
                ctx3.fill_rect(x - size as f64 / 2.0, y - size as f64 / 2.0, size as f64, size as f64);
            } else {
                let color = PAINT_COLOR.with(|c| c.borrow().clone());
                ctx3.set_fill_style(&(&color).into());
                ctx3.set_stroke_style(&(&color).into());
                ctx3.begin_path();
                let tool = PAINT_TOOL.with(|t| t.borrow().clone());
                let radius = if tool == "brush" { size as f64 * 1.5 } else { size as f64 * 0.5 };
                let _ = ctx3.arc(x, y, radius, 0.0, std::f64::consts::TAU);
                ctx3.fill();
            }
        }
    );
    canvas.add_event_listener_with_callback("mousemove", mm_cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(mm_cb);

    let mu_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
        move |_ev: web_sys::MouseEvent| {
            IS_DRAWING.with(|d| *d.borrow_mut() = false);
        }
    );
    doc.add_event_listener_with_callback("mouseup", mu_cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(mu_cb);

    // Set canvas size on next tick
    let p_clone = parent.clone();
    let set_size = Closure::<dyn FnMut()>::new(move || {
        let w = p_clone.client_width() as u32;
        let h = p_clone.client_height() as u32 - 64; // toolbar + palette
        canvas.set_width(w.max(200));
        canvas.set_height(h.max(100));
        ctx.set_fill_style(&"#ffffff".into());
        ctx.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    });
    if let Some(w) = web_sys::window() {
        let fn_ref = set_size.as_ref().unchecked_ref::<js_sys::Function>().clone();
        let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(&fn_ref, 50);
    }
    crate::app_state::store_closure(set_size);
}
