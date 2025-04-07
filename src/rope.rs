//! A naive Rope implementation.

use regex::Regex;
use std::{cmp::min, fmt::Display, ops::Range};

/**************************************************************/

#[derive(Debug, Clone)]
pub struct Rope<'a> {
    pub data: Vec<&'a str>,
}

impl<'a> Rope<'a> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn append(&mut self, other: Rope<'a>) {
        self.data.extend(other.data);
    }

    pub fn slice(&self, range: Range<usize>) -> Rope<'a> {
        let start = range.start;
        let end = range.end;

        let mut data = vec![];
        let mut s_start;
        let mut s_end = 0;
        for slice in &self.data {
            s_start = s_end;
            s_end = s_start + slice.len();
            if s_end <= start {
                continue;
            }
            if s_start >= end {
                break;
            }
            let max_start = start.saturating_sub(s_start);
            let min_end = min(slice.len(), end - s_start);
            data.push(&slice[max_start..min_end]);
        }
        Rope { data }
    }

    pub fn len(&self) -> usize {
        self.data.iter().map(|s| s.len()).sum()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn re_range(&self, re: &Regex) -> Option<Range<usize>> {
        re.find(&self.to_string()).map(|mat| Range {
            start: mat.start(),
            end: mat.end(),
        })
    }

    pub fn re_ranges(&self, re: &Regex) -> Vec<Range<usize>> {
        re.find_iter(&self.to_string())
            .map(|mat| Range {
                start: mat.start(),
                end: mat.end(),
            })
            .collect()
    }

    pub fn re_slice(&self, re: &Regex) -> Option<Rope<'a>> {
        self.re_range(re).map(|range| self.slice(range))
    }

    pub fn re_slices(&self, re: &Regex) -> Vec<Rope<'a>> {
        self.re_ranges(re)
            .into_iter()
            .map(|range| self.slice(range))
            .collect()
    }

    pub fn re_replace(&self, re: &Regex, replacement: &Rope<'a>) -> Rope<'a> {
        if let Some(range) = self.re_range(re) {
            let mut new_rope = Rope::new();
            new_rope.append(self.slice(0..range.start));
            new_rope.append(replacement.clone());
            new_rope.append(self.slice(range.end..self.len()));
            new_rope
        } else {
            self.clone()
        }
    }

    pub fn re_replaces(&self, re: &Regex, replacement: &Rope<'a>) -> Rope<'a> {
        let mut new_rope = Rope::new();
        let mut ind = 0;
        for range in self.re_ranges(re) {
            new_rope.append(self.slice(ind..range.start));
            new_rope.append(replacement.clone());
            ind = range.end;
        }
        if ind < self.len() {
            new_rope.append(self.slice(ind..self.len()));
        }
        new_rope
    }

    pub fn indent(&self, indent: &Rope<'a>) -> Rope<'a> {
        let mut ret = indent.clone();
        let re_newline = Regex::new(r#"\n"#).unwrap();
        let mut replacement: Rope<'_> = "\n".into();
        replacement.append(indent.clone());
        ret.append(self.re_replaces(&re_newline, &replacement));
        ret
    }

    pub fn addrs(&self) -> Vec<usize> {
        let mut addresses = vec![];
        for data in &self.data {
            let mut addr = data.as_ptr() as usize;
            for c in data.chars() {
                addresses.push(addr);
                addr += c.len_utf8();
            }
        }
        addresses
    }
}

impl Default for Rope<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<&'a str> for Rope<'a> {
    fn from(s: &'a str) -> Self {
        Self { data: vec![s] }
    }
}

impl Display for Rope<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for slice in &self.data {
            write!(f, "{slice}")?;
        }
        Ok(())
    }
}
