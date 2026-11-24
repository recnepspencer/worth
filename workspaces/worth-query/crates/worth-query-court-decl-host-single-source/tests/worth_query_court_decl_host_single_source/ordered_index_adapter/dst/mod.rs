//! Single-harness 64-seed Differential DST + Sorted-Scan Oracle + TLC Trace Emitter.
//!
//! `WORTH_DST_SEEDS` (default 64) controls seed count, `WORTH_DST_SEED` runs one seed,
//! and `WORTH_DST_TRACE_DIR` writes NDJSON traces (`ordered_index_adapter.ndjson` for seed 0
//! and `seed-<N>.ndjson`) validated by `specs/tla/ordered_index_adapter/OrderedIndexAdapterTrace.tla`.
mod checks;
mod driver;
mod model;
mod trace;
mod watches;

use driver::DstDriver;
use model::{OracleTask, SortedScanOracle, MAX_COORD, MAX_ROWS, PREFIXES, TITLES};
use trace::Trace;

const STEPS: usize = 8;

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(0x9E37_79B9_7F4A_7C15))
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn range(&mut self, bound: usize) -> usize {
        (self.next_u64() as usize) % bound
    }
    fn interval(&mut self) -> (i64, i64) {
        let a = self.range((MAX_COORD + 1) as usize) as i64;
        let b = self.range((MAX_COORD + 1) as usize) as i64;
        if a <= b {
            (a, b)
        } else {
            (b, a)
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Coverage {
    inserts: usize,
    updates: usize,
    deletes: usize,
    reopens: usize,
    rebuilds: usize,
    queries: usize,
    cold_refusals: usize,
    watch_opens: usize,
    watch_deliveries: usize,
    watch_idle_polls: usize,
    budget_refusals: usize,
}

impl Coverage {
    fn add(&mut self, other: Self) {
        self.inserts += other.inserts;
        self.updates += other.updates;
        self.deletes += other.deletes;
        self.reopens += other.reopens;
        self.rebuilds += other.rebuilds;
        self.queries += other.queries;
        self.cold_refusals += other.cold_refusals;
        self.watch_opens += other.watch_opens;
        self.watch_deliveries += other.watch_deliveries;
        self.watch_idle_polls += other.watch_idle_polls;
        self.budget_refusals += other.budget_refusals;
    }
}

/// Rich durable Directory Home journey for seed 0: multi-node AVL tree (>= 4 live rows),
/// watch Open -> unrelated mutation -> Idle poll -> matching mutation -> Deliver ->
/// same-host budget refusal -> WatchClose -> 2-child Delete -> ColdReopen -> typed
/// NotReady refusal -> RebuildIndex -> Query.
fn run_durable_seed_zero(
    trace_dir: Option<&std::path::Path>,
    record_trace: bool,
) -> Result<Coverage, String> {
    let mut oracle = SortedScanOracle::default();
    let mut driver = DstDriver::new(0, Some(None), record_trace);
    let mut trace = Trace::default();
    let mut cov = Coverage::default();

    driver.rebuild(&oracle, &mut trace)?;
    cov.rebuilds += 1;

    // 1. Insert 4 live rows across distinct title families & overlapping/wide intervals.
    for (id, title, lo, hi) in [
        (1_u64, "Alpha", 2_i64, 5_i64),
        (2, "Beta", 8, 12),
        (3, "delta", 1, 14),
        (4, "Focus", 13, 15),
    ] {
        driver.insert(
            id,
            OracleTask {
                title: title.to_owned(),
                lo,
                hi,
            },
            &mut oracle,
            &mut trace,
        )?;
        cov.inserts += 1;
    }
    driver.query_both("al", 4, 9, &oracle, &mut trace)?;
    cov.queries += 1;

    // 2. Open watches on title "al" and focus [4, 9].
    driver.open_watches("al", 4, 9, &mut oracle, &mut trace)?;
    cov.watch_opens += 1;

    // 3. Unrelated insert: row 5 ("élan", [15, 16]) matches NEITHER watch -> must remain Idle.
    driver.insert(
        5,
        OracleTask {
            title: "élan".to_owned(),
            lo: 15,
            hi: 16,
        },
        &mut oracle,
        &mut trace,
    )?;
    cov.inserts += 1;
    let delivered = driver.poll_watches(&mut oracle, &mut trace)?;
    if delivered != 0 {
        return Err(format!("unrelated insert woke {delivered} watches"));
    }
    cov.watch_idle_polls += 1;

    // 4. Related update: row 2 becomes ("alpine", [10, 11]) -> enters "al" AND leaves [4, 9].
    driver.update(
        2,
        OracleTask {
            title: "alpine".to_owned(),
            lo: 10,
            hi: 11,
        },
        &mut oracle,
        &mut trace,
    )?;
    cov.updates += 1;
    cov.watch_deliveries += driver.poll_watches(&mut oracle, &mut trace)?;

    // 5. Same-host non-no-op budget refusal on live row 1 (attempts moving "Alpha" -> "Beta",
    //    a different title probe group, while keeping focus fields unchanged) -> rows, queries,
    //    and watches stay unchanged.
    driver.probe_same_host_budget_refusal(
        1,
        &OracleTask {
            title: "Beta".to_owned(),
            lo: 2,
            hi: 5,
        },
        "al",
        4,
        9,
        &oracle,
        &mut trace,
    )?;
    cov.budget_refusals += 1;
    let post_refusal_delivered = driver.poll_watches(&mut oracle, &mut trace)?;
    if post_refusal_delivered != 0 {
        return Err("refused budget edit spuriously woke a watch".to_owned());
    }
    cov.watch_idle_polls += 1;

    // 6. Close watches and exercise 2-child deletes across the 5-node AVL tree.
    driver.close_watches(&mut oracle, &mut trace)?;
    driver.delete(2, &mut oracle, &mut trace)?;
    driver.delete(3, &mut oracle, &mut trace)?;
    cov.deletes += 2;
    driver.query_both("al", 0, 16, &oracle, &mut trace)?;
    cov.queries += 1;

    // 7. Cold Directory Home reopen -> typed NotReady refusal -> RebuildIndex -> Query.
    driver.cold_reopen(&mut oracle, &mut trace)?;
    cov.reopens += 1;
    driver.query_both("al", 0, 16, &oracle, &mut trace)?;
    cov.cold_refusals += 1;
    driver.rebuild(&oracle, &mut trace)?;
    cov.rebuilds += 1;
    driver.query_both("al", 0, 16, &oracle, &mut trace)?;
    cov.queries += 1;

    if record_trace && let Some(dir) = trace_dir {
        trace
            .write(dir, 0)
            .map_err(|e| format!("seed 0 trace write: {e}"))?;
    }
    Ok(cov)
}

fn run_memory_seed(
    seed: u64,
    driver: &mut DstDriver,
    trace_dir: Option<&std::path::Path>,
    record_trace: bool,
) -> Result<Coverage, String> {
    let mut rng = Rng::new(seed);
    let mut oracle = SortedScanOracle::default();
    driver.set_tracing(record_trace);
    let mut trace = Trace::default();

    let result = (|| -> Result<Coverage, String> {
        let mut cov = Coverage::default();
        driver.rebuild(&oracle, &mut trace)?;
        cov.rebuilds += 1;

        for step in 0..STEPS {
            let fail = |cause: String| format!("seed {seed} step {step}: {cause}");
            let roll = rng.range(100);
            if !driver.has_watch() && roll < 18 {
                let prefix = PREFIXES[rng.range(PREFIXES.len())];
                let (w_lo, w_hi) = rng.interval();
                driver
                    .open_watches(prefix, w_lo, w_hi, &mut oracle, &mut trace)
                    .map_err(fail)?;
                cov.watch_opens += 1;
            } else if driver.has_watch() && roll < 10 {
                driver
                    .close_watches(&mut oracle, &mut trace)
                    .map_err(fail)?;
            } else {
                let existing: Vec<u64> = oracle.rows.keys().copied().collect();
                let absent: Vec<u64> = (1..=MAX_ROWS)
                    .filter(|k| !oracle.rows.contains_key(k))
                    .collect();
                if !absent.is_empty() && (existing.is_empty() || roll < 50) {
                    let key = absent[rng.range(absent.len())];
                    let title = TITLES[rng.range(TITLES.len())].to_owned();
                    let (lo, hi) = rng.interval();
                    driver
                        .insert(key, OracleTask { title, lo, hi }, &mut oracle, &mut trace)
                        .map_err(fail)?;
                    cov.inserts += 1;
                } else if !existing.is_empty() && roll < 80 {
                    let key = existing[rng.range(existing.len())];
                    let title = TITLES[rng.range(TITLES.len())].to_owned();
                    let (lo, hi) = rng.interval();
                    driver
                        .update(key, OracleTask { title, lo, hi }, &mut oracle, &mut trace)
                        .map_err(fail)?;
                    cov.updates += 1;
                } else if !existing.is_empty() {
                    let key = existing[rng.range(existing.len())];
                    driver.delete(key, &mut oracle, &mut trace).map_err(fail)?;
                    cov.deletes += 1;
                }
            }
            if driver.has_watch() {
                let deliveries = driver.poll_watches(&mut oracle, &mut trace).map_err(fail)?;
                if deliveries == 0 {
                    cov.watch_idle_polls += 1;
                } else {
                    cov.watch_deliveries += deliveries;
                }
            }
            let prefix = PREFIXES[rng.range(PREFIXES.len())];
            let (q_lo, q_hi) = rng.interval();
            driver
                .query_both(prefix, q_lo, q_hi, &oracle, &mut trace)
                .map_err(fail)?;
            cov.queries += 1;
        }

        if record_trace && let Some(dir) = trace_dir {
            trace
                .write(dir, seed)
                .map_err(|e| format!("seed {seed} trace write: {e}"))?;
        }
        Ok(cov)
    })();
    let remaining: Vec<u64> = oracle.rows.keys().copied().collect();
    driver.reset_empty(&remaining);
    result
}

fn seeds() -> Vec<u64> {
    let var = |name: &str| std::env::var(name).ok().and_then(|v| v.parse::<u64>().ok());
    match var("WORTH_DST_SEED") {
        Some(seed) => vec![seed],
        None => (0..var("WORTH_DST_SEEDS").unwrap_or(64)).collect(),
    }
}

#[test]
fn seeded_ordered_index_walks_agree_with_sorted_scan_oracle() {
    let seeds = seeds();
    let single_seed = seeds.len() == 1;
    let trace_dir = std::env::var_os("WORTH_DST_TRACE_DIR").map(std::path::PathBuf::from);
    let mut failures = Vec::new();
    let mut total = Coverage::default();
    let mut mem_driver: Option<DstDriver> = None;
    for &seed in &seeds {
        let record_trace = trace_dir.is_some() && (seed == 0 || single_seed);
        let result = if seed == 0 {
            run_durable_seed_zero(trace_dir.as_deref(), record_trace)
        } else {
            let driver = mem_driver.get_or_insert_with(|| DstDriver::new(seed, None, false));
            run_memory_seed(seed, driver, trace_dir.as_deref(), record_trace)
        };
        match result {
            Ok(cov) => total.add(cov),
            Err(err) => failures.push(err),
        }
    }
    assert!(
        failures.is_empty(),
        "{} seeds failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
    eprintln!(
        "ordered-index-adapter DST: {} seeds x {STEPS} steps, {total:?}",
        seeds.len()
    );
    if seeds.len() >= 64 {
        for (label, count) in [
            ("inserts", total.inserts),
            ("updates", total.updates),
            ("deletes", total.deletes),
            ("cold reopens", total.reopens),
            ("rebuilds", total.rebuilds),
            ("queries", total.queries),
            ("cold NotReady refusals", total.cold_refusals),
            ("watch opens", total.watch_opens),
            ("watch deliveries", total.watch_deliveries),
            ("watch idle polls", total.watch_idle_polls),
            ("budget refusals", total.budget_refusals),
        ] {
            assert!(count > 0, "64 seeds never exercised {label}");
        }
    }
}
