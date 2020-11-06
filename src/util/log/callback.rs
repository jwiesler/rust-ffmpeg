use super::Level;
use std::convert::{TryFrom};
use std::str::from_utf8_unchecked;
use std::ffi::CStr;
use std::sync::{Arc, RwLock};
use super::{set_raw_callback, set_default_callback};

#[cfg(target_family = "unix")]
pub type Args = *mut sys::__va_list_tag;

#[cfg(target_family = "windows")]
pub type Args = sys::va_list;

pub type Context = *mut libc::c_void;
pub type c_int = libc::c_int;
pub type c_char = libc::c_char;
pub type RawCallback = unsafe extern "C" fn(context: Context, level: c_int, fmt: *const c_char, args: Args);

pub fn call_with<F: Fn(Context, Level, &str, Args)>(context: Context, level: libc::c_int, fmt: *const libc::c_char, args: Args, f: F) {
    unsafe {
        let fmt = from_utf8_unchecked(CStr::from_ptr(fmt).to_bytes());
        let level = Level::try_from(level).unwrap_or(Level::Info);
        f(context, level, fmt, args)
    }
}

pub trait LogCallback: Send + Sync {
    fn log(&self, avcl: Context, level: Level, fmt: &str, args: Args);
}

lazy_static! {
    static ref CALLBACK_CONTEXT: RwLock<Option<Arc<dyn LogCallback>>> = RwLock::new(None);
}

extern "C" fn context_callback(context: Context, level: c_int, fmt: * const c_char, args: Args) {
    call_with(context, level, fmt, args, |context, level, fmt, args| {
        CALLBACK_CONTEXT.read().unwrap().as_ref().map(|callback| callback.log(context, level, fmt, args));
    });
}

fn set_context(callback: Arc<dyn LogCallback>) {
    *CALLBACK_CONTEXT.write().unwrap() = Some(callback);
    set_raw_callback(context_callback);
}

fn clear_context() {
    *CALLBACK_CONTEXT.write().unwrap() = None;
    set_default_callback()
}

pub struct CallbackOwner {}

impl Drop for CallbackOwner {
    fn drop(&mut self) {
        clear_context()
    }
}

pub fn set_callback(callback: Arc<dyn LogCallback>) -> CallbackOwner {
    set_context(callback);
    CallbackOwner {}
}
