use std::{
    collections::{BTreeSet, HashMap},
    fmt::{self, Write as _},
};

use itertools::Itertools;

use crate::{Brex, Group, Numeric, Suffix, split_inclusive_start};

#[derive(thiserror::Error, Debug)]
pub enum EncodeError {
    #[error("Error formatting encoded string - {0}")]
    FmtError(#[from] std::fmt::Error),
}

type Result<T> = std::result::Result<T, EncodeError>;

impl fmt::Display for Numeric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Numeric::Single(v) => write!(f, "{v}"),
            Numeric::Range(start, end) => write!(f, "{start}..{end}"),
        }
    }
}
impl fmt::Display for Brex<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.preamble)?;

        f.write_char('<')?;

        let mut groups = self.groups.clone();
        //groups.sort_unstable_by(|a, b| a.prefix.cmp(b.prefix));

        for Group { prefix, suffixes } in groups {
            f.write_str(prefix)?;
            f.write_char('{')?;
            let mut suffixes = suffixes.clone();
            //suffixes.sort_unstable_by(|a, b| a.suffix.cmp(b.suffix));
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

        Ok(())
    }
}

pub fn encode<'a>(raw: &'a str) -> Result<Brex<'a>> {
    let line = raw.strip_suffix(".bin").unwrap_or(raw.as_ref());
    let mut parts = split_inclusive_start(line, '_');

    let mut a = Vec::<(&str, usize)>::new();
    let mut b = Vec::<(&str, usize)>::new();
    let mut parity = false;

    for part in parts.clone() {
        let set = match parity {
            false => &mut a,
            true => &mut b,
        };

        match set.last_mut() {
            Some(last) => {
                if last.0 == part {
                    last.1 += 1
                } else {
                    set.push((part, 1));
                }
            }
            None => {
                set.push((part, 1));
            }
        }

        parity = !parity;
    }
    //eprintln!("a: {a:?}");
    //eprintln!("b: {b:?}");

    // pick set with fewest prefixes
    let (offset, best_set) = match a.len() < b.len() {
        true => (0, a),
        false => (1, b),
    };

    //eprintln!("\nbest set: {best_set:?}");

    let best_set_skip = best_set
        .iter()
        .position(|(_, count)| *count > 1)
        .expect("at least one doubled prefix"); // TODO: properly handle no dupes

    let preamble = offset + (2 * best_set_skip);
    let preamble = (&mut parts)
        .take(preamble)
        .map(|part| part.len())
        .sum::<usize>();
    let preamble = &line[..preamble];

    //eprintln!("preamble: {preamble:?}");

    let groups = best_set
        .iter()
        .skip(best_set_skip)
        .map(|(prefix, size)| {
            let mut group = Vec::with_capacity(*size);
            for _ in 0..*size {
                assert_eq!(parts.next(), Some(*prefix));
                group.push(parts.next().expect("enough parts"));
            }
            (prefix, group)
        })
        .collect::<Vec<_>>();

    let groups = groups
        .into_iter()
        .map(|(prefix, suffixes)| {
            //eprintln!("\n{prefix}:\n  -     raw: {suffixes:?}");
            let suffixes = suffixes
                .iter()
                .map(
                    |suffix| match suffix.chars().position(|ch| ch.is_numeric()) {
                        Some(first_numeric) => {
                            let (left, right) = suffix.split_at(first_numeric);
                            let right = right.parse::<u32>().ok();
                            (left, right)
                        }
                        None => (*suffix, None),
                    },
                )
                .collect::<Vec<_>>();

            let mut non_numeric = Vec::new();
            let mut numerics: HashMap<&str, BTreeSet<u32>> = HashMap::with_capacity(suffixes.len());

            for (suffix, number) in suffixes {
                match number {
                    Some(number) => {
                        // we are assuming all the entries are sorted lexicographically
                        numerics.entry(suffix).or_default().insert(number);
                    }
                    None => non_numeric.push(suffix),
                }
            }

            let numerics = numerics
                .into_iter()
                .map(|(k, v)| {
                    let mut ranges = Vec::<Numeric>::new();
                    for v in v {
                        match ranges.last_mut() {
                            Some(last) if last.end() + 1 == v => *last = last.with_end(v).unwrap(),
                            _ => ranges.push(Numeric::new(v)),
                        }
                    }
                    (k, ranges)
                })
                .collect::<HashMap<_, _>>();

            //eprintln!("  -  normal: {non_numeric:?}");
            //eprintln!("  - numeric: {numerics:?}");
            Group {
                prefix,
                suffixes: non_numeric
                    .into_iter()
                    .map(Suffix::simple)
                    .chain(
                        numerics
                            .into_iter()
                            .map(|(suffix, numerics)| Suffix::numeric(suffix, numerics)),
                    )
                    .collect(),
            }
        })
        .collect_vec();

    Ok(Brex { preamble, groups })
}
//write!(
//    result,
//    "{prefix}{{{}",
//    non_numeric
//        .iter()
//        .map(|n| n.to_string())
//        .collect::<Vec<_>>()
//        .join(",")
//)?;
//if !numerics.is_empty() {
//    if !non_numeric.is_empty() {
//        write!(result, ",")?;
//    }

//    write!(
//        result,
//        "{}",
//        numerics
//            .iter()
//            .map(|(word, ranges)| {
//                let ranges = ranges
//                    .iter()
//                    .map(|(a, b)| format!("{a}..{b}"))
//                    .collect::<Vec<_>>()
//                    .join(",");
//                format!("{word}{{{ranges}}}")
//            })
//            .collect::<Vec<_>>()
//            .join(",")
//    )?;
//}
//write!(result, "}}")?;
