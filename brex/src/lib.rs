mod decode;
mod models;

mod util;

pub mod encode;
pub use encode::{Error as EncodeError, Result as EncodeResult};

pub mod parse;
pub use parse::{Error as ParseError, Result as ParseResult};

pub use models::*;

pub use decode::*;
pub use encode::*;

#[cfg(test)]
mod tests;

pub fn encode(input: &str) -> encode::Result<String> {
    Ok(Brex::encode(input)?.to_string())
}
pub fn decode(encoded: &str) -> parse::Result<String> {
    Ok(Brex::parse(encoded)?.unroll())
}
