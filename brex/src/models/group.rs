use crate::Suffix;

#[derive(Debug, Clone)]
/// A group of one prefix & multiple suffixes. Each suffix (and each numeric of each suffix) is appended to the prefix, to get the final expanded result.
pub struct Group<'a> {
    /// The group's prefix. Present before each suffix.
    pub prefix: &'a str,
    /// The group's suffixes. Preceded by [`Self::prefix`]
    pub suffixes: Vec<Suffix<'a>>,
}
