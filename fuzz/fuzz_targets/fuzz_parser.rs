//! Fuzz target: Axon parser
//!
//! Feeds random bytes to the lexer+parser pipeline.
//! The compiler should never panic on arbitrary input — only produce errors.
//!
//! Run with:
//!   cargo fuzz run fuzz_parser -- -max_len=10000 -max_total_time=300

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Only proceed if the data is valid UTF-8 (the parser expects strings)
    if let Ok(source) = std::str::from_utf8(data) {
        // The parser should never panic, regardless of input
        let _ = axonc::parse_source(source, "fuzz.axon");
    }
});
