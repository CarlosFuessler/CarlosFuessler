use std::cell::RefCell;
use std::rc::Rc;

use crate::vfs::VirtualFS;

pub struct AppState {
    pub window_manager: crate::window_manager::WindowManager,
    pub taskbar: Option<crate::taskbar::Taskbar>,
    pub vfs: Option<Rc<VirtualFS>>,
}

thread_local! {
    pub static APP_STATE: RefCell<Option<Rc<RefCell<AppState>>>> =
        const { RefCell::new(None) };
}

pub fn init_app_state(document: web_sys::Document) {
    let wm = crate::window_manager::WindowManager::new(document);
    let state = Rc::new(RefCell::new(AppState {
        window_manager: wm,
        taskbar: None,
        vfs: None,
    }));
    APP_STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}

/// General-purpose accessor that gives a mutable reference to the entire AppState.
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

/// Store the loaded VirtualFS into global state.
pub fn set_vfs(vfs: VirtualFS) {
    with_app(|app| {
        app.vfs = Some(Rc::new(vfs));
    });
}
