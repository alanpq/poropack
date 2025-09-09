use core::fmt;
use std::fmt::Write as _;

use crate::{Group, Suffix};

#[derive(Debug, Clone)]
/// IR of a brex string.
///
/// Created via [`Brex::encode()`] or [`Brex::parse()`].
/// To expand out to plaintext, see [`Brex::expand()`].
pub struct Brex<'a> {
    /// Plaintext before the `<`
    pub preamble: Option<&'a str>,
    /// The groups within the `<>` pair
    pub groups: Vec<Group<'a>>,
    /// Plaintext after the `>`
    pub postamble: Option<&'a str>,
}

impl<'a> Brex<'a> {
    pub(crate) fn empty(preamble: &'a str) -> Self {
        Self {
            preamble: Some(preamble),
            groups: vec![],
            postamble: None,
        }
    }
}

impl fmt::Display for Brex<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            preamble,
            groups,
            postamble,
        } = self;
        if let Some(preamble) = preamble {
            f.write_str(preamble)?;
        }

        if groups.is_empty() {
            return Ok(());
        }

        f.write_char('<')?;
        for Group { prefix, suffixes } in groups {
            f.write_str(prefix)?;
            f.write_char('{')?;
            for (i, Suffix { suffix, numerics }) in suffixes.iter().enumerate() {
                f.write_str(suffix)?;
                if let Some(numerics) = numerics {
                    f.write_char('{')?;
                    for (i, numeric) in numerics.iter().enumerate() {
                        numeric.fmt(f)?;
                        if i < numerics.len() - 1 {
                            f.write_char(',')?;
                        }
                    }
                    f.write_char('}')?;
                }
                if i < suffixes.len() - 1 {
                    f.write_char(',')?;
                }
            }
            f.write_char('}')?;
        }
        f.write_char('>')?;
        if let Some(postamble) = postamble {
            f.write_str(postamble)?;
        }

        Ok(())
    }
}
