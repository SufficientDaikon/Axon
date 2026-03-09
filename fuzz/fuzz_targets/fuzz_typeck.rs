//! Fuzz target: Axon type checker
//!
//! Feeds random bytes through the full lex → parse → typecheck pipeline.
//! The compiler should never panic on arbitrary input — only produce errors.
//!
//! Run with:
//!   cargo fuzz run fuzz_typeck -- -max_len=10000 -max_total_time=300

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Only proceed if the data is valid UTF-8
    if let Ok(source) = std::str::from_utf8(data) {
        // Run the full front-end pipeline: lex → parse → typecheck → borrow check
        // None of these stages should ever panic on arbitrary input.
        let _ = axonc::check_source(source, "fuzz.axon");
    }
});
