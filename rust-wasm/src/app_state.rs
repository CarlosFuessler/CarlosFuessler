use std::cell::RefCell;
use std::rc::Rc;

pub struct AppState {
    pub window_manager: crate::window_manager::WindowManager,
    pub taskbar: Option<crate::taskbar::Taskbar>,
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
    }));
    APP_STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}

pub fn with_wm<F, R>(f: F) -> R
where
    F: FnOnce(&mut crate::window_manager::WindowManager) -> R,
{
    APP_STATE.with(|s| {
        let state = s.borrow();
        let state = state.as_ref().expect("AppState not initialized");
        let mut app = state.borrow_mut();
        f(&mut app.window_manager)
    })
}

pub fn with_taskbar<F, R>(f: F) -> R
where
    F: FnOnce(&mut crate::taskbar::Taskbar) -> R,
{
    APP_STATE.with(|s| {
        let state = s.borrow();
        let state = state.as_ref().expect("AppState not initialized");
        let mut app = state.borrow_mut();
        f(app.taskbar.as_mut().expect("Taskbar not initialized"))
    })
}

pub fn set_taskbar(tb: crate::taskbar::Taskbar) {
    APP_STATE.with(|s| {
        let state = s.borrow();
        let state = state.as_ref().expect("AppState not initialized");
        let mut app = state.borrow_mut();
        app.taskbar = Some(tb);
    });
}
