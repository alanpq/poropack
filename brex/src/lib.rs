mod decode;
mod encode;

pub use decode::*;
pub use encode::*;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct SplitInclusiveStart<'a> {
    remainder: &'a str,
    delim: char,
}

impl<'a> SplitInclusiveStart<'a> {
    pub fn new(s: &'a str, delim: char) -> Self {
        Self {
            remainder: s,
            delim,
        }
    }
}

impl<'a> Iterator for SplitInclusiveStart<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remainder.is_empty() {
            return None;
        }

        //eprintln!("remainder: {}", self.remainder);
        if let Some(pos) = &self.remainder[self.delim.len_utf8()..].find(self.delim) {
            let pos = *pos + 1;
            //eprintln!("pos: {pos}");
            if pos == 0 {
                // remainder starts with delimiter
                let len = self.delim.len_utf8();
                let (piece, rest) = self.remainder.split_at(len);
                self.remainder = rest;
                Some(piece)
            } else {
                let (piece, rest) = self.remainder.split_at(pos);
                self.remainder = rest; // rest starts with delimiter
                Some(piece)
            }
        } else {
            // no more delimiters
            let piece = self.remainder;
            self.remainder = "";
            Some(piece)
        }
    }
}

pub fn split_inclusive_start<'a>(s: &'a str, delim: char) -> SplitInclusiveStart<'a> {
    SplitInclusiveStart::new(s, delim)
}
