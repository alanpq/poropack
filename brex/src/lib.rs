//! Crate for encoding & decoding brex strings, an encoding dedicated to representing repetitive League of Legends game file names succinctly.
#![deny(missing_docs)]
#![deny(warnings)]

mod decode;
mod models;
mod util;

pub mod encode;
pub mod parse;

pub use models::*;

#[cfg(test)]
mod tests;

/// Encode text to a brex string.
///
/// This is a convenience wrapper around [`Brex::encode`], stringifying the resulting [`Brex`]
pub fn encode(input: &str) -> Result<String, encode::Error> {
    Ok(Brex::encode(input)?.to_string())
}

/// Parse and expand a brex string.
///
/// This is a convenience wrapper around [`Brex::parse()`] and [`Brex::expand()`]
pub fn decode(encoded: &str) -> Result<String, parse::Error> {
    Ok(Brex::parse(encoded)?.expand())
}
