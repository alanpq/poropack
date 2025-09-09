use crate::Numeric;

#[derive(Debug, Clone)]
/// The deduplicated suffix in a [`super::Group`]
pub struct Suffix<'a> {
    /// The suffix in question
    pub suffix: &'a str,
    /// [`Numeric`] suffixes of this suffix
    pub numerics: Option<Vec<Numeric>>,
}

impl<'a> Suffix<'a> {
    /// Create a plain suffix, without any numerics
    pub fn simple(suffix: &'a str) -> Self {
        Self {
            suffix,
            numerics: None,
        }
    }
    /// Creates a suffix with numerics
    pub fn numeric(suffix: &'a str, numerics: Vec<Numeric>) -> Self {
        Self {
            suffix,
            numerics: Some(numerics),
        }
    }
}
