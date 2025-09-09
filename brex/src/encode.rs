//! Encoding brex strings
//!
//! See [`Brex::encode()`] and [`crate::encode()`]

use std::collections::{BTreeSet, HashMap};

use itertools::Itertools;

use crate::{Brex, Group, Numeric, Suffix, util::split_inclusive_start};

#[derive(thiserror::Error, Debug)]
/// Error encoding a brex string
pub enum Error {
    #[error("Error formatting encoded string - {0}")]
    /// Error formatting the encoded string
    FmtError(#[from] std::fmt::Error),
}

impl<'a> Brex<'a> {
    /// Try to encode a string as brex. Not guaranteed to be the minimal possible representation.
    ///
    /// See [`crate::encode()`] for a convenience wrapper that encodes directly to a [`String`].
    pub fn encode(input: &'a str) -> Result<Self, Error> {
        let (line, postamble) = input
            .rsplit_once(".")
            .map(|(l, r)| (l, Some(&input[input.len() - (r.len() + 1)..])))
            .unwrap_or((input, None));
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

        let Some(best_set_skip) = best_set.iter().position(|(_, count)| *count > 1) else {
            // if there's no duplicates in the best set, there's no point encoding any groups
            return Ok(Brex::empty(input));
        };
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
                let mut numerics: HashMap<&str, BTreeSet<u32>> =
                    HashMap::with_capacity(suffixes.len());

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
                                Some(last) if last.end() + 1 == v => {
                                    *last = last.with_end(v).unwrap()
                                }
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

        Ok(Brex {
            preamble: match preamble.is_empty() {
                true => None,
                false => Some(preamble),
            },
            groups,
            postamble,
        })
    }
}
