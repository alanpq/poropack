use std::{
    collections::{BTreeSet, HashMap},
    fmt::Write as _,
};

#[derive(thiserror::Error, Debug)]
pub enum EncodeError {
    #[error("Error formatting encoded string - {0}")]
    FmtError(#[from] std::fmt::Error),
}

type Result<T> = std::result::Result<T, EncodeError>;

pub fn encode(raw: impl AsRef<str>) -> Result<String> {
    let line = raw.as_ref().strip_suffix(".bin").unwrap_or(raw.as_ref());
    let mut parts = line.split_terminator('_');

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
    let preamble = (&mut parts).take(preamble).collect::<Vec<_>>();

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

    let mut result = preamble.join("_") + "<";
    for (prefix, suffixes) in groups {
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
                let mut ranges = Vec::<(u32, u32)>::new();
                for v in v {
                    match ranges.last_mut() {
                        Some(last) if last.1 + 1 == v => last.1 = v,
                        _ => ranges.push((v, v)),
                    }
                }
                (k, ranges)
            })
            .collect::<HashMap<_, _>>();

        //eprintln!("  -  normal: {non_numeric:?}");
        //eprintln!("  - numeric: {numerics:?}");

        write!(
            result,
            "_{prefix}{{{}",
            non_numeric
                .iter()
                .map(|n| format!("_{n}"))
                .collect::<Vec<_>>()
                .join(",")
        )?;
        if !numerics.is_empty() {
            if !non_numeric.is_empty() {
                write!(result, ",")?;
            }

            write!(
                result,
                "{}",
                numerics
                    .iter()
                    .map(|(word, ranges)| {
                        let ranges = ranges
                            .iter()
                            .map(|(a, b)| format!("{a}..{b}"))
                            .collect::<Vec<_>>()
                            .join(",");
                        format!("_{word}{{{ranges}}}")
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            )?;
        }
        write!(result, "}}")?;
    }
    write!(result, ">")?;

    Ok(result)
}
