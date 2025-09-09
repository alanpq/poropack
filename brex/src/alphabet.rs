//! Grammar alphabet constants

/// A pair of two chars, that define a parseable block
pub struct Pair {
    /// Block start symbol
    pub start: char,
    /// Block end symbol
    pub end: char,
}
impl Pair {
    /// Creates a new symbol pair
    pub const fn new(start: char, end: char) -> Self {
        Self { start, end }
    }
}

/// Starts/ends a 'brex block' (`[preamble]<brex block>[postamble]`)
pub const BREX_BLOCK: Pair = Pair::new('❮', '❯');
/// Starts/ends a group block
pub const GROUP_BLOCK: Pair = Pair::new('{', '}');
/// Starts/ends a numeric block
pub const NUMERIC_BLOCK: Pair = Pair::new('{', '}');

/// A numeric range delimiter
pub const NUMERIC_RANGE_DELIM: char = '→';
/// Numeric list separator
pub const NUMERIC_LIST_SEP: char = ',';

/// Group suffix list separator
pub const GROUP_SUFFIX_SEP: char = ',';
