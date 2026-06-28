use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

pub struct Terminal;

/// Simple input history ring buffer.
struct History {
    entries: Vec<String>,
    index: usize,
}

impl History {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            index: 0,
        }
    }

    fn push(&mut self, cmd: String) {
        if cmd.is_empty() {
            return;
        }
        if self.entries.last().map(|s| s.as_str()) == Some(&cmd) {
            return;
        }
        self.entries.push(cmd);
        self.index = self.entries.len();
    }

    fn up(&mut self) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }
        if self.index > 0 {
            self.index -= 1;
        }
        Some(self.entries[self.index].as_str())
    }

    fn down(&mut self) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }
        if self.index < self.entries.len() - 1 {
            self.index += 1;
            Some(self.entries[self.index].as_str())
        } else {
            self.index = self.entries.len();
            None
        }
    }
}

impl Terminal {
    pub fn open(document: &web_sys::Document) {
        let id = crate::app_state::with_wm(|wm| {
            wm.create_window("terminal", "Terminal", 600, 350)
        });

        let content = crate::app_state::with_wm(|wm| {
            wm.get_content(id).expect("window content not found")
        });

        // Build terminal UI
        content.set_inner_html(r#"
            <div id="terminal-body" style="background:#000;color:#0f0;font-family:monospace;font-size:14px;padding:8px;height:100%;overflow-y:auto;display:flex;flex-direction:column;">
                <div id="terminal-output" style="flex:1;overflow-y:auto;white-space:pre-wrap;"></div>
                <div id="terminal-input-line" style="display:flex;margin-top:4px;">
                    <span id="terminal-prompt" style="color:#0f0;">C:\&gt;</span>
                    <input id="terminal-input" type="text" style="flex:1;background:transparent;border:none;color:#0f0;font-family:monospace;font-size:14px;outline:none;" autofocus />
                </div>
            </div>
        "#);

        // --- Shared state ---
        let history: Rc<RefCell<History>> = Rc::new(RefCell::new(History::new()));

        // --- Auto-focus input when window content is clicked ---
        let input = document.get_element_by_id("terminal-input").unwrap();
        let input_focus = input.clone();
        let focus_cb = Closure::<dyn FnMut()>::new(move || {
            let _ = input_focus
                .unchecked_ref::<web_sys::HtmlInputElement>()
                .focus();
        });
        content
            .add_event_listener_with_callback("click", focus_cb.as_ref().unchecked_ref())
            .unwrap();
        focus_cb.forget();

        // --- Handle Enter key and arrow keys ---
        let history_enter = history.clone();
        let win_id = id;
        let doc = document.clone();

        let enter_cb = Closure::<dyn FnMut(KeyboardEvent)>::new(
            move |e: KeyboardEvent| {
                let key = e.key();

                let input_el = doc
                    .get_element_by_id("terminal-input")
                    .expect("terminal-input not found");
                let inp = input_el.unchecked_ref::<web_sys::HtmlInputElement>();

                if key == "Enter" {
                    let cmd = inp.value();
                    inp.set_value("");

                    let output =
                        doc.get_element_by_id("terminal-output")
                            .expect("terminal-output not found");

                    // Store in history
                    history_enter.borrow_mut().push(cmd.clone());

                    if cmd.trim().to_lowercase() == "clear" {
                        output.set_inner_html("");
                    } else if cmd.trim().to_lowercase() == "exit" {
                        // Close this terminal's window
                        crate::app_state::with_wm(|wm| wm.close_window(win_id));
                    } else {
                        let response = Self::execute_command(&cmd);
                        let current = output.inner_html();
                        let new_html = format!(
                            r#"{}<div><span style="color:#fff;">C:\&gt;</span> {}</div><div>{}</div>"#,
                            current,
                            cmd.trim(),
                            response
                        );
                        output.set_inner_html(&new_html);
                    }

                    // Scroll output to bottom
                    let body =
                        doc.get_element_by_id("terminal-body")
                            .expect("terminal-body not found");
                    let _ = body.set_scroll_top(body.scroll_height());
                } else if key == "ArrowUp" {
                    e.prevent_default();
                    if let Some(prev) =
                        history_enter.borrow_mut().up()
                    {
                        inp.set_value(prev);
                    }
                } else if key == "ArrowDown" {
                    e.prevent_default();
                    if let Some(next) =
                        history_enter.borrow_mut().down()
                    {
                        inp.set_value(next);
                    } else {
                        inp.set_value("");
                    }
                }
            },
        );
        input
            .add_event_listener_with_callback("keydown", enter_cb.as_ref().unchecked_ref())
            .unwrap();
        enter_cb.forget();

        // Focus input on open
        let _ = input.unchecked_ref::<web_sys::HtmlInputElement>().focus();
    }

    fn execute_command(cmd: &str) -> String {
        match cmd.trim().to_lowercase().as_str() {
            "help" => {
                "Available commands:\n\
                 \x20 help     - Show this help\n\
                 \x20 whoami   - About me\n\
                 \x20 projects - List projects\n\
                 \x20 skills   - My tech skills\n\
                 \x20 email    - Contact info\n\
                 \x20 clear    - Clear screen\n\
                 \x20 exit     - Close terminal"
                    .to_string()
            }
            "whoami" => {
                "Carlos — Software Engineer & Rust Enthusiast".to_string()
            }
            "projects" => {
                "Open the Projects Gallery window from the Start menu to see my work."
                    .to_string()
            }
            "skills" => {
                "Rust, WebAssembly, TypeScript, Python, Systems Programming, Web Development"
                    .to_string()
            }
            "email" => "carlos@example.com".to_string(),
            _ => format!(
                "'{}' is not recognized as a command. Type 'help' for available commands.",
                cmd.trim()
            ),
        }
    }
}
