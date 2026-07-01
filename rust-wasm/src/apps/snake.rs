use std::cell::RefCell;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

const GRID_SIZE: i32 = 20;

struct SnakeState {
    body: Vec<(i32, i32)>,
    direction: (i32, i32),
    next_direction: (i32, i32),
    food: (i32, i32),
    score: u32,
    high_score: u32,
    game_over: bool,
    paused: bool,
    speed_ms: u32,
    timer_handle: Option<js_sys::Function>,
}

thread_local! {
    static STATE: RefCell<SnakeState> = const { RefCell::new(SnakeState {
        body: Vec::new(),
        direction: (1, 0),
        next_direction: (1, 0),
        food: (10, 10),
        score: 0,
        high_score: 0,
        game_over: false,
        paused: false,
        speed_ms: 150,
        timer_handle: None,
    })};
}

pub struct SnakeApp;

impl SnakeApp {
    pub fn open(document: &web_sys::Document) {
        let id = crate::app_state::create_window("snake", "Snake", 340, 420);
        let content = crate::app_state::with_wm(|wm| wm.get_content(id)).unwrap();
        create(&content, 340, 420);
    }
}

pub fn create(parent: &web_sys::Element, _w: u32, _h: u32) {
    parent.set_attribute("style", "padding:0;display:flex;flex-direction:column;background:var(--silver);height:100%;box-sizing:border-box;overflow:hidden;").unwrap();

    parent.set_inner_html(r#"
<div style="display:flex;flex-direction:column;align-items:center;padding:8px;gap:8px;height:100%;box-sizing:border-box;">
  <div style="display:flex;justify-content:space-between;width:100%;padding:4px 8px;background:#c0c0c0;border:2px inset var(--silver-dark);box-sizing:border-box;font-family:var(--font);font-size:12px;">
    <div>Score: <span id="snake-score" style="font-weight:bold;">0</span></div>
    <div>High Score: <span id="snake-high-score" style="font-weight:bold;">0</span></div>
    <div id="snake-status" style="color:blue;font-weight:bold;">PLAYING</div>
  </div>
  <div style="position:relative;width:300px;height:300px;border:2px inset var(--silver-dark);background:#000;">
    <canvas id="snake-canvas" width="300" height="300" style="display:block;"></canvas>
    <div id="snake-overlay" style="display:none;position:absolute;inset:0;background:rgba(0,0,0,0.8);color:#fff;font-family:var(--font);flex-direction:column;align-items:center;justify-content:center;gap:12px;user-select:none;">
      <div id="snake-overlay-title" style="font-size:20px;font-weight:bold;color:red;">GAME OVER</div>
      <div style="font-size:12px;">Press SPACE to restart</div>
    </div>
  </div>
  <div style="display:flex;gap:8px;width:100%;justify-content:center;">
    <button class="win95-btn" id="snake-btn-pause" style="padding:4px 16px;">Pause</button>
    <button class="win95-btn" id="snake-btn-reset" style="padding:4px 16px;">Reset</button>
  </div>
</div>
"#);

    let canvas_el = parent.query_selector("#snake-canvas").unwrap().unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas_el.dyn_into().unwrap();
    let ctx = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

    let doc = parent.owner_document().unwrap();
    let score_el = doc.get_element_by_id("snake-score").unwrap();
    let high_el = doc.get_element_by_id("snake-high-score").unwrap();
    let status_el = doc.get_element_by_id("snake-status").unwrap();
    let overlay_el = doc.get_element_by_id("snake-overlay").unwrap();

    // Load High Score from localStorage
    let mut saved_high = 0;
    if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
        if let Ok(Some(val)) = storage.get_item("snake_high_score") {
            if let Ok(num) = val.parse::<u32>() {
                saved_high = num;
                high_el.set_inner_html(&num.to_string());
            }
        }
    }

    // Reset game state
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.body = vec![(10, 10), (9, 10), (8, 10)];
        state.direction = (1, 0);
        state.next_direction = (1, 0);
        state.score = 0;
        state.high_score = saved_high;
        state.game_over = false;
        state.paused = false;
        state.speed_ms = 150;
        state.timer_handle = None;
        spawn_food(&mut state);
    });

    draw_game(&canvas, &ctx);

    // Pause Button Click
    let p_status = status_el.clone();
    let p_canvas = canvas.clone();
    let p_ctx = ctx.clone();
    let p_doc = doc.clone();
    let pause_btn = doc.get_element_by_id("snake-btn-pause").unwrap();
    let pause_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            if state.game_over { return; }
            state.paused = !state.paused;
            if state.paused {
                p_status.set_inner_html("PAUSED");
                state.timer_handle = None;
            } else {
                p_status.set_inner_html("PLAYING");
                start_game_loop(&p_canvas, &p_ctx, &p_doc);
            }
        });
    });
    pause_btn.add_event_listener_with_callback("click", pause_cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(pause_cb);

    // Reset Button Click
    let r_status = status_el.clone();
    let r_score = score_el.clone();
    let r_overlay = overlay_el.clone();
    let r_canvas = canvas.clone();
    let r_ctx = ctx.clone();
    let r_doc = doc.clone();
    let reset_btn = doc.get_element_by_id("snake-btn-reset").unwrap();
    let reset_cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        reset_game(&r_status, &r_score, &r_overlay, &r_canvas, &r_ctx, &r_doc);
    });
    reset_btn.add_event_listener_with_callback("click", reset_cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(reset_cb);

    // Keydown handler
    let k_status = status_el.clone();
    let k_score = score_el.clone();
    let k_overlay = overlay_el.clone();
    let k_canvas = canvas.clone();
    let k_ctx = ctx.clone();
    let k_doc = doc.clone();
    let kb_cb = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        let mut handled = false;

        STATE.with(|s| {
            let mut state = s.borrow_mut();
            if state.game_over {
                if key == " " {
                    handled = true;
                }
                return;
            }

            match key.as_str() {
                "ArrowUp" | "w" | "W" => {
                    if state.direction.1 == 0 {
                        state.next_direction = (0, -1);
                    }
                    handled = true;
                }
                "ArrowDown" | "s" | "S" => {
                    if state.direction.1 == 0 {
                        state.next_direction = (0, 1);
                    }
                    handled = true;
                }
                "ArrowLeft" | "a" | "A" => {
                    if state.direction.0 == 0 {
                        state.next_direction = (-1, 0);
                    }
                    handled = true;
                }
                "ArrowRight" | "d" | "D" => {
                    if state.direction.0 == 0 {
                        state.next_direction = (1, 0);
                    }
                    handled = true;
                }
                "p" | "P" => {
                    state.paused = !state.paused;
                    if state.paused {
                        k_status.set_inner_html("PAUSED");
                        state.timer_handle = None;
                    } else {
                        k_status.set_inner_html("PLAYING");
                        start_game_loop(&k_canvas, &k_ctx, &k_doc);
                    }
                    handled = true;
                }
                _ => {}
            }
        });

        if handled {
            ev.prevent_default();
            if key == " " {
                reset_game(&k_status, &k_score, &k_overlay, &k_canvas, &k_ctx, &k_doc);
            }
        }
    });
    doc.add_event_listener_with_callback("keydown", kb_cb.as_ref().unchecked_ref()).unwrap();
    crate::app_state::store_closure(kb_cb);

    // Start game loop
    start_game_loop(&canvas, &ctx, &doc);
}

fn reset_game(
    status_el: &web_sys::Element,
    score_el: &web_sys::Element,
    overlay_el: &web_sys::Element,
    canvas: &web_sys::HtmlCanvasElement,
    ctx: &web_sys::CanvasRenderingContext2d,
    doc: &web_sys::Document,
) {
    status_el.set_inner_html("PLAYING");
    score_el.set_inner_html("0");
    overlay_el.set_attribute("style", "display:none;").unwrap();

    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.body = vec![(10, 10), (9, 10), (8, 10)];
        state.direction = (1, 0);
        state.next_direction = (1, 0);
        state.score = 0;
        state.game_over = false;
        state.paused = false;
        state.speed_ms = 150;
        state.timer_handle = None;
        spawn_food(&mut state);
    });

    draw_game(canvas, ctx);
    start_game_loop(canvas, ctx, doc);
}

fn spawn_food(state: &mut SnakeState) {
    let mut rng = js_sys::Math::random;
    loop {
        let x = (rng() * GRID_SIZE as f64) as i32;
        let y = (rng() * GRID_SIZE as f64) as i32;
        // Don't spawn food on the snake body
        if !state.body.contains(&(x, y)) {
            state.food = (x, y);
            break;
        }
    }
}

fn start_game_loop(
    canvas: &web_sys::HtmlCanvasElement,
    ctx: &web_sys::CanvasRenderingContext2d,
    doc: &web_sys::Document,
) {
    let window = web_sys::window().unwrap();
    let c = canvas.clone();
    let cx = ctx.clone();
    let d = doc.clone();

    let cb = Closure::<dyn FnMut()>::new(move || {
        let mut should_continue = false;
        let mut game_over = false;
        let mut speed = 150;

        STATE.with(|s| {
            let mut state = s.borrow_mut();
            if state.paused || state.game_over {
                return;
            }

            // Update direction
            state.direction = state.next_direction;

            // Move snake
            let head = state.body[0];
            let next_head = (
                head.0 + state.direction.0,
                head.1 + state.direction.1
            );

            // Check walls collision
            if next_head.0 < 0 || next_head.0 >= GRID_SIZE || next_head.1 < 0 || next_head.1 >= GRID_SIZE {
                state.game_over = true;
                game_over = true;
                return;
            }

            // Check self collision
            if state.body.contains(&next_head) {
                state.game_over = true;
                game_over = true;
                return;
            }

            // Insert new head
            state.body.insert(0, next_head);

            // Check food collision
            if next_head == state.food {
                state.score += 10;
                // Save high score if beaten
                if state.score > state.high_score {
                    state.high_score = state.score;
                    if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                        let _ = storage.set_item("snake_high_score", &state.score.to_string());
                    }
                }
                spawn_food(&mut state);
                // Speed up slightly
                state.speed_ms = (150 - (state.score / 20) * 5).max(60);
            } else {
                // Remove tail
                state.body.pop();
            }

            should_continue = true;
            speed = state.speed_ms;
        });

        // Draw new state
        draw_game(&c, &cx);

        // Update DOM elements
        if let Some(score_val) = d.get_element_by_id("snake-score") {
            let score = STATE.with(|s| s.borrow().score);
            score_val.set_inner_html(&score.to_string());
        }
        if let Some(high_val) = d.get_element_by_id("snake-high-score") {
            let high = STATE.with(|s| s.borrow().high_score);
            high_val.set_inner_html(&high.to_string());
        }

        if game_over {
            if let Some(overlay) = d.get_element_by_id("snake-overlay") {
                overlay.set_attribute("style", "display:flex;").unwrap();
            }
            if let Some(status) = d.get_element_by_id("snake-status") {
                status.set_inner_html("GAME OVER");
            }
        }

        if should_continue {
            STATE.with(|s| {
                let s_ref = s.borrow();
                if let Some(h) = &s_ref.timer_handle {
                    if let Some(w) = web_sys::window() {
                        let fn_ref = h.clone();
                        let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                            fn_ref.unchecked_ref(), speed as i32
                        );
                    }
                }
            });
        }
    });

    let fn_ref = cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
    crate::app_state::store_closure(cb);
    
    // Schedule first tick
    let speed = STATE.with(|s| s.borrow().speed_ms);
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&fn_ref, speed as i32);
    
    STATE.with(|s| {
        s.borrow_mut().timer_handle = Some(fn_ref);
    });
}

fn draw_game(canvas: &web_sys::HtmlCanvasElement, ctx: &web_sys::CanvasRenderingContext2d) {
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;
    let cell_w = w / GRID_SIZE as f64;
    let cell_h = h / GRID_SIZE as f64;

    // Clear board
    ctx.set_fill_style(&"#000000".into());
    ctx.fill_rect(0.0, 0.0, w, h);

    STATE.with(|s| {
        let state = s.borrow();

        // Draw food (Retro red square with inner border)
        let fx = state.food.0 as f64 * cell_w;
        let fy = state.food.1 as f64 * cell_h;
        ctx.set_fill_style(&"#ff0000".into());
        ctx.fill_rect(fx, fy, cell_w, cell_h);
        ctx.set_stroke_style(&"#ff8888".into());
        ctx.set_line_width(1.0);
        ctx.stroke_rect(fx + 1.0, fy + 1.0, cell_w - 2.0, cell_h - 2.0);

        // Draw snake
        for (i, segment) in state.body.iter().enumerate() {
            let sx = segment.0 as f64 * cell_w;
            let sy = segment.1 as f64 * cell_h;

            if i == 0 {
                // Head: bright lime green
                ctx.set_fill_style(&"#00ff00".into());
            } else {
                // Body: forest green
                ctx.set_fill_style(&"#008800".into());
            }
            ctx.fill_rect(sx, sy, cell_w, cell_h);

            // Give segments a subtle grid separator
            ctx.set_stroke_style(&"#000000".into());
            ctx.set_line_width(1.0);
            ctx.stroke_rect(sx, sy, cell_w, cell_h);
        }
    });
}
