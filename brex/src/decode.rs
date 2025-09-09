use itertools::Itertools;
use nom::{
    Finish, IResult, Parser as _,
    bytes::complete::{is_not, tag, take_until, take_while1},
    combinator::opt,
    multi::{many1, separated_list1},
    sequence::{delimited, preceded, terminated},
};

#[derive(Debug, Clone)]
pub struct Brex<'a> {
    pub preamble: Option<&'a str>,
    pub groups: Vec<Group<'a>>,
    pub postamble: Option<&'a str>,
}

/// Sort function that has substrings > their superstring's
/// (this is the opposite of normal str::cmp behaviour)
///
/// e.g "superfan" > "superfanvariant"
fn inverted_substr_sort(a: &str, b: &str) -> std::cmp::Ordering {
    let len = a.len().min(b.len());
    match &a[..len].cmp(&b[..len]) {
        std::cmp::Ordering::Equal => b.len().cmp(&a.len()),
        order => *order,
    }
}
impl<'a> Brex<'a> {
    pub fn empty(preamble: &'a str) -> Self {
        Self {
            preamble: Some(preamble),
            groups: vec![],
            postamble: None,
        }
    }
    pub fn unroll(&self) -> String {
        let groups = self
            .groups
            .iter()
            .sorted_unstable_by(|a, b| inverted_substr_sort(a.prefix, b.prefix))
            .flat_map(|group| group.unroll())
            .collect::<Vec<_>>();
        match (self.preamble, self.postamble) {
            (Some(pre), Some(post)) => format!("{pre}{}{post}", groups.join("")),
            (Some(pre), None) => format!("{pre}{}", groups.join("")),
            (None, Some(post)) => format!("{}{post}", groups.join("")),
            (None, None) => groups.join(""),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Group<'a> {
    pub prefix: &'a str,
    pub suffixes: Vec<Suffix<'a>>,
}

impl<'a> Group<'a> {
    pub fn unroll(&self) -> impl Iterator<Item = String> {
        self.suffixes
            .iter()
            //.sorted_by(|a, b| new_sort(a.suffix, b.suffix))
            .flat_map(|suffix| suffix.unroll())
            .map(|suffix| format!("{}{suffix}", self.prefix))
    }
}

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

    pub fn unroll(&self) -> Vec<String> {
        match &self.numerics {
            Some(numerics) => numerics
                .iter()
                .flat_map(|numeric| numeric.start()..=numeric.end())
                .map(|num| format!("{}{num}", self.suffix))
                .sorted()
                .collect(),
            None => vec![self.suffix.to_string()],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Numeric {
    Single(u32),
    Range(u32, u32),
}

impl Numeric {
    pub fn new(v: u32) -> Self {
        Self::Single(v)
    }
    pub fn start(&self) -> u32 {
        match self {
            Self::Single(start) | Self::Range(start, _) => *start,
        }
    }
    pub fn end(&self) -> u32 {
        match self {
            Self::Single(end) | Self::Range(_, end) => *end,
        }
    }

    pub fn with_end(self, end: u32) -> Option<Self> {
        match self {
            Numeric::Single(start) | Numeric::Range(start, _) if end >= start => {
                Some(Self::Range(start, end))
            }
            _ => None,
        }
    }
}

impl TryFrom<(&str, Option<&str>)> for Numeric {
    type Error = std::num::ParseIntError;

    fn try_from(value: (&str, Option<&str>)) -> std::result::Result<Self, Self::Error> {
        match value {
            (start, Some(end)) => Ok(Self::Range(start.parse()?, end.parse()?)),
            (start, None) => Ok(Self::Single(start.parse()?)),
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error(transparent)]
    ParseNumberError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    NomError(#[from] nom::error::Error<String>),
}

type Result<T> = std::result::Result<T, nom::error::Error<String>>;

impl<'a> Brex<'a> {
    pub fn parse(input: &'a str) -> Result<Self> {
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
                .map_err(|err: nom::error::Error<&str>| err.to_owned())?;

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

pub fn decode(encoded: &str) -> Result<String> {
    Ok(Brex::parse(encoded)?.unroll())
}
