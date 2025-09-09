//! Parsing brex strings
//!
//! See [`Brex::parse()`]

use nom::{
    Finish, Parser as _,
    bytes::complete::{is_not, take_till, take_while1},
    character::complete::char,
    combinator::opt,
    multi::{many1, separated_list1},
    sequence::{delimited, preceded},
};

use crate::{
    Brex, Group, Numeric, Suffix,
    alphabet::{self, BREX_BLOCK, GROUP_BLOCK, GROUP_SUFFIX_SEP, NUMERIC_BLOCK, NUMERIC_LIST_SEP},
};

/// Error parsing a brex string
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    /// Error parsing a numeric suffix
    ParseNumberError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    /// Underlying parser error
    NomError(#[from] nom::error::Error<String>),
}

impl<'a> Brex<'a> {
    /// Parse a brex string.
    pub fn parse(input: &'a str) -> Result<Self, Error> {
        let range = delimited(
            char(NUMERIC_BLOCK.start),
            separated_list1(
                char(NUMERIC_LIST_SEP),
                (
                    take_while1(|c: char| c.is_numeric()),
                    opt(preceded(
                        char(alphabet::NUMERIC_RANGE_DELIM),
                        take_while1(|c: char| c.is_numeric()),
                    )),
                ),
            ),
            char(NUMERIC_BLOCK.end),
        )
        .map(|parts| {
            parts
                .into_iter()
                .map(|range: (&str, Option<&str>)| Numeric::try_from(range).unwrap())
                .collect::<Vec<_>>()
        });

        let group_suffixes = separated_list1(
            char(GROUP_SUFFIX_SEP),
            (
                is_not(&[GROUP_SUFFIX_SEP, NUMERIC_BLOCK.start, NUMERIC_BLOCK.end][..]),
                opt(range),
            ),
        )
        .map(|groups| {
            groups
                .into_iter()
                .map(|(suffix, numerics)| Suffix { suffix, numerics })
                .collect::<Vec<_>>()
        });

        let group_prefix = take_till(|c| c == GROUP_BLOCK.start);
        let group = (
            group_prefix,
            delimited(
                char(GROUP_BLOCK.start),
                group_suffixes,
                char(GROUP_BLOCK.end),
            ),
        )
            .map(|(prefix, suffixes)| Group { prefix, suffixes });
        let groups = many1(group);

        let preamble = opt(take_till(|c| c == BREX_BLOCK.start));
        let (input, (preamble, groups)): (_, (_, Option<_>)) = (
            preamble,
            opt(delimited(
                char(alphabet::BREX_BLOCK.start),
                groups,
                char(alphabet::BREX_BLOCK.end),
            )),
        )
            .parse(input)
            .finish()
            .map_err(|err: nom::error::Error<&str>| err.cloned())?;

        // eprintln!("{input:?}");
        // eprintln!("{preamble:?}");
        // eprintln!("{groups:?}");

        Ok(Brex {
            preamble,
            postamble: match input.is_empty() {
                true => None,
                false => Some(input),
            },
            groups: groups.unwrap_or_default(),
        })
    }
}
