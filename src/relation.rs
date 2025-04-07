use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
pub struct Relation {
    rel: BTreeMap<usize, BTreeSet<usize>>,
    inv: BTreeMap<usize, BTreeSet<usize>>,
}

impl Relation {
    pub fn add(&mut self, a: usize, b: usize) {
        self.rel.entry(a).or_default().insert(b);
        self.inv.entry(b).or_default().insert(a);
    }

    pub fn add_1_n(&mut self, a: usize, bs: &[usize]) {
        for &b in bs {
            self.add(a, b);
        }
    }

    pub fn add_n_1(&mut self, as_: &[usize], b: usize) {
        for &a in as_ {
            self.add(a, b);
        }
    }

    pub fn add_n_n(&mut self, as_: &[usize], bs: &[usize]) {
        for &a in as_ {
            for &b in bs {
                self.add(a, b);
            }
        }
    }

    pub fn add_str_str(&mut self, a: &str, b: &str) {
        self.add_n_n(&str_addrs(a), &str_addrs(b));
    }

    pub fn rel(&self, a: usize) -> Option<&BTreeSet<usize>> {
        self.rel.get(&a)
    }

    pub fn inv(&self, b: usize) -> Option<&BTreeSet<usize>> {
        self.inv.get(&b)
    }

    pub fn track(&mut self, f: impl Fn(&str) -> String, s: &str) -> String {
        let output = f(s);
        self.add_str_str(s, &output);
        output
    }
}

pub fn str_addrs(s: &str) -> Vec<usize> {
    let mut addrs = Vec::new();
    let mut addr = s.as_ptr() as usize;
    for c in s.chars() {
        addrs.push(addr);
        addr += c.len_utf8();
    }
    addrs
}
