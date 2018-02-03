//! Small utilities.

use serde_json::Value as JsonValue;
use std::io;
use std::net::TcpListener;

/// Find a TCP port number to use. This is racy but see
/// https://bugzilla.mozilla.org/show_bug.cgi?id=1240830
///
/// If port is Some, check if we can bind to the given port. Otherwise
/// pick a random port.
pub (crate) fn check_tcp_port(port: Option<u16>) -> io::Result<u16> {
    TcpListener::bind(&("localhost", port.unwrap_or(0)))
        .and_then(|stream| stream.local_addr())
        .map(|x| x.port())
}

/// Recursively merge serde_json::Value's from a then b into a new
/// returned value.
///
/// # Example
///
/// ```
/// # #[macro_use] extern crate serde_json;
/// # extern crate webdriver_client;
/// #
/// # use webdriver_client::util::merge_json;
/// # fn main() {
/// #
/// let a = json!({
///     "a": "only in a",
///     "overwritten": "value in a",
///     "child_object": { "x": "value in a" },
///     "array": ["value 1 in a", "value 2 in a"],
///     "different_types": 5
/// });
/// let b = json!({
///     "b": "only in b",
///     "overwritten": "value in b",
///     "child_object": { "x": "value in b" },
///     "array": ["value in b"],
///     "different_types": true
/// });
/// let merged = merge_json(&a, &b);
///
/// assert_eq!(merged, json!({
///     // When only one input contains the key, the value is cloned.
///     "a": "only in a",
///     "b": "only in b",
///
///     // When both inputs contain a key, the value from b is cloned.
///     "overwritten": "value in b",
///
///     // When a child object is present in both values, it is recursively
///     // merged.
///     "child_object": { "x": "value in b" },
///
///     // If both values are an array, the value from b is cloned.
///     "array": ["value in b"],
///
///     // If the two values have different types, the value from b is cloned.
///     "different_types": true
/// }));
/// #
/// # } // Close main.
pub fn merge_json(a: &JsonValue, b: &JsonValue) -> JsonValue {
    let mut out = a.clone();
    merge_json_mut(&mut out, b);
    out
}

/// Recursively merge serde_json::Value's from b into a.
pub fn merge_json_mut(a: &mut JsonValue, b: &JsonValue) {
    match (a, b) {
        // Recurse when a and b are both objects.
        (&mut JsonValue::Object(ref mut a), &JsonValue::Object(ref b)) => {
            for (k, v) in b {
                let a_entry = a.entry(k.clone()).or_insert(JsonValue::Null);
                merge_json_mut(a_entry, v);
            }
        }
        // When a and b aren't both objects, overwrite a.
        (a, b) => {
            *a = b.clone();
        }
    }
}
