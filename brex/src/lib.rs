mod decode;
mod models;
mod util;

pub mod encode;
pub mod parse;

pub use models::*;

#[cfg(test)]
mod tests;

pub fn encode(input: &str) -> encode::Result<String> {
    Ok(Brex::encode(input)?.to_string())
}
pub fn decode(encoded: &str) -> parse::Result<String> {
    Ok(Brex::parse(encoded)?.expand())
}
