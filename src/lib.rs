#![feature(convert)]
#![allow(raw_pointer_derive, non_camel_case_types)]

extern crate libc;
extern crate ffmpeg_sys as ffi;
#[macro_use] extern crate bitflags;

pub mod util;
pub use util::error::Error;
pub use util::dictionary::Dictionary;