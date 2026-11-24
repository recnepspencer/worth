//! 64-seed Differential DST across ALL 4 stock checked index families
//! (`TextPrefix`, `Interval1D`, `Box2D`, `Box3D`) against a brute-force oracle.
use super::dst_driver::{FourFamilyDriver, FourFamilyProbe};
use super::dst_oracle::{verify_budget_refusal, FourFamilyOracle, PREFIXES, TITLES};
use super::operations::AssetSpec;
use super::schema::{FocusWindow, QueryBox2D, QueryBox3D};

const MAX_ROWS: u64 = 8;
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
    fn coord(&mut self) -> i64 {
        (self.range(25) as i64) - 12
    }
    fn half_open_span(&mut self) -> (i64, i64) {
        let a = self.coord();
        let len = (self.range(8) as i64) + 1;
        (a, a + len)
    }
    fn closed_span(&mut self) -> (i64, i64) {
        let a = self.coord();
        let len = self.range(8) as i64;
        (a, a + len)
    }
    fn asset_spec(&mut self, key: u64) -> AssetSpec {
        let title = TITLES[self.range(TITLES.len())];
        // 10% chance of degenerate empty/point span to stress half-open vs closed soundness.
        let iv = if self.range(10) == 0 {
            let p = self.coord();
            (p, p)
        } else {
            self.half_open_span()
        };
        let (x0, x1) = if self.range(10) == 0 {
            let p = self.coord();
            (p, p)
        } else {
            self.closed_span()
        };
        let (y0, y1) = self.closed_span();
        let (z0, z1) = self.closed_span();
        AssetSpec::i64_only(key, title, iv, [x0, y0, z0], [x1, y1, z1])
    }
    fn probe_window(&mut self) -> FocusWindow {
        let (start, end) = self.half_open_span();
        FocusWindow { start, end }
    }
    fn probe_box2d(&mut self) -> QueryBox2D {
        let (min_x, max_x) = self.half_open_span();
        let (min_y, max_y) = self.half_open_span();
        QueryBox2D {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }
    fn probe_box3d(&mut self) -> QueryBox3D {
        let (min_x, max_x) = self.closed_span();
        let (min_y, max_y) = self.closed_span();
        let (min_z, max_z) = self.closed_span();
        QueryBox3D {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }
}

#[derive(Default, Debug)]
struct Coverage {
    inserts: usize,
    updates: usize,
    deletes: usize,
    reopens: usize,
    queries: usize,
    watch_opens: usize,
    watch_deliveries: usize,
    watch_pending_verified: usize,
    budget_refusals: usize,
}

impl Coverage {
    fn add(&mut self, o: Self) {
        self.inserts += o.inserts;
        self.updates += o.updates;
        self.deletes += o.deletes;
        self.reopens += o.reopens;
        self.queries += o.queries;
        self.watch_opens += o.watch_opens;
        self.watch_deliveries += o.watch_deliveries;
        self.watch_pending_verified += o.watch_pending_verified;
        self.budget_refusals += o.budget_refusals;
    }
}

fn run_seed(seed: u64, driver: &mut FourFamilyDriver) -> Result<Coverage, String> {
    let mut rng = Rng::new(seed);
    let mut oracle = FourFamilyOracle::default();
    let result = (|| -> Result<Coverage, String> {
        let mut cov = Coverage::default();
        let mut reopened = false;
        let steps = if driver.is_durable() { 6 } else { STEPS };
        for step in 0..steps {
            let fail = |e: String| format!("seed {seed} step {step}: {e}");
            let roll = rng.range(100);
            if driver.is_durable() && !reopened && step == steps / 2 {
                driver.cold_reopen_and_verify_not_ready().map_err(fail)?;
                reopened = true;
                cov.reopens += 1;
            } else if seed < 2 && step == steps - 2 {
                verify_budget_refusal(&oracle).map_err(fail)?;
                cov.budget_refusals += 1;
            } else if !driver.has_watches() && roll < 18 {
                let probe = FourFamilyProbe {
                    prefix: PREFIXES[rng.range(PREFIXES.len())],
                    window: rng.probe_window(),
                    box2d: rng.probe_box2d(),
                    box3d: rng.probe_box3d(),
                };
                driver.open_watches(&probe, &oracle).map_err(fail)?;
                cov.watch_opens += 1;
            } else if driver.has_watches() && roll < 10 {
                driver.close_watches();
            } else {
                let existing: Vec<u64> = oracle.rows.keys().copied().collect();
                let absent: Vec<u64> = (1..=MAX_ROWS)
                    .filter(|k| !oracle.rows.contains_key(k))
                    .collect();
                if !absent.is_empty() && (existing.is_empty() || roll < 52) {
                    let key = absent[rng.range(absent.len())];
                    let spec = rng.asset_spec(key);
                    driver.insert(spec.clone());
                    oracle.rows.insert(key, spec);
                    cov.inserts += 1;
                } else if !existing.is_empty() && roll < 82 {
                    let key = existing[rng.range(existing.len())];
                    let old = oracle.rows.get(&key).cloned();
                    let spec = rng.asset_spec(key);
                    driver.update(old.as_ref(), spec.clone());
                    oracle.rows.insert(key, spec);
                    cov.updates += 1;
                } else if !existing.is_empty() {
                    let key = existing[rng.range(existing.len())];
                    let old = oracle.rows.remove(&key);
                    driver.delete(key, old.as_ref());
                    cov.deletes += 1;
                }
            }
            if driver.has_watches() {
                let (deliv, pend) = driver.poll_watches(&oracle).map_err(fail)?;
                cov.watch_deliveries += deliv;
                cov.watch_pending_verified += pend;
            }
            let probe = FourFamilyProbe {
                prefix: PREFIXES[rng.range(PREFIXES.len())],
                window: rng.probe_window(),
                box2d: rng.probe_box2d(),
                box3d: rng.probe_box3d(),
            };
            driver.query_all(&probe, &oracle).map_err(fail)?;
            cov.queries += 1;
        }
        Ok(cov)
    })();
    if !driver.is_durable() {
        let live: Vec<u64> = oracle.rows.keys().copied().collect();
        driver.reset_empty(&live);
    }
    result
}

#[test]
fn seeded_four_family_stock_indexes_agree_with_brute_force_oracle() {
    let seed_count = std::env::var("WORTH_DST_SEEDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(64);
    let mut failures = Vec::new();
    let mut total = Coverage::default();
    let mut mem_driver: Option<FourFamilyDriver> = None;
    for seed in 0..seed_count {
        let res = if seed == 0 {
            let mut durable = FourFamilyDriver::new(seed, true);
            run_seed(seed, &mut durable)
        } else {
            let drv = mem_driver.get_or_insert_with(|| FourFamilyDriver::new(seed, false));
            run_seed(seed, drv)
        };
        match res {
            Ok(c) => total.add(c),
            Err(e) => failures.push(e),
        }
    }
    assert!(
        failures.is_empty(),
        "{} seeds failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
    eprintln!("four-family stock-index DST: {seed_count} seeds, {total:?}");
    if seed_count >= 64 {
        assert!(total.inserts > 0);
        assert!(total.updates > 0);
        assert!(total.deletes > 0);
        assert!(total.reopens > 0);
        assert!(total.queries > 0);
        assert!(total.watch_opens > 0);
        assert!(total.watch_deliveries > 0);
        assert!(total.watch_pending_verified > 0);
        assert!(total.budget_refusals > 0);
    }
}
