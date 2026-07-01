use std::cell::RefCell;

thread_local! {
    static STATE: RefCell<CalcState> = const { RefCell::new(CalcState {
        display: String::new(),
        current: 0.0,
        previous: 0.0,
        operation: None,
        memory: 0.0,
        has_memory: false,
        new_input: true,
    })};
}

struct CalcState {
    display: String,
    current: f64,
    previous: f64,
    operation: Option<String>,
    memory: f64,
    has_memory: bool,
    new_input: bool,
}

pub struct CalculatorApp;

impl CalculatorApp {
    pub fn open(document: &web_sys::Document) {
        let id = crate::app_state::create_window("calculator", "Calculator", 240, 320);
        let content = crate::app_state::with_wm(|wm| wm.get_content(id)).unwrap();
        create(&content, 240, 320);
    }
}

pub fn create(parent: &web_sys::Element, _w: u32, _h: u32) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    parent.set_inner_html(r#"
<div style="display:flex;flex-direction:column;gap:4px;padding:4px;background:var(--silver);height:100%;">
  <div style="text-align:right;overflow:hidden;display:flex;flex-direction:column;">
    <div id="calc-indicators" style="font-size:10px;height:14px;color:var(--silver-dark);padding:0 4px;text-align:left;"></div>
    <div id="calc-display" style="
      background:var(--white);border:2px inset var(--silver-dark);
      padding:4px 8px;font-family:'Courier New',monospace;font-size:22px;
      text-align:right;overflow:hidden;min-height:32px;
    ">0</div>
  </div>
  <div id="calc-buttons" style="display:grid;grid-template-columns:repeat(4,1fr);gap:2px;flex:1;">
    <button class="calc-btn" data-action="MC">MC</button>
    <button class="calc-btn" data-action="MR">MR</button>
    <button class="calc-btn" data-action="MS">MS</button>
    <button class="calc-btn" data-action="M+">M+</button>
    <button class="calc-btn" data-action="7">7</button>
    <button class="calc-btn" data-action="8">8</button>
    <button class="calc-btn" data-action="9">9</button>
    <button class="calc-btn" data-action="/">÷</button>
    <button class="calc-btn" data-action="4">4</button>
    <button class="calc-btn" data-action="5">5</button>
    <button class="calc-btn" data-action="6">6</button>
    <button class="calc-btn" data-action="*">×</button>
    <button class="calc-btn" data-action="1">1</button>
    <button class="calc-btn" data-action="2">2</button>
    <button class="calc-btn" data-action="3">3</button>
    <button class="calc-btn" data-action="-">−</button>
    <button class="calc-btn" data-action="0">0</button>
    <button class="calc-btn" data-action=".">.</button>
    <button class="calc-btn" data-action="+/-">±</button>
    <button class="calc-btn" data-action="+">+</button>
    <button class="calc-btn" data-action="C" style="grid-column:span 2;">C</button>
    <button class="calc-btn" data-action="CE">CE</button>
    <button class="calc-btn" data-action="Back">⌫</button>
    <button class="calc-btn" data-action="=" style="grid-column:span 2;background:#dfdfdf;">=</button>
  </div>
</div>
"#);

    let display_el = parent.query_selector("#calc-display").unwrap().unwrap();
    let indicators_el = parent.query_selector("#calc-indicators").unwrap().unwrap();
    let buttons = parent.query_selector("#calc-buttons").unwrap().unwrap();

    let doc = parent.owner_document().unwrap();

    // Reset state on open
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.display = "0".to_string();
        state.current = 0.0;
        state.previous = 0.0;
        state.operation = None;
        state.new_input = true;
    });

    let cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new({
        let display = display_el.clone();
        let indicators = indicators_el.clone();
        move |ev: web_sys::MouseEvent| {
            ev.stop_propagation();
            if let Some(target) = ev.target() {
                if let Some(btn) = target.dyn_ref::<web_sys::HtmlElement>() {
                    if let Some(action) = btn.get_attribute("data-action") {
                        STATE.with(|s| {
                            let mut state = s.borrow_mut();
                            handle_calc_action(&mut state, &action, &display, &indicators);
                        });
                    }
                }
            }
        }
    });
    buttons.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(cb);

    // Keyboard support
    let d2 = display_el.clone();
    let i2 = indicators_el.clone();
    let kb_cb = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
        move |ev: web_sys::KeyboardEvent| {
            let key = ev.key();
            let action = match key.as_str() {
                k if k.len() == 1 && k.chars().next().unwrap().is_ascii_digit() => k.to_string(),
                "." => ".".to_string(),
                "Enter" | "=" => "=".to_string(),
                "+" => "+".to_string(),
                "-" => "-".to_string(),
                "*" => "*".to_string(),
                "/" => "/".to_string(),
                "Escape" => "C".to_string(),
                "Backspace" => "Back".to_string(),
                _ => return,
            };
            ev.prevent_default();
            STATE.with(|s| {
                let mut state = s.borrow_mut();
                handle_calc_action(&mut state, &action, &d2, &i2);
            });
        }
    );
    doc.add_event_listener_with_callback("keydown", kb_cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(kb_cb);
}

fn handle_calc_action(
    state: &mut CalcState,
    action: &str,
    display: &web_sys::Element,
    indicators: &web_sys::Element,
) {
    match action {
        "C" => {
            state.display = "0".to_string();
            state.current = 0.0;
            state.previous = 0.0;
            state.operation = None;
            state.new_input = true;
        }
        "CE" => {
            state.display = "0".to_string();
            state.current = 0.0;
            state.new_input = true;
        }
        "Back" => {
            if state.display.len() > 1 {
                state.display = state.display[..state.display.len()-1].to_string();
            } else {
                state.display = "0".to_string();
            }
            state.current = state.display.parse().unwrap_or(0.0);
        }
        "+/-" => {
            if let Ok(v) = state.display.parse::<f64>() {
                let neg = -v;
                state.display = format_num(neg);
                state.current = neg;
            }
        }
        "MC" => { state.memory = 0.0; state.has_memory = false; }
        "MR" => {
            state.current = state.memory;
            state.display = format_num(state.memory);
            state.new_input = true;
        }
        "MS" => { state.memory = state.current; state.has_memory = true; }
        "M+" => { state.memory += state.current; state.has_memory = true; }
        "+" | "-" | "*" | "/" => {
            if let Some(ref op) = state.operation {
                let result = apply_op(state.previous, state.current, op);
                state.display = format_num(result);
                state.current = result;
            }
            state.previous = state.current;
            state.operation = Some(action.to_string());
            state.new_input = true;
        }
        "=" => {
            if let Some(ref op) = state.operation {
                let result = apply_op(state.previous, state.current, op);
                state.display = format_num(result);
                state.current = result;
                state.operation = None;
                state.new_input = true;
            }
        }
        _ => {
            // Number or decimal
            if action == "." && state.display.contains('.') {
                if !state.new_input { return; }
            }
            if state.new_input {
                if action == "." {
                    state.display = "0.".to_string();
                } else {
                    state.display = action.to_string();
                }
                state.new_input = false;
            } else if state.display.len() < 15 {
                if state.display == "0" && action != "." {
                    state.display = action.to_string();
                } else {
                    state.display.push_str(action);
                }
            }
            state.current = state.display.parse().unwrap_or(0.0);
        }
    }
    display.set_inner_html(&state.display);
    indicators.set_inner_html(if state.has_memory { "M" } else { "" });
}

fn apply_op(a: f64, b: f64, op: &str) -> f64 {
    match op {
        "+" => a + b,
        "-" => a - b,
        "*" => a * b,
        "/" => if b != 0.0 { a / b } else { 0.0 },
        _ => b,
    }
}

fn format_num(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else if n.abs() < 1e15 {
        format!("{}", n)
    } else {
        format!("{:e}", n)
    }
}
