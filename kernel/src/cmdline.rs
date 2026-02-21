//! Kernel Command Line Parser
//!
//! Parses kernel boot parameters passed by the bootloader.
//! Supports Linux-style kernel parameters (key=value, flags).

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use spin::Mutex;

/// Maximum command line length
const MAX_CMDLINE_LEN: usize = 4096;

/// Parsed kernel command line parameters
static CMDLINE_PARAMS: Mutex<Option<BTreeMap<String, String>>> = Mutex::new(None);

/// Raw command line string
static RAW_CMDLINE: Mutex<Option<String>> = Mutex::new(None);

/// Initialize and parse kernel command line
///
/// # Arguments
///
/// * `cmdline` - Command line string from bootloader
///
/// # Examples
///
/// ```
/// init("root=/dev/sda1 ro quiet init=/sbin/init");
/// ```
pub fn init(cmdline: &str) {
    // Store raw command line
    let trimmed = cmdline.trim();
    if trimmed.len() > MAX_CMDLINE_LEN {
        *RAW_CMDLINE.lock() = Some(trimmed[..MAX_CMDLINE_LEN].to_string());
    } else {
        *RAW_CMDLINE.lock() = Some(trimmed.to_string());
    }

    // Parse parameters
    let params = parse_cmdline(trimmed);
    *CMDLINE_PARAMS.lock() = Some(params);
}

/// Parse command line string into key-value pairs
fn parse_cmdline(cmdline: &str) -> BTreeMap<String, String> {
    let mut params = BTreeMap::new();

    // Split by whitespace
    for token in cmdline.split_whitespace() {
        if token.is_empty() {
            continue;
        }

        // Check if it's a key=value pair
        if let Some(eq_pos) = token.find('=') {
            let key = &token[..eq_pos];
            let value = &token[eq_pos + 1..];
            params.insert(key.to_string(), value.to_string());
        } else {
            // It's a flag (no value), store with empty string
            params.insert(token.to_string(), String::new());
        }
    }

    params
}

/// Get a parameter value by key
///
/// # Arguments
///
/// * `key` - Parameter name
///
/// # Returns
///
/// Some(value) if parameter exists, None otherwise
pub fn get(key: &str) -> Option<String> {
    CMDLINE_PARAMS
        .lock()
        .as_ref()
        .and_then(|params| params.get(key).cloned())
}

/// Check if a flag is present
///
/// # Arguments
///
/// * `flag` - Flag name
///
/// # Returns
///
/// true if flag is present, false otherwise
pub fn has_flag(flag: &str) -> bool {
    CMDLINE_PARAMS
        .lock()
        .as_ref()
        .map(|params| params.contains_key(flag))
        .unwrap_or(false)
}

/// Get all parameters
pub fn all() -> Option<BTreeMap<String, String>> {
    CMDLINE_PARAMS.lock().clone()
}

/// Get raw command line string
pub fn raw() -> Option<String> {
    RAW_CMDLINE.lock().clone()
}

/// Check if command line was initialized
pub fn is_initialized() -> bool {
    CMDLINE_PARAMS.lock().is_some()
}

/// Common boot parameters that can be queried

/// Get root device parameter (e.g., "/dev/sda1")
pub fn root_device() -> Option<String> {
    get("root")
}

/// Check if root should be mounted read-only
pub fn is_readonly() -> bool {
    has_flag("ro")
}

/// Check if root should be mounted read-write
pub fn is_readwrite() -> bool {
    has_flag("rw")
}

/// Check if quiet mode is enabled (suppress boot messages)
pub fn is_quiet() -> bool {
    has_flag("quiet")
}

/// Check if verbose mode is enabled
pub fn is_verbose() -> bool {
    has_flag("verbose") || has_flag("debug")
}

/// Get init program path (default: /sbin/init)
pub fn init_program() -> String {
    get("init").unwrap_or_else(|| "/sbin/init".to_string())
}

/// Get console device
pub fn console() -> Option<String> {
    get("console")
}

/// Get memory limit in bytes
pub fn mem_limit() -> Option<u64> {
    get("mem").and_then(|s| parse_size(&s))
}

/// Parse size strings like "256M", "1G", "512K"
fn parse_size(s: &str) -> Option<u64> {
    if s.is_empty() {
        return None;
    }

    let last_char = s.chars().last()?;
    let (num_str, multiplier) = if last_char.is_alphabetic() {
        let num = &s[..s.len() - 1];
        let mult = match last_char.to_ascii_uppercase() {
            'K' => 1024,
            'M' => 1024 * 1024,
            'G' => 1024 * 1024 * 1024,
            _ => return None,
        };
        (num, mult)
    } else {
        (s, 1)
    };

    num_str.parse::<u64>().ok().map(|n| n * multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let params = parse_cmdline("root=/dev/sda1 ro");
        assert_eq!(params.get("root"), Some(&"/dev/sda1".to_string()));
        assert_eq!(params.get("ro"), Some(&String::new()));
    }

    #[test]
    fn test_parse_complex() {
        let params = parse_cmdline("root=/dev/sda1 init=/bin/sh mem=256M quiet");
        assert_eq!(params.get("root"), Some(&"/dev/sda1".to_string()));
        assert_eq!(params.get("init"), Some(&"/bin/sh".to_string()));
        assert_eq!(params.get("mem"), Some(&"256M".to_string()));
        assert!(params.contains_key("quiet"));
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("256M"), Some(256 * 1024 * 1024));
        assert_eq!(parse_size("1G"), Some(1024 * 1024 * 1024));
        assert_eq!(parse_size("512K"), Some(512 * 1024));
        assert_eq!(parse_size("100"), Some(100));
    }

    #[test]
    fn test_init_and_get() {
        init("root=/dev/sda1 ro quiet");
        assert_eq!(root_device(), Some("/dev/sda1".to_string()));
        assert!(is_readonly());
        assert!(is_quiet());
        assert!(!is_readwrite());
    }
}
