use std::cell::RefCell;

thread_local! {
    static CAL_YEAR: RefCell<i32> = const { RefCell::new(0) };
    static CAL_MONTH: RefCell<u32> = const { RefCell::new(0) };
}

const MONTH_NAMES: &[&str] = &[
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December"
];

pub struct CalendarApp;

impl CalendarApp {
    pub fn open(document: &web_sys::Document) {
        let id = crate::app_state::create_window("calendar", "Calendar", 280, 280);
        let content = crate::app_state::with_wm(|wm| wm.get_content(id)).unwrap();
        create(&content, 280, 280);
    }
}

pub fn create(parent: &web_sys::Element, _w: u32, _h: u32) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    // Get current date
    let now = js_sys::Date::new_0();
    let cur_year = now.get_full_year() as i32;
    let cur_month = now.get_month() + 1; // 1-indexed
    let today = now.get_date() as i32;

    CAL_YEAR.with(|y| *y.borrow_mut() = cur_year);
    CAL_MONTH.with(|m| *m.borrow_mut() = cur_month);

    parent.set_inner_html("");
    let doc = parent.owner_document().unwrap();
    let container = doc.create_element("div").unwrap();
    container.set_attribute("style", "padding:4px;font-family:var(--font);height:100%;background:var(--silver);box-sizing:border-box;").unwrap();
    parent.append_child(&container).unwrap();

    let p = container.clone();
    let d = doc.clone();
    let cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
        move |ev: web_sys::MouseEvent| {
            ev.stop_propagation();
            if let Some(target) = ev.target() {
                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                    if let Some(dir) = el.get_attribute("data-nav") {
                        CAL_MONTH.with(|m| {
                            let mut month = m.borrow_mut();
                            if dir == "prev" {
                                if *month == 1 {
                                    *month = 12;
                                    CAL_YEAR.with(|y| *y.borrow_mut() -= 1);
                                } else {
                                    *month -= 1;
                                }
                            } else {
                                if *month == 12 {
                                    *month = 1;
                                    CAL_YEAR.with(|y| *y.borrow_mut() += 1);
                                } else {
                                    *month += 1;
                                }
                            }
                        });
                        render_calendar(&p, &d, today);
                    }
                }
            }
        }
    );
    container.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(cb);

    render_calendar(&container, &doc, today);
}

fn render_calendar(container: &web_sys::Element, doc: &web_sys::Document, today: i32) {
    let year = CAL_YEAR.with(|y| *y.borrow());
    let month = CAL_MONTH.with(|m| *m.borrow());

    // First day of month and days in month
    let first = js_sys::Date::new_with_year_month(year as u32, (month - 1) as i32);
    let start_day = first.get_day() as i32; // 0=Sun, 6=Sat

    let last_day = if month == 12 {
        31
    } else {
        let next = js_sys::Date::new_with_year_month(year as u32, month as i32);
        let ms = next.get_time() - first.get_time();
        (ms / 86400000.0) as i32
    };

    let month_name = MONTH_NAMES.get((month - 1) as usize).unwrap_or(&"?");

    let mut html = format!(
        r#"<div style="text-align:center;font-size:14px;font-weight:bold;padding:8px 4px;display:flex;justify-content:space-between;align-items:center;">
           <span style="cursor:pointer;user-select:none;padding:2px 8px;border:1px outset var(--white);background:var(--silver);" data-nav="prev">◀</span>
           <span>{} {}</span>
           <span style="cursor:pointer;user-select:none;padding:2px 8px;border:1px outset var(--white);background:var(--silver);" data-nav="next">▶</span>
         </div>"#,
        month_name, year
    );

    html.push_str(r#"<table style="width:100%;border-collapse:collapse;text-align:center;font-size:12px;background:var(--white);border:2px inset var(--silver-dark);">
      <tr style="background:var(--silver-dark);color:white;"><th>Su</th><th>Mo</th><th>Tu</th><th>We</th><th>Th</th><th>Fr</th><th>Sa</th></tr><tr style="height:28px;">"#);

    for _ in 0..start_day {
        html.push_str("<td></td>");
    }

    let mut col = start_day;
    for day in 1..=last_day {
        // Highlight today only if it's the current real-world month and year
        let real_now = js_sys::Date::new_0();
        let is_today = day == today 
            && (month - 1) == real_now.get_month() 
            && year == real_now.get_full_year() as i32;

        let style = if is_today {
            "background:var(--title-active);color:var(--title-active-text);font-weight:bold;border:1px outset var(--white);"
        } else {
            "border:1px solid #eee;"
        };

        html.push_str(&format!(r#"<td style="{}">{}</td>"#, style, day));
        col += 1;
        if col > 6 && day < last_day {
            html.push_str(r#"</tr><tr style="height:28px;">"#);
            col = 0;
        }
    }

    // Fill remaining cells in the last row
    if col > 0 && col <= 6 {
        for _ in col..=6 {
            html.push_str("<td></td>");
        }
    }

    html.push_str("</tr></table>");
    container.set_inner_html(&html);
}
