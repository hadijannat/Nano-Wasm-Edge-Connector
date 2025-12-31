//! Nano-Wasm Edge Connector - Guest Policy Module
//!
//! A minimal no_std WebAssembly module for policy evaluation.
//! Uses explicit memory definition.

#![no_std]

use core::slice;

// Fixed buffer location - host writes request data here
const INPUT_BUFFER_OFFSET: usize = 1024; // After first 1KB

// Host function imports
#[link(wasm_import_module = "host")]
extern "C" {
    /// Log a message to the host
    fn log(ptr: i32, len: i32);
}

/// Helper to log messages to host
fn host_log(msg: &str) {
    unsafe { log(msg.as_ptr() as i32, msg.len() as i32) }
}

/// Get the input buffer pointer for host to write data
#[no_mangle]
pub extern "C" fn get_input_buffer() -> i32 {
    INPUT_BUFFER_OFFSET as i32
}

/// Main policy evaluation entry point
#[no_mangle]
pub extern "C" fn evaluate_access(ptr: i32, len: i32) -> i32 {
    if len <= 0 || len > 8192 || ptr < 0 {
        return 0; // Invalid parameters
    }
    
    let data = unsafe { slice::from_raw_parts(ptr as *const u8, len as usize) };
    
    // Rule 1: Blocked requests are denied
    if pattern_match(data, b"\"blocked\":true") {
        host_log("Access DENIED: blocked flag present");
        return 0;
    }
    
    // Rule 2: Admin role always allowed
    if pattern_match(data, b"\"admin\"") {
        host_log("Access GRANTED: admin role detected");
        return 1;
    }
    
    // Rule 3: Operator role with restrictions
    if pattern_match(data, b"\"operator\"") {
        if pattern_match(data, b"\"secret\"") {
            host_log("Access DENIED: operator cannot access sensitive");
            return 0;
        }
        host_log("Access GRANTED: operator role");
        return 1;
    }
    
    // Rule 4: Viewer - read only
    if pattern_match(data, b"\"viewer\"") {
        if pattern_match(data, b"\"write\"") {
            host_log("Access DENIED: viewer cannot write");
            return 0;
        }
        host_log("Access GRANTED: viewer read-only access");
        return 1;
    }
    
    // Default policy: allow
    host_log("Access GRANTED: default policy");
    1
}

/// Simple byte pattern matching
fn pattern_match(hay: &[u8], needle: &[u8]) -> bool {
    if needle.len() > hay.len() {
        return false;
    }
    let mut i = 0;
    while i <= hay.len() - needle.len() {
        let mut found = true;
        let mut j = 0;
        while j < needle.len() {
            if hay[i + j] != needle[j] {
                found = false;
                break;
            }
            j += 1;
        }
        if found {
            return true;
        }
        i += 1;
    }
    false
}

// Panic handler for no_std
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
