//! Parsing brex strings
//!
//! See [`Brex::parse()`]

use nom::{
    Finish, Parser as _,
    bytes::complete::{is_not, tag, take_until, take_while1},
    combinator::opt,
    multi::{many1, separated_list1},
    sequence::{delimited, preceded},
};

use crate::{Brex, Group, Numeric, Suffix};

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
            tag("{"),
            separated_list1(
                tag(","),
                (
                    take_while1(|c: char| c.is_numeric()),
                    opt(preceded(tag(".."), take_while1(|c: char| c.is_numeric()))),
                ),
            ),
            tag("}"),
        )
        .map(|parts| {
            parts
                .into_iter()
                .map(|range: (&str, Option<&str>)| Numeric::try_from(range).unwrap())
                .collect::<Vec<_>>()
        });

        let group_suffixes = separated_list1(tag(","), (is_not("{,}"), opt(range))).map(|groups| {
            groups
                .into_iter()
                .map(|(suffix, numerics)| Suffix { suffix, numerics })
                .collect::<Vec<_>>()
        });

        let group_prefix = take_until("{");
        let group = (group_prefix, delimited(tag("{"), group_suffixes, tag("}")))
            .map(|(prefix, suffixes)| Group { prefix, suffixes });
        let groups = many1(group);

        let preamble = opt(is_not("<"));
        let (input, (preamble, groups)): (_, (_, Option<_>)) =
            (preamble, opt(delimited(tag("<"), groups, tag(">"))))
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
