//! 64-seed Differential DST runner over real WORTH Directory Homes comparing
//! every state, query, managed equality index, and typed refusal against the
//! independent `ProgramTransition.tla` reference model, plus passive NDJSON
//! trace export.

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};

use super::driver::Driver;
use super::reference_model::{plan_seed_walk, Op, ReferenceModel, Trace};

#[derive(Clone, Copy, Debug, Default)]
struct Coverage {
    v2_writes: usize,
    reopen_v2: usize,
    refused_without_transition: usize,
    budget_stops: usize,
    upgrades: usize,
    refused_downgrades: usize,
    v3_writes: usize,
    reopen_v3: usize,
}

impl Coverage {
    fn count(&mut self, op: &Op) {
        match op {
            Op::WriteV2 { .. } => self.v2_writes += 1,
            Op::ReopenV2 => self.reopen_v2 += 1,
            Op::OpenV3WithoutTransition => self.refused_without_transition += 1,
            Op::OpenV3WithTransitionBudgetStop => self.budget_stops += 1,
            Op::OpenV3WithTransition => self.upgrades += 1,
            Op::OpenV2OnV3 => self.refused_downgrades += 1,
            Op::WriteV3 { .. } => self.v3_writes += 1,
            Op::ReopenV3 => self.reopen_v3 += 1,
            Op::CreateV2 | Op::CloseHome => {}
        }
    }

    fn add(&mut self, other: Self) {
        self.v2_writes += other.v2_writes;
        self.reopen_v2 += other.reopen_v2;
        self.refused_without_transition += other.refused_without_transition;
        self.budget_stops += other.budget_stops;
        self.upgrades += other.upgrades;
        self.refused_downgrades += other.refused_downgrades;
        self.v3_writes += other.v3_writes;
        self.reopen_v3 += other.reopen_v3;
    }
}

fn run_seed(seed: u64, trace_dir: Option<&std::path::Path>) -> Result<Coverage, String> {
    let ops = plan_seed_walk(seed);
    let mut model = ReferenceModel::default();
    let mut driver = Driver::new(seed);
    let mut trace = Trace::default();
    let mut coverage = Coverage::default();

    for (idx, op) in ops.iter().enumerate() {
        coverage.count(op);
        driver
            .execute(op)
            .map_err(|e| format!("seed {seed} step {idx} ({op:?}): {e}"))?;
        model.apply(op);
        let observed = driver
            .observe(&model)
            .map_err(|e| format!("seed {seed} step {idx} observe ({op:?}): {e}"))?;
        let expected = model.snapshot();
        if observed != expected {
            return Err(format!(
                "seed {seed} step {idx} ({op:?}) diverged:\n  observed: {observed:?}\n  expected: {expected:?}"
            ));
        }
        trace.record(op, &observed);
    }

    if let Some(dir) = trace_dir {
        trace
            .write(dir, seed)
            .map_err(|e| format!("seed {seed} trace write failed: {e}"))?;
    }
    Ok(coverage)
}

fn seeds() -> Vec<u64> {
    let var = |name: &str| std::env::var(name).ok().and_then(|v| v.parse::<u64>().ok());
    match var("WORTH_DST_SEED") {
        Some(seed) => vec![seed],
        None => (0..var("WORTH_DST_SEEDS").unwrap_or(64)).collect(),
    }
}

#[test]
fn sixty_four_seed_directory_home_transition_dst_agrees_with_tla_model() {
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
    if seeds.len() >= 64 {
        for (label, count) in [
            ("WriteV2", total.v2_writes),
            ("ReopenV2", total.reopen_v2),
            ("OpenV3WithoutTransition", total.refused_without_transition),
            ("OpenV3WithTransitionBudgetStop", total.budget_stops),
            ("OpenV3WithTransition", total.upgrades),
            ("OpenV2OnV3", total.refused_downgrades),
            ("WriteV3", total.v3_writes),
            ("ReopenV3", total.reopen_v3),
        ] {
            assert!(count > 0, "64-seed DST never exercised {label}");
        }
    }
}
