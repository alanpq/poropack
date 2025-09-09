use core::fmt;

/// A number/range of numbers
#[derive(Debug, Clone, Copy)]
pub enum Numeric {
    /// A single number
    Single(u32),
    /// A range of numbers (inclusive)
    Range(u32, u32),
}

impl Numeric {
    /// Create a single number
    pub fn new(v: u32) -> Self {
        Self::Single(v)
    }

    /// The start of this numeric. [`Numeric::Single`]'s return their same single value for [`Self::start()`] and [`Self::end()`]
    pub fn start(&self) -> u32 {
        match self {
            Self::Single(start) | Self::Range(start, _) => *start,
        }
    }

    /// The end of this numeric. [`Numeric::Single`]'s return their same single value for [`Self::start()`] and [`Self::end()`]
    pub fn end(&self) -> u32 {
        match self {
            Self::Single(end) | Self::Range(_, end) => *end,
        }
    }

    /// Creates a new numeric with the given end. Converts [`Numeric::Single`]'s to [`Numeric::Range`]'s
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
