use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

pub struct ProjectsGallery;

impl ProjectsGallery {
    pub fn open(document: &web_sys::Document) {
        let id = crate::app_state::create_window("projects", "Projects Gallery", 650, 450);

        let content = crate::app_state::with_wm(|wm| {
            wm.get_content(id).expect("window content not found")
        });

        content.set_inner_html(r#"
            <div id="gallery-filter" style="padding:4px;border-bottom:1px inset var(--silver-dark);display:flex;gap:4px;flex-wrap:wrap;">
                <button class="win95-btn" data-filter="all">All</button>
                <button class="win95-btn" data-filter="rust">Rust</button>
                <button class="win95-btn" data-filter="web">Web</button>
                <button class="win95-btn" data-filter="systems">Systems</button>
            </div>
            <div id="gallery-cards" style="padding:8px;display:flex;flex-wrap:wrap;gap:12px;overflow-y:auto;height:calc(100% - 42px);align-content:flex-start;">
                <div class="project-card" data-tags="rust web">
                    <h3>System 95 Desktop</h3>
                    <p>A retro 95-themed desktop environment in the browser — built with Rust WASM.</p>
                    <div class="project-tags">Rust, WASM, CSS</div>
                </div>
                <div class="project-card" data-tags="rust systems">
                    <h3>Project Alpha</h3>
                    <p>Systems programming project description.</p>
                    <div class="project-tags">Rust, Systems</div>
                </div>
                <div class="project-card" data-tags="web">
                    <h3>Project Beta</h3>
                    <p>Web development project description.</p>
                    <div class="project-tags">TypeScript, React, Node</div>
                </div>
            </div>
        "#);

        // Wire filter buttons
        let doc = document.clone();
        let filter_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |evt: web_sys::MouseEvent| {
                if let Some(btn) = evt.target() {
                    let btn_el = btn.unchecked_ref::<web_sys::Element>();
                    if let Some(filter) = btn_el.get_attribute("data-filter") {
                        if let Some(cards_container) =
                            doc.get_element_by_id("gallery-cards")
                        {
                            let children = cards_container.children();
                            for i in 0..children.length() {
                                if let Some(card) = children.item(i) {
                                    if filter == "all" {
                                        card.set_attribute(
                                            "style",
                                            "",
                                        )
                                        .unwrap();
                                    } else {
                                        let tags = card
                                            .get_attribute("data-tags")
                                            .unwrap_or_default();
                                        if tags.contains(&filter) {
                                            card.set_attribute(
                                                "style",
                                                "",
                                            )
                                            .unwrap();
                                        } else {
                                            card.set_attribute(
                                                "style",
                                                "display:none",
                                            )
                                            .unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
        );

        // Attach click listener to each filter button
        if let Some(filter_div) = document.get_element_by_id("gallery-filter") {
            let children = filter_div.children();
            for i in 0..children.length() {
                if let Some(btn) = children.item(i) {
                    if btn.class_list().contains("win95-btn") {
                        let _ = btn
                            .add_event_listener_with_callback(
                                "click",
                                filter_cb.as_ref().unchecked_ref(),
                            );
                    }
                }
            }
        }
        crate::app_state::store_closure(filter_cb);
    }
}
