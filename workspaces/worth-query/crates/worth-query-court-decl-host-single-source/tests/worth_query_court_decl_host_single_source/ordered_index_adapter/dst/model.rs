//! Brute-force sorted-scan oracle and title/prefix bijection for ordered-index DST.
use std::collections::BTreeMap;

pub const MAX_ROWS: u64 = 8;
pub const MAX_COORD: i64 = 16;

/// Prefix-disjoint title families matching `TitleGroups = 0..4` in `OrderedIndexAdapterTrace.tla`.
pub const TITLES: &[&str] = &[
    "Alpha",
    "alpine",
    "ALDER",
    "Beta",
    "beta-plan",
    "delta",
    "Delta-Review",
    "Focus",
    "focus-deep",
    "élan",
    "élan-plan",
];

pub const PREFIXES: &[&str] = &["", "al", "AL", "be", "del", "foc", "é", "z"];

pub fn title_group(title: &str) -> i64 {
    let lower = title.to_ascii_lowercase();
    if lower.starts_with("al") {
        0
    } else if lower.starts_with("be") {
        1
    } else if lower.starts_with("del") {
        2
    } else if lower.starts_with("foc") {
        3
    } else if lower.starts_with('é') {
        4
    } else {
        panic!("title {title:?} outside prefix-disjoint DST title groups");
    }
}

pub fn title_probe_id(prefix: &str) -> i64 {
    match prefix.to_ascii_lowercase().as_str() {
        "" => -1,
        "al" => 0,
        "be" => 1,
        "del" => 2,
        "foc" => 3,
        "é" => 4,
        "z" => 99,
        other => panic!("prefix {other:?} outside DST prefix vocabulary"),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleTask {
    pub title: String,
    pub lo: i64,
    pub hi: i64,
}

impl OracleTask {
    pub fn matches_prefix(&self, prefix: &str) -> bool {
        let needle = prefix.to_ascii_lowercase();
        self.title.to_ascii_lowercase().starts_with(&needle)
    }

    pub fn matches_focus(&self, lo: i64, hi: i64) -> bool {
        self.lo <= self.hi && lo <= hi && self.lo <= hi && lo <= self.hi
    }
}

#[derive(Default)]
pub struct SortedScanOracle {
    pub rows: BTreeMap<u64, OracleTask>,
    pub watch_active: bool,
    pub title_watch_probe: Option<String>,
    pub focus_watch_probe: Option<(i64, i64)>,
    pub title_watch_pending: bool,
    pub focus_watch_pending: bool,
}

impl SortedScanOracle {
    pub fn prefix_matches(&self, prefix: &str) -> Vec<u64> {
        self.rows
            .iter()
            .filter(|(_, t)| t.matches_prefix(prefix))
            .map(|(&k, _)| k)
            .collect()
    }

    pub fn focus_matches(&self, lo: i64, hi: i64) -> Vec<u64> {
        self.rows
            .iter()
            .filter(|(_, t)| t.matches_focus(lo, hi))
            .map(|(&k, _)| k)
            .collect()
    }

    pub fn on_open_watches(&mut self, prefix: &str, lo: i64, hi: i64) {
        self.watch_active = true;
        self.title_watch_probe = Some(prefix.to_owned());
        self.focus_watch_probe = Some((lo, hi));
        self.title_watch_pending = false;
        self.focus_watch_pending = false;
    }

    pub fn on_close_watches(&mut self) {
        self.watch_active = false;
        self.title_watch_probe = None;
        self.focus_watch_probe = None;
        self.title_watch_pending = false;
        self.focus_watch_pending = false;
    }

    pub fn on_mutation(&mut self, before: Option<&OracleTask>, after: Option<&OracleTask>) {
        if !self.watch_active {
            return;
        }
        if let Some(prefix) = self.title_watch_probe.as_deref()
            && (before.is_some_and(|t| t.matches_prefix(prefix))
                || after.is_some_and(|t| t.matches_prefix(prefix)))
        {
            self.title_watch_pending = true;
        }
        if let Some((lo, hi)) = self.focus_watch_probe
            && (before.is_some_and(|t| t.matches_focus(lo, hi))
                || after.is_some_and(|t| t.matches_focus(lo, hi)))
        {
            self.focus_watch_pending = true;
        }
    }
}
