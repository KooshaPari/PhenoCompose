#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Try parsing the string as a manifest name; accept any outcome.
        // Real parser would be more involved; this is a smoke target.
        let _ = s.len();
    }
});
