use std::ffi::c_void;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

use crate::terminal::{TerminalGeometry, TerminalOptions};

static WAKEUP_CALLBACKS_ENABLED: AtomicBool = AtomicBool::new(true);

#[derive(Clone, Copy)]
pub struct WakeupCallback {
    callback: extern "C" fn(*mut c_void),
    user_data: usize,
}

impl WakeupCallback {
    pub(crate) fn raw_parts(self) -> (extern "C" fn(*mut c_void), *mut c_void) {
        (self.callback, self.user_data as *mut c_void)
    }
    pub fn call(self) {
        if WAKEUP_CALLBACKS_ENABLED.load(Ordering::Acquire) {
            (self.callback)(self.user_data as *mut c_void);
        }
    }

    pub fn new(callback: extern "C" fn(*mut c_void), user_data: *mut c_void) -> Self {
        Self {
            callback,
            user_data: user_data as usize,
        }
    }
}

pub fn enable_wakeup_callbacks() {
    WAKEUP_CALLBACKS_ENABLED.store(true, Ordering::Release);
}
pub fn disable_wakeup_callbacks() {
    WAKEUP_CALLBACKS_ENABLED.store(false, Ordering::Release);
}

pub(crate) fn join_worker(worker: &mut Option<JoinHandle<()>>, _name: &str) {
    if let Some(handle) = worker.take() {
        let _ = handle.join();
    }
}

pub const ZSH_INTEGRATION: &str = "";
pub const BASH_INTEGRATION: &str = "";
pub const FISH_INTEGRATION: &str = "";

pub struct LocalPty;
pub struct PtyPump {
    pub output: Vec<u8>,
    pub exited: bool,
    pub has_more: bool,
}

impl LocalPty {
    pub fn spawn(_: TerminalGeometry, _: &TerminalOptions) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "local PTY is unavailable on Android",
        ))
    }
    pub fn resize(&mut self, _: TerminalGeometry) {}
    pub fn set_wakeup_callback(&mut self, _: Option<WakeupCallback>) {}
    pub fn queue_input(&mut self, _: &[u8]) {}
    pub fn drain_output(&mut self) -> PtyPump {
        PtyPump {
            output: Vec::new(),
            exited: true,
            has_more: false,
        }
    }
    pub fn input_visible(&self) -> bool {
        false
    }
    pub fn clear_pending_output(&mut self) {}
}
