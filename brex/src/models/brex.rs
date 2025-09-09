use core::fmt;
use std::fmt::Write as _;

use crate::{Group, Suffix};

#[derive(Debug, Clone)]
pub struct Brex<'a> {
    pub preamble: Option<&'a str>,
    pub groups: Vec<Group<'a>>,
    pub postamble: Option<&'a str>,
}

impl<'a> Brex<'a> {
    pub fn empty(preamble: &'a str) -> Self {
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
