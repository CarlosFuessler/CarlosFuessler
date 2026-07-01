use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

pub struct Terminal;

struct History {
    entries: Vec<String>,
    index: usize,
}

impl History {
    fn new() -> Self {
        Self { entries: Vec::new(), index: 0 }
    }

    fn push(&mut self, cmd: String) {
        if cmd.is_empty() { return; }
        if self.entries.last().map(|s| s.as_str()) == Some(&cmd) { return; }
        self.entries.push(cmd);
        self.index = self.entries.len();
    }

    fn up(&mut self) -> Option<&str> {
        if self.entries.is_empty() { return None; }
        if self.index > 0 { self.index -= 1; }
        Some(self.entries[self.index].as_str())
    }

    fn down(&mut self) -> Option<&str> {
        if self.entries.is_empty() { return None; }
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
        let id = crate::app_state::create_window("terminal", "MS-DOS Prompt", 680, 400);

        let content = crate::app_state::with_wm(|wm| {
            wm.get_content(id).expect("window content not found")
        });

        content.set_inner_html(r#"
<div id="terminal-body" style="background:#000;color:#c0c0c0;font-family:'Courier New',monospace;font-size:14px;height:100%;display:flex;flex-direction:column;">
  <div id="terminal-output" style="flex:1;overflow-y:auto;padding:4px;white-space:pre-wrap;scroll-behavior:smooth;line-height:1.4;"></div>
  <div id="terminal-input-line" style="display:flex;padding:2px 4px 4px;border-top:1px solid #333;flex-shrink:0;">
    <span id="terminal-prompt" style="color:#c0c0c0;white-space:nowrap;">C:\&gt;</span>
    <input id="terminal-input" type="text" style="flex:1;background:transparent;border:none;color:#c0c0c0;font-family:'Courier New',monospace;font-size:14px;outline:none;padding:0;margin-left:4px;" autofocus />
  </div>
</div>
        "#);

        let input = document.get_element_by_id("terminal-input").unwrap();

        // Focus input on click
        let inp_focus = input.clone();
        let focus_cb = Closure::<dyn FnMut()>::new(move || {
            let _ = inp_focus.unchecked_ref::<web_sys::HtmlInputElement>().focus();
        });
        content.add_event_listener_with_callback("click", focus_cb.as_ref().unchecked_ref()).unwrap();
        crate::app_state::store_closure(focus_cb);

        let history: Rc<RefCell<History>> = Rc::new(RefCell::new(History::new()));
        let history_enter = history.clone();
        let win_id = id;
        let doc = document.clone();
        let output = document.get_element_by_id("terminal-output").unwrap();

        // Write boot banner
        output.set_inner_html("\
System 95 [Version 95.0.0]\r\n\
   (C)Copyright 1981-1996.\r\n\
\r\n\
C:\\&gt; Type HELP for available commands.\r\n\
\r\n");

        let enter_cb = Closure::<dyn FnMut(KeyboardEvent)>::new(
            move |e: KeyboardEvent| {
                let key = e.key();
                let input_el = doc.get_element_by_id("terminal-input")
                    .expect("terminal-input not found");
                let inp = input_el.unchecked_ref::<web_sys::HtmlInputElement>();

                if key == "Enter" {
                    let cmd = inp.value();
                    inp.set_value("");
                    history_enter.borrow_mut().push(cmd.clone());

                    let out = doc.get_element_by_id("terminal-output")
                        .expect("terminal-output not found");

                    if cmd.trim().to_lowercase() == "clear" {
                        out.set_inner_html("");
                    } else if cmd.trim().to_lowercase() == "exit" {
                        crate::app_state::with_wm(|wm| wm.close_window(win_id));
                    } else {
                        let response = Self::execute_command(&cmd);
                        let current = out.inner_html();
                        let new_html = format!(
                            r#"{}<div><span style="color:#c0c0c0;">C:\&gt;</span> {}</div><div>{}</div>"#,
                            current, cmd.trim(), response
                        );
                        out.set_inner_html(&new_html);
                    }

                    if let Some(body) = doc.get_element_by_id("terminal-body") {
                        let scroll = body.scroll_height();
                        body.set_scroll_top(scroll);
                    }
                } else if key == "ArrowUp" {
                    e.prevent_default();
                    if let Some(prev) = history_enter.borrow_mut().up() {
                        inp.set_value(prev);
                    }
                } else if key == "ArrowDown" {
                    e.prevent_default();
                    if let Some(next) = history_enter.borrow_mut().down() {
                        inp.set_value(next);
                    } else {
                        inp.set_value("");
                    }
                }
            },
        );
        input.add_event_listener_with_callback("keydown", enter_cb.as_ref().unchecked_ref()).unwrap();
        crate::app_state::store_closure(enter_cb);

        let _ = input.unchecked_ref::<web_sys::HtmlInputElement>().focus();
    }

    fn execute_command(cmd: &str) -> String {
        let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
        if parts.is_empty() { return String::new(); }
        let command = parts[0].to_lowercase();
        let args = &parts[1..];

        match command.as_str() {
            "help" => {
                "\
Available commands:\r\n\
  HELP      Show this help\r\n\
  WHOAMI    Display user info\r\n\
  PROJECTS  List projects\r\n\
  SKILLS    Show tech skills\r\n\
  EMAIL     Show contact info\r\n\
  DATE      Show current date\r\n\
  TIME      Show current time\r\n\
  ECHO      Display a message\r\n\
  DIR       List files (simulated)\r\n\
  TYPE      Display file contents\r\n\
  VER       Show version\r\n\
  CLEAR     Clear the screen\r\n\
  EXIT      Close terminal".to_string()
            }
            "whoami" => {
                "Carlos — Software Engineer & Rust Enthusiast".to_string()
            }
            "projects" => {
                "Open the Projects Gallery from the Start menu to see my work.\r\n\
                 Notable: System 95 Desktop (this!), systems tooling, web apps.".to_string()
            }
            "skills" => {
                "Languages:  Rust, TypeScript, Python, C/C++\r\n\
                 Web:        WASM, HTML/CSS, React\r\n\
                 Systems:    Linux, Networking, Embedded\r\n\
                 Tools:      Git, Docker, Zig".to_string()
            }
            "email" => {
                "carlos@example.com".to_string()
            }
            "date" => {
                let now = js_sys::Date::new_0();
                let months = ["Jan","Feb","Mar","Apr","May","Jun",
                              "Jul","Aug","Sep","Oct","Nov","Dec"];
                let m = months[now.get_month() as usize];
                let d = now.get_date();
                let y = now.get_full_year();
                let wd = now.get_day();
                let days = ["Sun","Mon","Tue","Wed","Thu","Fri","Sat"];
                format!("{} {} {} {}", days[wd as usize], m, d, y)
            }
            "time" => {
                let now = js_sys::Date::new_0();
                let h = now.get_hours();
                let m = now.get_minutes();
                let s = now.get_seconds();
                let ampm = if h < 12 { "AM" } else { "PM" };
                let h12 = if h % 12 == 0 { 12 } else { h % 12 };
                format!("{}:{:02}:{:02} {}", h12, m, s, ampm)
            }
            "echo" => {
                if args.is_empty() {
                    "ECHO is on.".to_string()
                } else {
                    args.join(" ")
                }
            }
            "dir" => {
                "\
 Volume in drive C is SYSTEM95\r\n\
 Directory of C:\\\r\n\
\r\n\
about/        &lt;DIR&gt;    06-29-26  12:00\r\n\
cv/           &lt;DIR&gt;    06-29-26  12:00\r\n\
projects/     &lt;DIR&gt;    06-29-26  12:00\r\n\
skills/       &lt;DIR&gt;    06-29-26  12:00\r\n\
contact/      &lt;DIR&gt;    06-29-26  12:00\r\n\
README         1,024  06-29-26  12:00\r\n\
        5 Dir(s)   1,234,567,890 bytes free".to_string()
            }
            "type" => {
                if args.is_empty() {
                    "Syntax: TYPE [filename]".to_string()
                } else {
                    format!("Cannot find file '{}' — try opening it with File Manager.", args.join(" "))
                }
            }
            "ver" => {
                "\
System 95 [Version 95.0.0]\r\n\
(C) 1981-1996.\r\n\
This CV is running in your browser via Rust + WebAssembly.".to_string()
            }
            _ => {
                format!(
                    "'{}' is not recognized as an internal or external command,\r\n\
                     operable program or batch file.",
                    cmd.trim()
                )
            }
        }
    }
}
