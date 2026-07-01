use std::cell::RefCell;

thread_local! {
    static TAB: RefCell<String> = const { RefCell::new(String::new()) };
}

pub struct SysInfoApp;

impl SysInfoApp {
    pub fn open(document: &web_sys::Document) {
        let id = crate::app_state::create_window("sysinfo", "System Properties", 400, 300);
        let content = crate::app_state::with_wm(|wm| wm.get_content(id)).unwrap();
        create(&content, 400, 300);
    }
}

pub fn create(parent: &web_sys::Element, _w: u32, _h: u32) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    parent.set_attribute("style", "padding:0;overflow:hidden;display:flex;flex-direction:column;height:100%;background:var(--silver);").unwrap();

    // Tab bar
    let tabs = parent.owner_document().unwrap().create_element("div").unwrap();
    tabs.set_attribute("class", "sysinfo-tabs").unwrap();
    tabs.set_inner_html(
        "<span class='sysinfo-tab' data-tab='general'>General</span> \
         <span class='sysinfo-tab' data-tab='hardware'>Hardware</span> \
         <span class='sysinfo-tab' data-tab='performance'>Performance</span>"
    );
    parent.append_child(&tabs).unwrap();

    let body = parent.owner_document().unwrap().create_element("div").unwrap();
    body.set_attribute("class", "sysinfo-body").unwrap();
    parent.append_child(&body).unwrap();

    let b = body.clone();
    let d = parent.owner_document().unwrap();
    
    // Set initial tab
    TAB.with(|t| *t.borrow_mut() = "general".to_string());

    let tab_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
        move |ev: web_sys::MouseEvent| {
            ev.stop_propagation();
            if let Some(target) = ev.target() {
                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                    if let Some(tab) = el.get_attribute("data-tab") {
                        TAB.with(|t| *t.borrow_mut() = tab);
                        render_tab_content(&b, &d);
                    }
                }
            }
        }
    );
    tabs.add_event_listener_with_callback("click", tab_cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(tab_cb);

    render_tab_content(&body, &parent.owner_document().unwrap());
}

fn render_tab_content(body: &web_sys::Element, _doc: &web_sys::Document) {
    use wasm_bindgen::JsCast;
    
    let active_tab = TAB.with(|t| t.borrow().clone());

    let content = match active_tab.as_str() {
        "hardware" => r#"
<h3>Hardware</h3>
<table style="width:100%;border-collapse:collapse;font-size:12px;">
<tr style="height:24px;"><td style="padding:4px;font-weight:bold;width:100px;">CPU:</td><td>WebAssembly (wasm32-unknown-unknown)</td></tr>
<tr style="height:24px;"><td style="padding:4px;font-weight:bold;">RAM:</td><td>4.00 GB (Virtual)</td></tr>
<tr style="height:24px;"><td style="padding:4px;font-weight:bold;">Architecture:</td><td>WASM virtual machine</td></tr>
</table>
"#.to_string(),
        "performance" => {
            let app_count = crate::app_state::with_wm(|wm| wm.window_count());
            format!(r#"
<h3>Performance</h3>
<table style="width:100%;border-collapse:collapse;font-size:12px;">
<tr style="height:24px;"><td style="padding:4px;font-weight:bold;width:120px;">System Status:</td><td>Active</td></tr>
<tr style="height:24px;"><td style="padding:4px;font-weight:bold;">Open Windows:</td><td>{}</td></tr>
<tr style="height:24px;"><td style="padding:4px;font-weight:bold;">Desktop Theme:</td><td>System 95 Classic</td></tr>
<tr style="height:24px;"><td style="padding:4px;font-weight:bold;">Build Target:</td><td>wasm32-unknown-unknown</td></tr>
</table>
"#, app_count)
        }
        _ => r#"
<h3>General</h3>
<table style="width:100%;border-collapse:collapse;font-size:12px;">
<tr style="height:24px;"><td style="padding:4px;font-weight:bold;width:100px;">System:</td><td>System 95</td></tr>
<tr style="height:24px;"><td style="padding:4px;font-weight:bold;">Version:</td><td>95.0.0 (WebAssembly Build)</td></tr>
<tr style="height:24px;"><td style="padding:4px;font-weight:bold;">Registered to:</td><td>Carlos Fuessler</td></tr>
<tr style="height:24px;"><td style="padding:4px;font-weight:bold;">Platform:</td><td>WebAssembly (Rust + wasm-bindgen)</td></tr>
</table>
"#.to_string(),
    };
    
    body.set_inner_html(&content);

    // Update active tab styles
    if let Some(parent_el) = body.parent_element() {
        if let Some(tabs_el) = parent_el.query_selector(".sysinfo-tabs").unwrap() {
            let tab_children = tabs_el.children();
            for i in 0..tab_children.length() {
                if let Some(tab) = tab_children.item(i) {
                    if tab.get_attribute("data-tab").as_deref() == Some(&active_tab) {
                        tab.class_list().add_1("active").ok();
                    } else {
                        tab.class_list().remove_1("active").ok();
                    }
                }
            }
        }
    }
}
