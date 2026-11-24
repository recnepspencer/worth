//! Single-harness 64-seed Differential DST + Sorted-Scan Oracle + TLC Trace Emitter.
//!
//! `WORTH_DST_SEEDS` (default 64) controls seed count, `WORTH_DST_SEED` runs one seed,
//! and `WORTH_DST_TRACE_DIR` writes NDJSON traces (`ordered_index_adapter.ndjson` for seed 0
//! and `seed-<N>.ndjson`) validated by `specs/tla/ordered_index_adapter/OrderedIndexAdapterTrace.tla`.
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
        self.budget_refusals += other.budget_refusals;
    }
}

fn run_seed_with_driver(
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
        let mut reopened_once = false;
        let mut budget_checked = false;

        driver.rebuild(&mut trace);
        cov.rebuilds += 1;

        let steps = if driver.is_durable() { 6 } else { STEPS };
        for step in 0..steps {
            let fail = |cause: String| format!("seed {seed} step {step}: {cause}");
            let roll = rng.range(100);
            if driver.is_durable() && !reopened_once && step == steps / 2 {
                driver.cold_reopen(&mut trace);
                cov.reopens += 1;
                reopened_once = true;
                let (q_lo, q_hi) = rng.interval();
                driver
                    .query_both(
                        PREFIXES[rng.range(PREFIXES.len())],
                        q_lo,
                        q_hi,
                        &oracle,
                        &mut trace,
                    )
                    .map_err(fail)?;
                cov.cold_refusals += 1;
                driver.rebuild(&mut trace);
                cov.rebuilds += 1;
                continue;
            }
            if !budget_checked && step == steps - 2 && seed < 2 {
                driver
                    .probe_budget_refusal(&oracle, &mut trace)
                    .map_err(fail)?;
                cov.budget_refusals += 1;
                budget_checked = true;
            } else if !driver.has_watch() && roll < 18 {
                let prefix = PREFIXES[rng.range(PREFIXES.len())];
                let (w_lo, w_hi) = rng.interval();
                driver
                    .open_watches(prefix, w_lo, w_hi, &oracle, &mut trace)
                    .map_err(fail)?;
                cov.watch_opens += 1;
            } else if driver.has_watch() && roll < 10 {
                driver.close_watches(&mut trace);
            } else {
                let existing: Vec<u64> = oracle.rows.keys().copied().collect();
                let absent: Vec<u64> = (1..=MAX_ROWS)
                    .filter(|k| !oracle.rows.contains_key(k))
                    .collect();
                if !absent.is_empty() && (existing.is_empty() || roll < 50) {
                    let key = absent[rng.range(absent.len())];
                    let title = TITLES[rng.range(TITLES.len())];
                    let (lo, hi) = rng.interval();
                    driver.insert(key, title, lo, hi, &mut trace);
                    oracle.rows.insert(
                        key,
                        OracleTask {
                            title: title.to_owned(),
                            lo,
                            hi,
                        },
                    );
                    cov.inserts += 1;
                } else if !existing.is_empty() && roll < 80 {
                    let key = existing[rng.range(existing.len())];
                    let title = TITLES[rng.range(TITLES.len())];
                    let (lo, hi) = rng.interval();
                    driver.update(key, title, lo, hi, &mut trace);
                    oracle.rows.insert(
                        key,
                        OracleTask {
                            title: title.to_owned(),
                            lo,
                            hi,
                        },
                    );
                    cov.updates += 1;
                } else if !existing.is_empty() {
                    let key = existing[rng.range(existing.len())];
                    driver.delete(key, &mut trace);
                    oracle.rows.remove(&key);
                    cov.deletes += 1;
                }
            }
            if driver.has_watch() && driver.poll_watches(&oracle, &mut trace).map_err(fail)? {
                cov.watch_deliveries += 1;
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
    if !driver.is_durable() {
        let remaining: Vec<u64> = oracle.rows.keys().copied().collect();
        driver.reset_empty(&remaining);
    }
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
            let mut durable = DstDriver::new(seed, true, record_trace);
            run_seed_with_driver(seed, &mut durable, trace_dir.as_deref(), record_trace)
        } else {
            let driver = mem_driver.get_or_insert_with(|| DstDriver::new(seed, false, false));
            run_seed_with_driver(seed, driver, trace_dir.as_deref(), record_trace)
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
            ("budget refusals", total.budget_refusals),
        ] {
            assert!(count > 0, "64 seeds never exercised {label}");
        }
    }
}
