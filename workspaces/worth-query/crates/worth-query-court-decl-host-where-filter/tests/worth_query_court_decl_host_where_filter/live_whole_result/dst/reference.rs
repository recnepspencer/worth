//! Reference relational & whole-result footprint oracle mirroring
//! `specs/tla/live_subscriptions/LiveSubscriptions.tla`.
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Today,
    Ready,
}

impl Kind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Today => "Today",
            Self::Ready => "Ready",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    pub status: BTreeMap<u64, &'static str>,
    /// 0 = None, 1 = Today (<= cutoff), 2 = Future (> cutoff)
    pub due: BTreeMap<u64, u8>,
    pub edges: BTreeSet<(u64, u64)>,
    pub status_rev: BTreeMap<u64, u64>,
    pub due_rev: BTreeMap<u64, u64>,
    pub adj_rev: BTreeMap<u64, u64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Footprint {
    pub status_rev: BTreeMap<u64, u64>,
    pub due_rev: BTreeMap<u64, u64>,
    pub adj_rev: BTreeMap<u64, u64>,
}

#[derive(Clone, Debug)]
pub struct SubOracle {
    pub active: bool,
    pub cursor: usize,
    pub last_rows: Vec<u64>,
    pub fp: Footprint,
}

impl Default for SubOracle {
    fn default() -> Self {
        Self {
            active: false,
            cursor: 1,
            last_rows: Vec::new(),
            fp: Footprint::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BoardOracle {
    pub project: BTreeMap<u64, &'static str>,
    pub history: Vec<Snapshot>,
    pub grant: bool,
    pub today: SubOracle,
    pub ready: SubOracle,
}

impl BoardOracle {
    pub fn new() -> Self {
        let mut project = BTreeMap::new();
        let mut status = BTreeMap::new();
        let mut due = BTreeMap::new();
        let mut status_rev = BTreeMap::new();
        let mut due_rev = BTreeMap::new();
        let mut adj_rev = BTreeMap::new();
        for key in 1..=6_u64 {
            project.insert(key, if key <= 4 { "p1" } else { "p2" });
            status.insert(key, "open");
            due.insert(key, 2);
            status_rev.insert(key, 0);
            due_rev.insert(key, 0);
            adj_rev.insert(key, 0);
        }
        let init = Snapshot {
            status,
            due,
            edges: BTreeSet::new(),
            status_rev,
            due_rev,
            adj_rev,
        };
        Self {
            project,
            history: vec![init],
            grant: true,
            today: SubOracle::default(),
            ready: SubOracle::default(),
        }
    }

    pub fn current(&self) -> &Snapshot {
        self.history.last().expect("non-empty history")
    }

    pub fn toggle_status(&mut self, task: u64) -> &'static str {
        let mut next = self.current().clone();
        let val = if next.status[&task] == "open" {
            "done"
        } else {
            "open"
        };
        next.status.insert(task, val);
        *next.status_rev.get_mut(&task).unwrap() += 1;
        self.history.push(next);
        val
    }

    pub fn set_due(&mut self, task: u64, bucket: u8) -> bool {
        if self.current().due[&task] == bucket {
            return false;
        }
        let mut next = self.current().clone();
        next.due.insert(task, bucket);
        *next.due_rev.get_mut(&task).unwrap() += 1;
        self.history.push(next);
        true
    }

    pub fn add_edge(&mut self, from: u64, to: u64) -> bool {
        if self.current().edges.contains(&(from, to)) {
            return false;
        }
        let mut next = self.current().clone();
        next.edges.insert((from, to));
        *next.adj_rev.get_mut(&from).unwrap() += 1;
        self.history.push(next);
        true
    }

    pub fn remove_edge(&mut self, from: u64, to: u64) -> bool {
        if !self.current().edges.contains(&(from, to)) {
            return false;
        }
        let mut next = self.current().clone();
        next.edges.remove(&(from, to));
        *next.adj_rev.get_mut(&from).unwrap() += 1;
        self.history.push(next);
        true
    }

    pub fn eval(&self, kind: Kind, snap: &Snapshot) -> Vec<u64> {
        let mut out = Vec::new();
        for (&t, &proj) in &self.project {
            if proj != "p1" || snap.status[&t] != "open" {
                continue;
            }
            match kind {
                Kind::Today => {
                    if snap.due[&t] == 1 {
                        out.push(t);
                    }
                }
                Kind::Ready => {
                    let blocked = snap
                        .edges
                        .range((t, 0)..=(t, u64::MAX))
                        .any(|&(_, b)| snap.status[&b] == "open");
                    if !blocked {
                        out.push(t);
                    }
                }
            }
        }
        out
    }

    pub fn capture_footprint(&self, kind: Kind, snap: &Snapshot) -> Footprint {
        let mut fp = Footprint::default();
        for (&t, &proj) in &self.project {
            if proj != "p1" {
                continue;
            }
            fp.status_rev.insert(t, snap.status_rev[&t]);
            if snap.status[&t] != "open" {
                continue;
            }
            match kind {
                Kind::Today => {
                    fp.due_rev.insert(t, snap.due_rev[&t]);
                }
                Kind::Ready => {
                    fp.adj_rev.insert(t, snap.adj_rev[&t]);
                    for &(_, b) in snap.edges.range((t, 0)..=(t, u64::MAX)) {
                        fp.status_rev.insert(b, snap.status_rev[&b]);
                        if snap.status[&b] == "open" {
                            break;
                        }
                    }
                }
            }
        }
        fp
    }

    fn footprint_current(fp: &Footprint, snap: &Snapshot) -> bool {
        fp.status_rev.iter().all(|(t, r)| snap.status_rev[t] == *r)
            && fp.due_rev.iter().all(|(t, r)| snap.due_rev[t] == *r)
            && fp.adj_rev.iter().all(|(t, r)| snap.adj_rev[t] == *r)
    }

    pub fn sub_mut(&mut self, kind: Kind) -> &mut SubOracle {
        match kind {
            Kind::Today => &mut self.today,
            Kind::Ready => &mut self.ready,
        }
    }

    pub fn expected_subscribe(&mut self, kind: Kind) -> (&'static str, Vec<u64>) {
        if !self.grant {
            return ("AuthDenied", Vec::new());
        }
        let idx = self.history.len();
        let snap = self.current().clone();
        let rows = self.eval(kind, &snap);
        let fp = self.capture_footprint(kind, &snap);
        let sub = self.sub_mut(kind);
        sub.active = true;
        sub.cursor = idx + 1;
        sub.last_rows = rows.clone();
        sub.fp = fp;
        ("Initial", rows)
    }

    pub fn expected_poll(&mut self, kind: Kind) -> (&'static str, Vec<u64>) {
        if !self.grant {
            let sub = self.sub_mut(kind);
            sub.active = false;
            return ("AuthDenied", Vec::new());
        }
        let (cursor, fp) = {
            let sub = self.sub_mut(kind);
            (sub.cursor, sub.fp.clone())
        };
        let mut hit = None;
        for idx in cursor..=self.history.len() {
            if !Self::footprint_current(&fp, &self.history[idx - 1]) {
                hit = Some(idx);
                break;
            }
        }
        if let Some(idx) = hit {
            let snap = self.history[idx - 1].clone();
            let rows = self.eval(kind, &snap);
            let next_fp = self.capture_footprint(kind, &snap);
            let sub = self.sub_mut(kind);
            sub.cursor = idx + 1;
            sub.last_rows = rows.clone();
            sub.fp = next_fp;
            ("Delivered", rows)
        } else {
            let next_cursor = self.history.len() + 1;
            let sub = self.sub_mut(kind);
            sub.cursor = next_cursor;
            ("Pending", Vec::new())
        }
    }

    pub fn close_sub(&mut self, kind: Kind) {
        *self.sub_mut(kind) = SubOracle::default();
    }

    pub fn reopen_home(&mut self) {
        self.today = SubOracle::default();
        self.ready = SubOracle::default();
    }
}
