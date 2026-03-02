pub mod debug;
pub mod filesystem;
pub mod error;
pub mod location;

pub use debug::*;
pub use filesystem::*;
pub use error::*;
pub use location::*;

/// Strip leading and trailing double quotes from a string, if present.
pub fn strip_quotes(s: &str) -> &str {
    s.strip_prefix('"')
     .and_then(|s| s.strip_suffix('"'))
     .unwrap_or(s)
}

/// strip leading and trailing str from a string, if present.
pub fn strip_str<'a>(s: &'a str, pre_suf: &'a str) -> &'a str {
    s.strip_prefix(pre_suf)
     .and_then(|s| s.strip_suffix(pre_suf))
     .unwrap_or(s)
}

/// convert string to PascalCase
pub fn pascal(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// true if string can fit into i64, false if it can fit into u64 but not i64, error if it exceeds u64
pub (crate) fn get_suitable_type(s: &str) -> Result<bool, String> {
    // 1. Determine the radix (base) and strip the prefix
    let (input, radix) = if s.starts_with("0x") || s.starts_with("0X") {
        (&s[2..], 16)
    } else if s.starts_with("0o") || s.starts_with("0O") {
        (&s[2..], 8)
    } else if s.starts_with("0b") || s.starts_with("0B") {
        (&s[2..], 2)
    } else {
        (s, 10)
    };

    // 2. Parse into a u128 to safely hold any 64-bit value (signed or unsigned)
    // Note: This logic assumes non-negative inputs for hex/bin/oct strings.
    let val = u128::from_str_radix(input, radix)
        .map_err(|_| format!("Invalid number format for base {}", radix))?;

    // 3. Check boundaries
    // true = fits in i64, false = needs u64
    if val <= i64::MAX as u128 {
        Ok(true)
    } else if val <= u64::MAX as u128 {
        Ok(false)
    } else {
        Err("Value exceeds 64-bit unsigned maximum".to_string())
    }
}