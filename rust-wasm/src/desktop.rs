pub struct DesktopIcon {
    pub id: String,
    pub title: String,
    pub icon: &'static str,
}

pub fn render_desktop_icons(document: &web_sys::Document, icons: &[DesktopIcon]) {
    let container = document.get_element_by_id("desktop-icons").unwrap();
    container.set_inner_html("");
    for icon in icons {
        let el = document.create_element("div").unwrap();
        el.set_attribute("class", "desktop-icon").unwrap();
        el.set_inner_html(&format!(
            r#"<div class="desktop-icon-img">{}</div><div class="desktop-icon-label">{}</div>"#,
            icon.icon, icon.title
        ));
        container.append_child(&el).unwrap();
    }
}
