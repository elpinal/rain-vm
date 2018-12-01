//! Rain VM has two kinds of versions.
//! The first is the byte version.
//! The second is the dominant version.

use std::collections::HashMap;

/// The current byte version.
pub const BYTE_VERSION: u8 = 1;

/// Returns a map from byte versions to dominant versions.
pub fn version_map() -> HashMap<u8, String> {
    let mut m = HashMap::new();
    m.insert(1, "0.1.0".to_string());
    m
}
