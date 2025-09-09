use crate::Suffix;

#[derive(Debug, Clone)]
pub struct Group<'a> {
    pub prefix: &'a str,
    pub suffixes: Vec<Suffix<'a>>,
}
