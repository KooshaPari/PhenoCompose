#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Try constructing a SecretRef; reject invalid input.
        let trimmed = s.trim();
        if !trimmed.is_empty() && !trimmed.contains('\0') && trimmed.len() < 1024 {
            let r = phenocompose_port_types::SecretRef::new(trimmed);
            let _ = r.locator();
        }
    }
});
