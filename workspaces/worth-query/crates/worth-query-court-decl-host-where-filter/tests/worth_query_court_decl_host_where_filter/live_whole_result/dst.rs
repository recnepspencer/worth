//! Single-harness 64-seed Differential Live-Subscription DST + Reference Oracle
//! verifying whole-result `Today` (`where_filter`) & `Ready` (`where_no_related`)
//! live subscriptions, real capability grant revoke/restore, unrelated-project
//! non-invalidation, and real Directory Home close/reopen + resubscribe.
//!
//! Environment knobs:
//! - `WORTH_DST_SEEDS` (default 64)
//! - `WORTH_DST_SEED` (run one seed)
//! - `WORTH_DST_TRACE_DIR` (emit NDJSON traces for `tools/tla/run_tlc.py trace`)
mod observe;
mod reference;
mod step;
mod trace;

use super::world::{
    close_directory_home, install, open_directory_home, Home, Host, TaskSeed, FUTURE_DUE_DAYS,
};
use reference::BoardOracle;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use step::run_epoch;
use trace::Trace;

const STEPS_PER_SEED: usize = 12;

pub(super) struct Rng(u64);
impl Rng {
    pub(super) fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(0x9e37_79b9_7f4a_7c15))
    }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    pub(super) fn pick(&mut self, bound: usize) -> usize {
        (self.next() as usize) % bound
    }
}

#[derive(Default, Debug)]
pub(super) struct Coverage {
    pub toggles: usize,
    pub due_edits: usize,
    pub edge_adds: usize,
    pub edge_removes: usize,
    pub p2_edits: usize,
    pub grant_revokes: usize,
    pub grant_restores: usize,
    pub reopens: usize,
    pub deliveries: usize,
    pub pendings: usize,
    pub auth_denials: usize,
}
impl Coverage {
    fn add(&mut self, o: Self) {
        self.toggles += o.toggles;
        self.due_edits += o.due_edits;
        self.edge_adds += o.edge_adds;
        self.edge_removes += o.edge_removes;
        self.p2_edits += o.p2_edits;
        self.grant_revokes += o.grant_revokes;
        self.grant_restores += o.grant_restores;
        self.reopens += o.reopens;
        self.deliveries += o.deliveries;
        self.pendings += o.pendings;
        self.auth_denials += o.auth_denials;
    }
}

enum RuntimeBacking {
    Memory(Host),
    Directory {
        home: Home,
        dir: PathBuf,
        seeds: Vec<TaskSeed>,
    },
}

impl RuntimeBacking {
    fn host(&self) -> &Host {
        match self {
            Self::Memory(host) => host,
            Self::Directory { home, .. } => home.runtime(),
        }
    }
    fn reopen(self) -> Self {
        match self {
            Self::Directory { home, dir, seeds } => {
                close_directory_home(home);
                let reopened = open_directory_home(dir.clone(), &seeds, &[]);
                Self::Directory {
                    home: reopened,
                    dir,
                    seeds,
                }
            }
            Self::Memory(host) => Self::Memory(host),
        }
    }
    fn cleanup(self) {
        if let Self::Directory { home, dir, .. } = self {
            close_directory_home(home);
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
}

fn initial_seeds() -> Vec<TaskSeed> {
    (1..=6_u64)
        .map(|key| TaskSeed {
            key,
            project: if key <= 4 { "p1" } else { "p2" }.to_owned(),
            title: format!("Task {key}"),
            notes: Some(format!("note-{key}")),
            status: "open".to_owned(),
            due_days: Some(FUTURE_DUE_DAYS),
        })
        .collect()
}

fn run_seed(seed: u64, trace_dir: Option<&std::path::Path>) -> Result<Coverage, String> {
    let mut rng = Rng::new(seed);
    let seeds = initial_seeds();
    let use_dir = trace_dir.is_some() || seed.is_multiple_of(16);
    let mut backing = if use_dir {
        let dir = std::env::temp_dir().join(format!("worth-p2c-dst-{}-{seed}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let home = open_directory_home(dir.clone(), &seeds, &[]);
        RuntimeBacking::Directory { home, dir, seeds }
    } else {
        RuntimeBacking::Memory(install(&seeds, &[]))
    };

    let mut oracle = BoardOracle::new();
    let mut version_map = std::collections::BTreeMap::new();
    let mut trace = Trace::default();
    let mut cov = Coverage::default();
    let mut idem = 1_u64;

    let first = run_epoch(
        seed,
        backing.host(),
        &mut oracle,
        &mut version_map,
        &mut rng,
        &mut trace,
        &mut cov,
        &mut idem,
        STEPS_PER_SEED / 2,
    );
    if let Err(err) = first {
        backing.cleanup();
        return Err(err);
    }
    if use_dir {
        backing = backing.reopen();
        cov.reopens += 1;
    }
    oracle.reopen_home();
    trace.reopen_home();
    let second = run_epoch(
        seed,
        backing.host(),
        &mut oracle,
        &mut version_map,
        &mut rng,
        &mut trace,
        &mut cov,
        &mut idem,
        STEPS_PER_SEED / 2,
    );
    backing.cleanup();
    second?;
    if let Some(dir) = trace_dir {
        trace
            .write(dir, seed)
            .map_err(|err| format!("trace write failed: {err}"))?;
    }
    Ok(cov)
}

fn seeds() -> Vec<u64> {
    let var = |name: &str| std::env::var(name).ok().and_then(|v| v.parse::<u64>().ok());
    match var("WORTH_DST_SEED") {
        Some(seed) => vec![seed],
        None => (0..var("WORTH_DST_SEEDS").unwrap_or(64)).collect(),
    }
}

#[test]
#[allow(clippy::disallowed_methods)]
fn seeded_differential_live_subscription_dst_agrees_with_oracle() {
    let seeds = seeds();
    let trace_dir = std::env::var_os("WORTH_DST_TRACE_DIR").map(PathBuf::from);
    let workers = std::thread::available_parallelism()
        .map_or(4, usize::from)
        .min(8)
        .min(seeds.len());
    let next = AtomicUsize::new(0);
    let failures = Mutex::new(Vec::new());
    let total = Mutex::new(Coverage::default());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                while let Some(&seed) = seeds.get(next.fetch_add(1, Ordering::Relaxed)) {
                    match run_seed(seed, trace_dir.as_deref()) {
                        Ok(cov) => total.lock().unwrap().add(cov),
                        Err(err) => failures.lock().unwrap().push(err),
                    }
                }
            });
        }
    });
    let failures = failures.into_inner().unwrap();
    assert!(
        failures.is_empty(),
        "{} seeds failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
    let total = total.into_inner().unwrap();
    eprintln!(
        "live-whole-result differential DST: {} seeds x {STEPS_PER_SEED} steps, {total:?}",
        seeds.len()
    );
    if seeds.len() >= 64 {
        assert!(total.toggles > 0, "must exercise status toggles");
        assert!(total.due_edits > 0, "must exercise due-date edits");
        assert!(
            total.edge_adds > 0,
            "must exercise BlockedBy edge additions"
        );
        assert!(
            total.edge_removes > 0,
            "must exercise BlockedBy edge removals"
        );
        assert!(
            total.p2_edits > 0,
            "must exercise unrelated project-2 edits"
        );
        assert!(total.grant_revokes > 0, "must exercise grant revocations");
        assert!(total.grant_restores > 0, "must exercise grant restorations");
        assert!(
            total.reopens > 0,
            "must exercise Directory Home close/reopen"
        );
        assert!(total.deliveries > 0, "must exercise live deliveries");
        assert!(
            total.pendings > 0,
            "must exercise non-invalidating Pending polls"
        );
        assert!(
            total.auth_denials > 0,
            "must exercise fail-closed AuthDenied outcomes"
        );
    }
}
