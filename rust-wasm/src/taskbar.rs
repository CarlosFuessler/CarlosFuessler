pub struct Taskbar {
    document: web_sys::Document,
}

impl Taskbar {
    pub fn new(document: &web_sys::Document) -> Self {
        Self {
            document: document.clone(),
        }
    }

    pub fn add_app_button(&self, id: &str, title: &str) {
        let container = self.document.get_element_by_id("tb-apps").unwrap();
        let btn = self.document.create_element("button").unwrap();
        btn.set_attribute("id", &format!("tb-btn-{}", id)).unwrap();
        btn.set_attribute("class", "tb-app-btn").unwrap();
        btn.set_inner_html(&format!("⬜ {}", title));
        container.append_child(&btn).unwrap();
    }

    pub fn remove_app_button(&self, id: &str) {
        let btn_id = format!("tb-btn-{}", id);
        if let Some(btn) = self.document.get_element_by_id(&btn_id) {
            if let Some(parent) = btn.parent_element() {
                parent.remove_child(&btn).unwrap();
            }
        }
    }

    pub fn set_active(&self, id: &str) {
        let container = self.document.get_element_by_id("tb-apps").unwrap();
        let children = container.children();
        for i in 0..children.length() {
            if let Some(child) = children.item(i) {
                child
                    .class_list()
                    .remove_1("active")
                    .unwrap();
            }
        }
        let btn_id = format!("tb-btn-{}", id);
        if let Some(btn) = self.document.get_element_by_id(&btn_id) {
            btn.class_list().add_1("active").unwrap();
        }
    }
}
