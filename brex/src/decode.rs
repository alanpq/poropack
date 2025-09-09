use nom::{
    Finish, IResult, Parser as _,
    bytes::complete::{is_not, tag, take_until, take_while1},
    character::char,
    combinator::opt,
    multi::{many1, separated_list1},
    sequence::{delimited, preceded, terminated},
};

#[derive(Debug, Clone)]
pub struct Brex<'a> {
    pub preamble: &'a str,
    pub groups: Vec<Group<'a>>,
}

impl<'a> Brex<'a> {
    fn unroll(&self) -> String {
        let mut groups = self
            .groups
            .iter()
            .flat_map(|group| group.unroll())
            .collect::<Vec<_>>();
        groups.sort_unstable();
        format!("{}{}", self.preamble, groups.join(""))
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
    pub fn unroll(&self) -> Vec<String> {
        match &self.numerics {
            Some(numerics) => numerics
                .iter()
                .flat_map(|numeric| numeric.start()..=numeric.end())
                .map(|num| format!("{}{num}", self.suffix))
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
}

impl TryFrom<(&str, Option<&str>)> for Numeric {
    type Error = std::num::ParseIntError;

    fn try_from(value: (&str, Option<&str>)) -> Result<Self, Self::Error> {
        match value {
            (start, Some(end)) => Ok(Self::Range(start.parse()?, end.parse()?)),
            (start, None) => Ok(Self::Single(start.parse()?)),
        }
    }
}

impl<'a> Brex<'a> {
    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let input = input.strip_suffix(".bin").unwrap_or(input);
        let preamble = terminated(take_until("<"), char('<'));

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

        let (input, (preamble, groups)) = (preamble, groups).parse(input)?;

        Ok((input, Brex { preamble, groups }))
    }
}
