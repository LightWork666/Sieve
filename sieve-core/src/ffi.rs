// FFI (C-ABI) exports for consumption by Go via CGo or any C-compatible language.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::{sift, SiftLevel};

/// Opaque result handle returned to C callers.
#[repr(C)]
pub struct CSiftResult {
    pub sifted: *mut c_char,
    pub tokens_removed: usize,
    /// Compression ratio × 10000 (integer representation to avoid float ABI issues).
    pub compression_ratio_bps: u32,
}

/// Sift a prompt string. The caller must free the result with `sieve_free_result`.
///
/// # Safety
/// - `prompt` must be a valid null-terminated UTF-8 C string.
/// - `level` must be one of: 0 (Low), 1 (Medium), 2 (High).
#[no_mangle]
pub unsafe extern "C" fn sieve_sift(prompt: *const c_char, level: u8) -> CSiftResult {
    let c_str = unsafe { CStr::from_ptr(prompt) };
    let prompt_str = c_str.to_str().unwrap_or("");

    let sift_level = match level {
        0 => SiftLevel::Low,
        2 => SiftLevel::High,
        _ => SiftLevel::Medium,
    };

    let result = sift(prompt_str, sift_level);

    let sifted_c = CString::new(result.sifted).unwrap_or_default();

    CSiftResult {
        sifted: sifted_c.into_raw(),
        tokens_removed: result.tokens_removed,
        compression_ratio_bps: (result.compression_ratio * 10000.0) as u32,
    }
}

/// Free a CSiftResult previously returned by `sieve_sift`.
///
/// # Safety
/// - `result` must have been obtained from `sieve_sift`.
/// - Must not be called more than once for the same result.
#[no_mangle]
pub unsafe extern "C" fn sieve_free_result(result: CSiftResult) {
    if !result.sifted.is_null() {
        drop(unsafe { CString::from_raw(result.sifted) });
    }
}
