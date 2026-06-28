pub struct ContactApp;

impl ContactApp {
    pub fn open(document: &web_sys::Document) {
        let id = crate::app_state::with_wm(|wm| {
            wm.create_window("contact", "Contact", 400, 250)
        });

        let content = crate::app_state::with_wm(|wm| {
            wm.get_content(id).expect("window content not found")
        });

        content.set_inner_html(r#"
            <div style="padding:16px;display:flex;flex-direction:column;gap:12px;">
                <div>📧 <a href="mailto:carlos@example.com" style="color:#0000ff;text-decoration:underline;">carlos@example.com</a></div>
                <div>🐙 <a href="https://github.com/carlos" target="_blank" style="color:#0000ff;text-decoration:underline;">github.com/carlos</a></div>
                <div>🔗 <a href="https://linkedin.com/in/carlos" target="_blank" style="color:#0000ff;text-decoration:underline;">linkedin.com/in/carlos</a></div>
            </div>
        "#);
    }
}
