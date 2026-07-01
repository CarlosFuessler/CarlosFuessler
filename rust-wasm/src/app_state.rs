use std::cell::RefCell;
use std::rc::Rc;

use crate::vfs::VirtualFS;

pub struct AppState {
    pub window_manager: crate::window_manager::WindowManager,
    pub taskbar: Option<crate::taskbar::Taskbar>,
    pub vfs: Option<Rc<VirtualFS>>,
    /// Holds closures so the JS event listeners keep working.
    pub closures: Vec<Box<dyn std::any::Any>>,
}

thread_local! {
    pub static APP_STATE: RefCell<Option<Rc<RefCell<AppState>>>> =
        const { RefCell::new(None) };
    static CLOSURES: RefCell<Vec<Box<dyn std::any::Any>>> =
        const { RefCell::new(Vec::new()) };
}

pub fn init_app_state(document: web_sys::Document) {
    let wm = crate::window_manager::WindowManager::new(document);
    let state = Rc::new(RefCell::new(AppState {
        window_manager: wm,
        taskbar: None,
        vfs: None,
        closures: Vec::new(),
    }));
    APP_STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}

/// Store a closure so it lives forever (prevents GC of JS event handlers).
/// Uses its own thread-local storage to avoid re-entrancy panics with APP_STATE.
pub fn store_closure<T: 'static>(c: T) {
    CLOSURES.with(|closures| {
        closures.borrow_mut().push(Box::new(c));
    });
}

pub fn with_app<F, R>(f: F) -> R
where
    F: FnOnce(&mut AppState) -> R,
{
    APP_STATE.with(|s| {
        let state = s.borrow();
        let state = state.as_ref().expect("AppState not initialized");
        let mut app = state.borrow_mut();
        f(&mut app)
    })
}

pub fn with_wm<F, R>(f: F) -> R
where
    F: FnOnce(&mut crate::window_manager::WindowManager) -> R,
{
    with_app(|app| f(&mut app.window_manager))
}

pub fn with_taskbar<F, R>(f: F) -> R
where
    F: FnOnce(&mut crate::taskbar::Taskbar) -> R,
{
    with_app(|app| f(app.taskbar.as_mut().expect("Taskbar not initialized")))
}

pub fn set_taskbar(tb: crate::taskbar::Taskbar) {
    with_app(|app| {
        app.taskbar = Some(tb);
    });
}

pub fn set_vfs(vfs: VirtualFS) {
    with_app(|app| {
        app.vfs = Some(Rc::new(vfs));
    });
}

// ---------------------------------------------------------------
// Safe window ops — taskbar ops done OUTSIDE the with_wm borrow
// so we never double-borrow APP_STATE.
// ---------------------------------------------------------------

pub fn create_window(app_id: &str, title: &str, w: u32, h: u32) -> u32 {
    let id = with_wm(|wm| wm.create_window(app_id, title, w, h));
    with_taskbar(|tb| tb.add_app_button(&format!("win-{}", id), title));
    crate::clippy::on_app_open(app_id);
    id
}

pub fn close_window(id: u32) {
    with_wm(|wm| wm.close_window(id));
    with_taskbar(|tb| tb.remove_app_button(&format!("win-{}", id)));
}

pub fn minimize_window(id: u32) {
    with_wm(|wm| wm.minimize_window(id));
}

pub fn maximize_window(id: u32) {
    with_wm(|wm| wm.maximize_window(id));
}

pub fn focus_window(id: u32) {
    with_wm(|wm| wm.focus_window_inner(id));
    with_taskbar(|tb| tb.set_active(&format!("win-{}", id)));
}
