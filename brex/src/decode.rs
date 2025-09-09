use itertools::Itertools;

use crate::{Brex, Group, Suffix, util::inverted_substr_sort};

impl<'a> Brex<'a> {
    pub fn expand(&self) -> String {
        let groups = self
            .groups
            .iter()
            .sorted_unstable_by(|a, b| inverted_substr_sort(a.prefix, b.prefix))
            .flat_map(|group| group.expand())
            .collect::<Vec<_>>();
        match (self.preamble, self.postamble) {
            (Some(pre), Some(post)) => format!("{pre}{}{post}", groups.join("")),
            (Some(pre), None) => format!("{pre}{}", groups.join("")),
            (None, Some(post)) => format!("{}{post}", groups.join("")),
            (None, None) => groups.join(""),
        }
    }
}
impl<'a> Group<'a> {
    pub fn expand(&self) -> impl Iterator<Item = String> {
        self.suffixes
            .iter()
            //.sorted_by(|a, b| new_sort(a.suffix, b.suffix))
            .flat_map(|suffix| suffix.expand())
            .map(|suffix| format!("{}{suffix}", self.prefix))
    }
}

impl<'a> Suffix<'a> {
    pub fn expand(&self) -> Vec<String> {
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
