pub mod level;

pub use self::level::Level;

pub mod flag;

pub use self::flag::Flags;

use ffi::*;
use std::convert::TryInto;

pub fn set_level(value: Level) {
    unsafe { av_log_set_level(value.into()) }
}

pub fn get_level() -> Result<Level, &'static str> {
    unsafe { av_log_get_level().try_into() }
}

pub fn set_flags(value: Flags) {
    unsafe { av_log_set_flags(value.bits()) }
}

pub fn get_flags() -> Flags {
    unsafe { Flags::from_bits_truncate(av_log_get_flags()) }
}

pub use libc::c_int;
pub use libc::c_char;

pub type Context = *mut libc::c_void;

#[cfg(all(target_arch = "x86_64", target_family = "windows"))]
pub type Args = sys::va_list;

#[cfg(all(target_arch = "x86_64", target_family = "unix"))]
pub type Args = *mut sys::__va_list_tag;

/// Type of a log callback
pub type Callback = unsafe extern "C" fn(context: Context, level: c_int, fmt: *const c_char, args: Args);

/// Sets a log callback
pub fn set_raw_callback(callback: Callback) {
    unsafe {
        sys::av_log_set_callback(Some(callback));
    };
}

/// Calls the default log function of ffmpeg
pub unsafe extern "C" fn default_unsafe_callback(context: Context, level: c_int, fmt: *const c_char, args: Args) {
    sys::av_log_default_callback(context, level, fmt, args);
}

/// Uses the log crate
/// Use the feature `respect-ffmpeg-log-level` to control whether the log level set by ffmpeg will be checked before outputting
pub unsafe extern "C" fn default_callback(_context: Context, level: c_int, fmt: *const c_char, args: Args) {
    let level = level.try_into().unwrap_or(Level::Info);
    #[cfg(feature = "respect-ffmpeg-log-level")]
    if let Ok(enabled_level) = get_level() {
        if level < enabled_level {
            return;
        }
    }

    let level_filter = match level {
        Level::Quiet => log::LevelFilter::Off,
        Level::Trace => log::LevelFilter::Trace,
        Level::Debug | Level::Verbose => log::LevelFilter::Debug,
        Level::Info => log::LevelFilter::Info,
        Level::Warning => log::LevelFilter::Warn,
        Level::Error | Level::Fatal | Level::Panic => log::LevelFilter::Error,
    };

    if let Some(log_level) = level_filter.to_level() {
        if log::log_enabled!(log_level) {
            let log = vsprintf::vsprintf(fmt, args).expect("failed to format ffmpeg log message");
            log::log!(log_level, "{}", log.trim());
        }
    }
}
