//! This file is part of the easybox package.
//
// (c) Zhihua Zhao <YuukaC@outlook.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

//! FFI bindings for the `libmagic` library.
//!
//! https://man7.org/linux/man-pages/man3/libmagic.3.html

use libc::{c_char, c_int, c_void, size_t};
use std::{ffi::CStr, ptr::null};

/// No special handling.
pub const MAGIC_NONE: c_int = 0x0000000;
/// Print debugging messages to stderr.
pub const MAGIC_DEBUG: c_int = 0x0000001;
/// If the file queried is a symlink, follow it.
pub const MAGIC_SYMLINK: c_int = 0x0000002;
/// If the file is compressed, unpack it and look at the contents.
pub const MAGIC_COMPRESS: c_int = 0x0000004;
/// If the file is a block or character special device, then open the device and try to look in its contents.
pub const MAGIC_DEVICES: c_int = 0x0000008;
/// Return a MIME type string, instead of a textual description.
pub const MAGIC_MIME_TYPE: c_int = 0x0000010;
/// Return all matches, not just the first.
pub const MAGIC_CONTINUE: c_int = 0x0000020;
/// Check  the  magic  database for consistency and print warnings to stderr.
pub const MAGIC_CHECK: c_int = 0x0000040;
/// On systems that support utime(3) or utimes(2), attempt to preserve the access time of files analysed.
pub const MAGIC_PRESERVE_ATIME: c_int = 0x0000080;
/// Don't translate unprintable characters to a \ooo octal representation.
pub const MAGIC_RAW: c_int = 0x0000100;
/// Treat operating system errors while trying to open files and follow symlinks as real errors, instead of printing them in the magic buffer.
pub const MAGIC_ERROR: c_int = 0x0000200;
/// Return a MIME encoding, instead of a textual description.
pub const MAGIC_MIME_ENCODING: c_int = 0x0000400;
/// A shorthand for MAGIC_MIME_TYPE | MAGIC_MIME_ENCODING.
pub const MAGIC_MIME: c_int = MAGIC_MIME_TYPE | MAGIC_MIME_ENCODING;
/// Return the Apple creator and type.
pub const MAGIC_APPLE: c_int = 0x0000800;
/// Return a slash-separated list of extensions for this file type.
pub const MAGIC_EXTENSION: c_int = 0x1000000;
/// Don't report on compression, only report about the uncompressed data.
pub const MAGIC_COMPRESS_TRANSP: c_int = 0x2000000;
/// Don't allow decompressors that use fork.
pub const MAGIC_NO_COMPRESS_FORK: c_int = 0x4000000;
///
pub const MAGIC_NODESC: c_int = MAGIC_EXTENSION | MAGIC_MIME | MAGIC_APPLE;

/// Don't look inside compressed files.
pub const MAGIC_NO_CHECK_COMPRESS: c_int = 0x0001000;
/// Don't examine tar files.
pub const MAGIC_NO_CHECK_TAR: c_int = 0x0002000;
/// Don't consult magic files.
pub const MAGIC_NO_CHECK_SOFT: c_int = 0x0004000;
/// Don't check for EMX application type (only on EMX).
pub const MAGIC_NO_CHECK_APPTYPE: c_int = 0x0008000;
/// Don't print ELF details.
pub const MAGIC_NO_CHECK_ELF: c_int = 0x0010000;
/// Don't check for various types of text files.
pub const MAGIC_NO_CHECK_TEXT: c_int = 0x0020000;
/// Don't get extra information on MS Composite Document Files.
pub const MAGIC_NO_CHECK_CDF: c_int = 0x0040000;
/// Don't examine CSV files.
pub const MAGIC_NO_CHECK_CSV: c_int = 0x0080000;
/// Don't look for known tokens inside ascii files.
pub const MAGIC_NO_CHECK_TOKENS: c_int = 0x0100000;
/// Don't check text encodings.
pub const MAGIC_NO_CHECK_ENCODING: c_int = 0x0200000;
/// Don't examine JSON files.
pub const MAGIC_NO_CHECK_JSON: c_int = 0x0400000;
/// Don't examine SIMH tape files.
pub const MAGIC_NO_CHECK_SIMH: c_int = 0x0800000;

/// No built-in tests; only consult the magic file
pub const MAGIC_NO_CHECK_BUILTIN: c_int = MAGIC_NO_CHECK_COMPRESS
    | MAGIC_NO_CHECK_TAR
    // | MAGIC_NO_CHECK_SOFT
    | MAGIC_NO_CHECK_APPTYPE
    | MAGIC_NO_CHECK_ELF
    | MAGIC_NO_CHECK_TEXT
    | MAGIC_NO_CHECK_CSV
    | MAGIC_NO_CHECK_CDF
    | MAGIC_NO_CHECK_TOKENS
    | MAGIC_NO_CHECK_ENCODING
    | MAGIC_NO_CHECK_JSON
    | MAGIC_NO_CHECK_SIMH;

/// Defined for backwards compatibility (renamed)
#[deprecated]
pub const MAGIC_NO_CHECK_ASCII: c_int = MAGIC_NO_CHECK_TEXT;

/// Defined for backwards compatibility (do nothing)
/// Don't check ascii/fortran
#[deprecated]
pub const MAGIC_NO_CHECK_FORTRAN: c_int = 0x000000;
/// Defined for backwards compatibility (do nothing)
/// Don't check ascii/troff
#[deprecated]
pub const MAGIC_NO_CHECK_TROFF: c_int = 0x000000;

///
pub const MAGIC_PARAM_INDIR_MAX: c_int = 0;
///
pub const MAGIC_PARAM_NAME_MAX: c_int = 1;
///
pub const MAGIC_PARAM_ELF_PHNUM_MAX: c_int = 2;
///
pub const MAGIC_PARAM_ELF_SHNUM_MAX: c_int = 3;
///
pub const MAGIC_PARAM_ELF_NOTES_MAX: c_int = 4;
///
pub const MAGIC_PARAM_REGEX_MAX: c_int = 5;
///
pub const MAGIC_PARAM_BYTES_MAX: c_int = 6;
///
pub const MAGIC_PARAM_ENCODING_MAX: c_int = 7;
///
pub const MAGIC_PARAM_ELF_SHSIZE_MAX: c_int = 8;
///
pub const MAGIC_PARAM_MAGWARN_MAX: c_int = 9;

// Used as the `action` parameter in magic_getpath function
///
pub const FILE_LOAD: c_int = 0;
///
pub const FILE_CHECK: c_int = 1;
///
pub const FILE_COMPILE: c_int = 2;
///
pub const FILE_LIST: c_int = 3;

///
pub const DEFAULT_BYTES_LIMIT: size_t = 1024 * 1024;
///
pub const DEFAULT_ELF_NOTES_LIMIT: size_t = 256;
///
pub const DEFAULT_ELF_PHNUM_LIMIT: size_t = 2 * 1024;
///
pub const DEFAULT_ELF_SHNUM_LIMIT: size_t = 32 * 1024;
///
pub const DEFAULT_ELF_SHSIZE_LIMIT: size_t = 128 * 1024 * 1024;
///
pub const DEFAULT_ENCODING_LIMIT: size_t = 65 * 1024;
///
pub const DEFAULT_INDIR_LIMIT: size_t = 50;
///
pub const DEFAULT_NAME_LIMIT: size_t = 50;
///
pub const DEFAULT_REGEX_LIMIT: size_t = 8 * 1024;
///
pub const DEFAULT_MAGWARN_LIMIT: size_t = 64;

// https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
///
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct magic_set {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

///
#[allow(non_camel_case_types)]
pub type magic_t = *mut magic_set;

mod libmagic {
    use super::*;

    #[link(name = "magic")]
    extern "C" {
        ///
        pub fn magic_open(flags: c_int) -> magic_t;
        /// Close the magic(5) database and deallocates any resources used.
        pub fn magic_close(cookie: magic_t);

        /// Return a textual description of the contents of the filename argument, or NULL if an error occurred. If the filename is NULL, then stdin is used.
        pub fn magic_file(cookie: magic_t, filename: *const c_char) -> *const c_char;

        /// Return a textual explanation of the last error, or NULL if there was no error.
        pub fn magic_error(cookie: magic_t) -> *const c_char;

        /// Must be used to load the colon separated list of database files passed in as filename, or NULL for the default database file before any magic queries can performed.
        /// The default database file is named by the MAGIC environment variable. If that variable is not set, the default database file name is /usr/share/file/misc/magic. magic_load() adds ".mgc" to the database filename as appropriate.
        pub fn magic_load(cookie: magic_t, filename: *const c_char) -> c_int;

        /// Can be used to compile the colon separated list of database files passed in as filename, or NULL for the default database. It returns 0 on success and -1 on failure. The compiled files created are named from the basename(1) of each file argument with “.mgc” appended to it.
        pub fn magic_compile(cookie: magic_t, filename: *const c_char) -> c_int;
        /// Can be used to check the validity of entries in the colon separated database files passed in as filename, or NULL for the default database. It returns 0 on success and -1 on failure.
        pub fn magic_check(cookie: magic_t, filename: *const c_char) -> c_int;
        /// Dump all magic entries in a human readable format, dumping first the entries that are matched against binary files and then the ones that match text files. It takes and optional filename argument which is a colon separated list of database files, or NULL for the default database.
        pub fn magic_list(cookie: magic_t, filename: *const c_char) -> c_int;

        ///
        pub fn magic_setparam(cookie: magic_t, param: c_int, value: *const c_void) -> c_int;
    }
}

///
pub fn magic_open(flags: i32) -> magic_t {
    unsafe { libmagic::magic_open(flags) }
}

///
pub fn magic_close(cookie: magic_t) {
    unsafe { libmagic::magic_close(cookie) }
}

///
pub fn magic_file(cookie: magic_t, filename: Option<&str>) -> Option<String> {
    unsafe {
        if let Some(s) = filename {
            let c_str = std::ffi::CString::new(s).unwrap();
            pointer_to_string(libmagic::magic_file(cookie, c_str.as_ptr()))
        } else {
            pointer_to_string(libmagic::magic_file(cookie, null()))
        }
    }
}

///
pub fn magic_error(cookie: magic_t) -> Option<String> {
    unsafe { pointer_to_string(libmagic::magic_error(cookie)) }
}

///
pub fn magic_load(cookie: magic_t, filename: Option<&str>) -> i32 {
    unsafe {
        if let Some(s) = filename {
            let c_str = std::ffi::CString::new(s).unwrap();
            libmagic::magic_load(cookie, c_str.as_ptr())
        } else {
            libmagic::magic_load(cookie, null())
        }
    }
}

///
pub fn magic_compile(cookie: magic_t, filename: Option<&str>) -> i32 {
    unsafe {
        if let Some(s) = filename {
            let c_str = std::ffi::CString::new(s).unwrap();
            libmagic::magic_compile(cookie, c_str.as_ptr())
        } else {
            libmagic::magic_compile(cookie, null())
        }
    }
}

///
pub fn magic_check(cookie: magic_t, filename: Option<&str>) -> i32 {
    unsafe {
        if let Some(s) = filename {
            let c_str = std::ffi::CString::new(s).unwrap();
            libmagic::magic_check(cookie, c_str.as_ptr())
        } else {
            libmagic::magic_check(cookie, null())
        }
    }
}

///
pub fn magic_list(cookie: magic_t, filename: Option<&str>) -> i32 {
    unsafe {
        if let Some(s) = filename {
            let c_str = std::ffi::CString::new(s).unwrap();
            libmagic::magic_list(cookie, c_str.as_ptr())
        } else {
            libmagic::magic_list(cookie, null())
        }
    }
}

///
pub fn magic_setparam(cookie: magic_t, param: i32, value: usize) -> i32 {
    unsafe { libmagic::magic_setparam(cookie, param, &value as *const usize as *const c_void) }
}

unsafe fn pointer_to_string(p: *const c_char) -> Option<String> {
    if p.is_null() {
        None
    } else {
        let c_str: &CStr = CStr::from_ptr(p);
        Some(c_str.to_str().unwrap().to_string())
    }
}
