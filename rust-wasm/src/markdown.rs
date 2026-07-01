use wasm_bindgen::JsCast;
use pulldown_cmark::{Parser, html};

pub struct MarkdownViewer;

impl MarkdownViewer {
    pub fn open(document: &web_sys::Document, title: &str, path: &str) {
        let id = crate::app_state::create_window("markdown", title, 550, 400);

        let content = crate::app_state::with_wm(|wm| {
            wm.get_content(id).expect("window content not found")
        });
        content.set_inner_html("<p style='padding:8px;'>Loading...</p>");

        let path = path.to_string();
        let content_clone = content.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let resp = wasm_bindgen_futures::JsFuture::from(
                web_sys::window().unwrap().fetch_with_str(&path)
            ).await;

            match resp {
                Ok(resp) => {
                    let text = wasm_bindgen_futures::JsFuture::from(
                        resp.dyn_into::<web_sys::Response>().unwrap().text().unwrap()
                    ).await;

                    match text {
                        Ok(text) => {
                            let md = text.as_string().unwrap();
                            let parser = Parser::new(&md);
                            let mut html_output = String::new();
                            html::push_html(&mut html_output, parser);
                            content_clone.set_inner_html(&format!(
                                r#"<div class="markdown-body">{}</div>"#, html_output
                            ));
                        }
                        Err(_) => {
                            content_clone.set_inner_html("<p style='padding:8px;color:red;'>Error loading file</p>");
                        }
                    }
                }
                Err(_) => {
                    content_clone.set_inner_html("<p style='padding:8px;color:red;'>Error fetching file</p>");
                }
            }
        });

        // Suppress unused warning for document parameter (kept for API consistency)
        let _ = document;
    }
}
