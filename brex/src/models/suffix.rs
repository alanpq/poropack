use crate::Numeric;

#[derive(Debug, Clone)]
pub struct Suffix<'a> {
    pub suffix: &'a str,
    pub numerics: Option<Vec<Numeric>>,
}

impl<'a> Suffix<'a> {
    pub fn simple(suffix: &'a str) -> Self {
        Self {
            suffix,
            numerics: None,
        }
    }
    pub fn numeric(suffix: &'a str, numerics: Vec<Numeric>) -> Self {
        Self {
            suffix,
            numerics: Some(numerics),
        }
    }
}
