use std::cell::RefCell;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

const GRID_SIZE: usize = 9;
const MINE_COUNT: usize = 10;

struct MsState {
    grid: Vec<Vec<CellState>>,
    revealed: Vec<Vec<bool>>,
    flagged: Vec<Vec<bool>>,
    game_over: bool,
    won: bool,
    first_click: bool,
    flag_count: i32,
    timer_secs: i32,
    timer_active: bool,
    timer_handle: Option<js_sys::Function>,
}

#[derive(Clone, Copy, PartialEq)]
enum CellState {
    Mine,
    Number(u8),
    Empty,
}

thread_local! {
    static MS: RefCell<MsState> = const { RefCell::new(MsState {
        grid: Vec::new(),
        revealed: Vec::new(),
        flagged: Vec::new(),
        game_over: false,
        won: false,
        first_click: false,
        flag_count: 0,
        timer_secs: 0,
        timer_active: false,
        timer_handle: None,
    })};
}

pub struct MinesweeperApp;

impl MinesweeperApp {
    pub fn open(document: &web_sys::Document) {
        let id = crate::app_state::create_window("minesweeper", "Minesweeper", 200, 280);
        let content = crate::app_state::with_wm(|wm| wm.get_content(id)).unwrap();
        create(&content, 200, 280);
    }
}

pub fn create(parent: &web_sys::Element, _w: u32, _h: u32) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    init_game();

    let doc = parent.owner_document().unwrap();

    parent.set_inner_html(r#"
<div style="display:flex;flex-direction:column;align-items:center;padding:4px;gap:4px;background:var(--silver);height:100%;box-sizing:border-box;">
  <div style="display:flex;justify-content:space-between;width:100%;padding:2px 4px;background:#c0c0c0;border:2px inset var(--silver-dark);box-sizing:border-box;">
    <div id="ms-mine-counter" style="font-family:'Courier New';font-size:16px;background:#000;color:red;padding:2px 6px;min-width:32px;text-align:center;">010</div>
    <div id="ms-face" style="font-size:12px;font-family:monospace;font-weight:bold;cursor:pointer;user-select:none;border:2px outset var(--white);background:var(--silver);padding:2px 6px;display:inline-flex;align-items:center;justify-content:center;">:-)</div>
    <div id="ms-timer" style="font-family:'Courier New';font-size:16px;background:#000;color:red;padding:2px 6px;min-width:32px;text-align:center;">000</div>
  </div>
  <div id="ms-grid" style="display:grid;grid-template-columns:repeat(9,20px);gap:0;border:2px inset var(--silver-dark);"></div>
</div>
"#);

    let grid_el = parent.query_selector("#ms-grid").unwrap().unwrap();
    let face_el = parent.query_selector("#ms-face").unwrap().unwrap();

    render_grid(&grid_el, &doc);

    // Face button reset
    let f = face_el.clone();
    let g = grid_el.clone();
    let d = doc.clone();
    let reset_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
        move |_ev: web_sys::MouseEvent| {
            MS.with(|ms| {
                let mut s = ms.borrow_mut();
                s.timer_active = false;
                s.timer_handle = None;
            });
            init_game();
            render_grid(&g, &d);
            f.set_inner_html(":-)");
            update_display(&d);
        }
    );
    face_el.add_event_listener_with_callback("click", reset_cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(reset_cb);

    // Click handler for grid cells
    let g2 = grid_el.clone();
    let d2 = doc.clone();
    let f2 = face_el.clone();
    let click_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
        move |ev: web_sys::MouseEvent| {
            ev.stop_propagation();
            if let Some(target) = ev.target() {
                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                    if let Some(idx_str) = el.get_attribute("data-idx") {
                        if let Ok(idx) = idx_str.parse::<usize>() {
                            let row = idx / GRID_SIZE;
                            let col = idx % GRID_SIZE;
                            MS.with(|ms| {
                                let mut s = ms.borrow_mut();
                                if s.game_over || s.won { return; }
                                if !s.first_click {
                                    seed_mines(&mut s, row, col);
                                    s.first_click = true;
                                    s.timer_active = true;
                                    start_timer(&d2);
                                }
                                if s.flagged[row][col] { return; }
                                if !reveal(&mut s, row, col) {
                                    s.game_over = true;
                                    s.timer_active = false;
                                    f2.set_inner_html("x_x");
                                } else if check_win(&s) {
                                    s.won = true;
                                    s.timer_active = false;
                                    f2.set_inner_html("B-)");
                                }
                            });
                            render_grid(&g2, &d2);
                            update_display(&d2);
                        }
                    }
                }
            }
        }
    );
    grid_el.add_event_listener_with_callback("click", click_cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(click_cb);

    // Right-click handler for flagging
    let g3 = grid_el.clone();
    let d3 = doc.clone();
    let context_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
        move |ev: web_sys::MouseEvent| {
            ev.prevent_default();
            ev.stop_propagation();
            if let Some(target) = ev.target() {
                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                    if let Some(idx_str) = el.get_attribute("data-idx") {
                        if let Ok(idx) = idx_str.parse::<usize>() {
                            let row = idx / GRID_SIZE;
                            let col = idx % GRID_SIZE;
                            MS.with(|ms| {
                                let mut s = ms.borrow_mut();
                                if s.game_over || s.won || !s.first_click || s.revealed[row][col] { return; }
                                s.flagged[row][col] = !s.flagged[row][col];
                                s.flag_count = s.flagged.iter().flatten().filter(|&&f| f).count() as i32;
                            });
                            render_grid(&g3, &d3);
                            update_display(&d3);
                        }
                    }
                }
            }
        }
    );
    grid_el.add_event_listener_with_callback("contextmenu", context_cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(context_cb);

    update_display(&doc);
}

fn init_game() {
    MS.with(|ms| {
        let mut s = ms.borrow_mut();
        s.grid = vec![vec![CellState::Empty; GRID_SIZE]; GRID_SIZE];
        s.revealed = vec![vec![false; GRID_SIZE]; GRID_SIZE];
        s.flagged = vec![vec![false; GRID_SIZE]; GRID_SIZE];
        s.game_over = false;
        s.won = false;
        s.first_click = false;
        s.flag_count = 0;
        s.timer_secs = 0;
        s.timer_active = false;
        s.timer_handle = None;
    });
}

fn seed_mines(s: &mut MsState, safe_row: usize, safe_col: usize) {
    let mut rng = js_sys::Math::random;
    let mut placed = 0;
    while placed < MINE_COUNT {
        let r = (rng() * GRID_SIZE as f64) as usize;
        let c = (rng() * GRID_SIZE as f64) as usize;
        if s.grid[r][c] == CellState::Mine || (r == safe_row && c == safe_col) { continue; }
        s.grid[r][c] = CellState::Mine;
        placed += 1;
    }
    // Calculate numbers
    for r in 0..GRID_SIZE {
        for c in 0..GRID_SIZE {
            if s.grid[r][c] == CellState::Mine { continue; }
            let mut count = 0u8;
            for dr in -1i32..=1 {
                for dc in -1i32..=1 {
                    let nr = r as i32 + dr;
                    let nc = c as i32 + dc;
                    if nr >= 0 && nr < GRID_SIZE as i32 && nc >= 0 && nc < GRID_SIZE as i32 {
                        if s.grid[nr as usize][nc as usize] == CellState::Mine {
                            count += 1;
                        }
                    }
                }
            }
            s.grid[r][c] = if count > 0 { CellState::Number(count) } else { CellState::Empty };
        }
    }
}

fn reveal(s: &mut MsState, row: usize, col: usize) -> bool {
    if row >= GRID_SIZE || col >= GRID_SIZE || s.revealed[row][col] || s.flagged[row][col] {
        return true;
    }
    s.revealed[row][col] = true;
    if s.grid[row][col] == CellState::Mine {
        return false;
    }
    if s.grid[row][col] == CellState::Empty {
        for dr in -1i32..=1 {
            for dc in -1i32..=1 {
                let nr = row as i32 + dr;
                let nc = col as i32 + dc;
                if nr >= 0 && nr < GRID_SIZE as i32 && nc >= 0 && nc < GRID_SIZE as i32 {
                    reveal(s, nr as usize, nc as usize);
                }
            }
        }
    }
    true
}

fn check_win(s: &MsState) -> bool {
    let total = GRID_SIZE * GRID_SIZE;
    let revealed_count = s.revealed.iter().flatten().filter(|&&r| r).count();
    total - revealed_count == MINE_COUNT
}

fn render_grid(grid_el: &web_sys::Element, _doc: &web_sys::Document) {
    let mut html = String::new();
    MS.with(|ms| {
        let s = ms.borrow();
        for r in 0..GRID_SIZE {
            for c in 0..GRID_SIZE {
                let idx = r * GRID_SIZE + c;
                let content = if s.revealed[r][c] {
                    if s.grid[r][c] == CellState::Mine {
                        "<svg class=\"icon-svg\" style=\"margin-right:0;\" viewBox=\"0 0 16 16\"><circle cx=\"8\" cy=\"8\" r=\"4\" fill=\"#000\"/><path stroke=\"#000\" d=\"M8,1 v14 M1,8 h14 M3,3 l10,10 M3,12 l10,-10\"/><circle cx=\"6\" cy=\"6\" r=\"1\" fill=\"#fff\"/></svg>"
                    } else if let CellState::Number(n) = s.grid[r][c] {
                        let colors = ["", "blue", "green", "red", "navy", "maroon", "teal", "black", "gray"];
                        let color = colors.get(n as usize).unwrap_or(&"black");
                        &format!("<span style='color:{};font-weight:bold'>{}</span>", color, n)
                    } else {
                        " "
                    }
                } else if s.flagged[r][c] {
                    "<svg class=\"icon-svg\" style=\"margin-right:0;\" viewBox=\"0 0 16 16\"><polygon points=\"4,2 12,5 4,8\" fill=\"red\" stroke=\"red\"/><line x1=\"4\" y1=\"2\" x2=\"4\" y2=\"14\" stroke=\"#000\" stroke-width=\"1.5\"/></svg>"
                } else if s.game_over {
                    if s.grid[r][c] == CellState::Mine {
                        "<svg class=\"icon-svg\" style=\"margin-right:0;\" viewBox=\"0 0 16 16\"><circle cx=\"8\" cy=\"8\" r=\"4\" fill=\"#000\"/><path stroke=\"#000\" d=\"M8,1 v14 M1,8 h14 M3,3 l10,10 M3,12 l10,-10\"/><circle cx=\"6\" cy=\"6\" r=\"1\" fill=\"#fff\"/></svg>"
                    } else { " " }
                } else {
                    " "
                };

                let border_style = if s.revealed[r][c] {
                    "border:1px solid #7f7f7f;background:#fff"
                } else {
                    "border:2px outset var(--white);background:var(--silver)"
                };

                let cls = if s.game_over && s.grid[r][c] == CellState::Mine && !s.flagged[r][c] {
                    "border:1px solid #7f7f7f;background:#ffcccc"
                } else {
                    border_style
                };

                html.push_str(&format!(
                    r#"<div data-idx="{}" style="width:20px;height:20px;display:flex;align-items:center;justify-content:center;font-size:11px;cursor:pointer;{};user-select:none;box-sizing:border-box;">{}</div>"#,
                    idx, cls, content
                ));
            }
        }
    });
    grid_el.set_inner_html(&html);
}

fn update_display(doc: &web_sys::Document) {
    if let Some(counter) = doc.get_element_by_id("ms-mine-counter") {
        let remaining = MS.with(|ms| (MINE_COUNT as i32 - ms.borrow().flag_count).max(0));
        counter.set_inner_html(&format!("{:03}", remaining));
    }
    if let Some(timer) = doc.get_element_by_id("ms-timer") {
        let secs = MS.with(|ms| ms.borrow().timer_secs);
        timer.set_inner_html(&format!("{:03}", secs.min(999)));
    }
}

fn start_timer(doc: &web_sys::Document) {
    let window = web_sys::window().unwrap();
    let d = doc.clone();
    let cb = Closure::<dyn FnMut()>::new(move || {
        let should_continue = MS.with(|ms| {
            let mut s = ms.borrow_mut();
            if s.timer_active && !s.game_over && !s.won {
                s.timer_secs += 1;
                true
            } else {
                false
            }
        });
        update_display(&d);
        if should_continue {
            MS.with(|ms| {
                let s = ms.borrow();
                if let Some(h) = &s.timer_handle {
                    if let Some(w) = web_sys::window() {
                        let fn_ref = h.clone();
                        let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                            fn_ref.unchecked_ref(), 1000
                        );
                    }
                }
            });
        }
    });
    let fn_ref = cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
    crate::app_state::store_closure(cb);
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&fn_ref, 1000);
    MS.with(|ms| {
        ms.borrow_mut().timer_handle = Some(fn_ref);
    });
}
