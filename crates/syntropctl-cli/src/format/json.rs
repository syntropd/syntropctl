//! JSON output serialization for script consumption and piping.

use serde::Serialize;

/// Serialize and print data structure as formatted JSON.
pub fn print_json<T: Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("{{\"error\": \"failed to serialize json: {}\"}}", e),
    }
}
