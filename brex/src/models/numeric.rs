use core::fmt;

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

impl fmt::Display for Numeric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Numeric::Single(v) => write!(f, "{v}"),
            Numeric::Range(start, end) => write!(f, "{start}..{end}"),
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
