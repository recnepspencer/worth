//! Independent reference model transcribed from
//! `specs/tla/program_transition/ProgramTransition.tla`, plus deterministic
//! RNG, action generator, and NDJSON trace serializer for `ProgramTransitionTrace.tla`.

use std::collections::{BTreeMap, BTreeSet};

pub const ROW_IDS: [&str; 3] = ["t1", "t2", "t3"];
pub const V2_VALUES: [&str; 3] = ["a", "b", "c"];
pub const PRIORITIES: [&str; 2] = ["Normal", "High"];
pub const DEFAULT_PRIORITY: &str = "Normal";

/// Deterministic SplitMix64 PRNG for seeded DST walks.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(0x9e37_79b9_7f4a_7c15))
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    pub fn pick(&mut self, len: usize) -> usize {
        (self.next_u64() as usize) % len
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RowState {
    pub present: bool,
    pub v2val: &'static str,
    pub priority: &'static str,
}

impl Default for RowState {
    fn default() -> Self {
        Self {
            present: false,
            v2val: "None",
            priority: "None",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Op {
    CreateV2,
    WriteV2 {
        row: &'static str,
        v2val: &'static str,
    },
    CloseHome,
    ReopenV2,
    OpenV3WithoutTransition,
    OpenV3WithTransitionBudgetStop,
    OpenV3WithTransition,
    OpenV2OnV3,
    WriteV3 {
        row: &'static str,
        v2val: &'static str,
        priority: &'static str,
    },
    ReopenV3,
}

/// Complete observable state matched between the real Directory Home and the TLA+ model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedState {
    pub disk_version: u32,
    pub home_open: bool,
    pub open_version: u32,
    pub index_state: &'static str,
    pub transition_runs: usize,
    pub outcome: &'static str,
    pub rows: BTreeMap<&'static str, RowState>,
}

/// Reference model of `ProgramTransition.tla`.
#[derive(Clone, Debug)]
pub struct ReferenceModel {
    pub disk_version: u32,
    pub home_open: bool,
    pub open_version: u32,
    pub index_state: &'static str,
    pub transition_runs: usize,
    pub outcome: &'static str,
    pub rows: BTreeMap<&'static str, RowState>,
    pub v2_rows_at_upgrade: BTreeMap<&'static str, (bool, &'static str)>,
    pub v3_edited_rows: BTreeSet<&'static str>,
}

impl Default for ReferenceModel {
    fn default() -> Self {
        let rows = ROW_IDS
            .into_iter()
            .map(|r| (r, RowState::default()))
            .collect();
        let snap = ROW_IDS.into_iter().map(|r| (r, (false, "None"))).collect();
        Self {
            disk_version: 0,
            home_open: false,
            open_version: 0,
            index_state: "None",
            transition_runs: 0,
            outcome: "Init",
            rows,
            v2_rows_at_upgrade: snap,
            v3_edited_rows: BTreeSet::new(),
        }
    }
}

impl ReferenceModel {
    pub fn snapshot(&self) -> ObservedState {
        ObservedState {
            disk_version: self.disk_version,
            home_open: self.home_open,
            open_version: self.open_version,
            index_state: self.index_state,
            transition_runs: self.transition_runs,
            outcome: self.outcome,
            rows: self.rows.clone(),
        }
    }

    pub fn rows_with_priority(&self, priority: &str) -> BTreeSet<&'static str> {
        self.rows
            .iter()
            .filter_map(|(&id, row)| (row.present && row.priority == priority).then_some(id))
            .collect()
    }

    pub fn apply(&mut self, op: &Op) {
        match *op {
            Op::CreateV2 => self.apply_create_v2(),
            Op::WriteV2 { row, v2val } => self.apply_write_v2(row, v2val),
            Op::CloseHome => self.apply_close(),
            Op::ReopenV2 => self.apply_reopen_v2(),
            Op::OpenV3WithoutTransition => {
                assert!(!self.home_open && self.disk_version == 2);
                self.outcome = "RefusedTransitionRequired";
            }
            Op::OpenV3WithTransitionBudgetStop => {
                assert!(!self.home_open && self.disk_version == 2);
                self.outcome = "RefusedBudgetStop";
            }
            Op::OpenV3WithTransition => self.apply_upgrade_v2_to_v3(),
            Op::OpenV2OnV3 => {
                assert!(!self.home_open && self.disk_version == 3);
                self.outcome = "RefusedDowngrade";
            }
            Op::WriteV3 {
                row,
                v2val,
                priority,
            } => self.apply_write_v3(row, v2val, priority),
            Op::ReopenV3 => self.apply_reopen_v3(),
        }
        self.assert_invariants();
    }

    fn apply_create_v2(&mut self) {
        assert_eq!(self.disk_version, 0);
        assert!(!self.home_open);
        self.disk_version = 2;
        self.home_open = true;
        self.open_version = 2;
        self.index_state = "None";
        self.outcome = "CreatedV2";
    }

    fn apply_write_v2(&mut self, row: &'static str, v2val: &'static str) {
        assert!(self.home_open && self.open_version == 2 && self.disk_version == 2);
        self.rows.insert(
            row,
            RowState {
                present: true,
                v2val,
                priority: "None",
            },
        );
        self.outcome = "WroteV2";
    }

    fn apply_close(&mut self) {
        assert!(self.home_open);
        self.home_open = false;
        self.open_version = 0;
        self.index_state = if self.disk_version == 3 {
            "Cold"
        } else {
            "None"
        };
        self.outcome = "Closed";
    }

    fn apply_reopen_v2(&mut self) {
        assert!(!self.home_open && self.disk_version == 2);
        self.home_open = true;
        self.open_version = 2;
        self.outcome = "ReopenedV2";
    }

    fn apply_upgrade_v2_to_v3(&mut self) {
        assert!(!self.home_open && self.disk_version == 2);
        self.disk_version = 3;
        self.home_open = true;
        self.open_version = 3;
        self.index_state = "Ready";
        self.transition_runs += 1;
        self.v3_edited_rows.clear();
        for (&id, row) in &mut self.rows {
            self.v2_rows_at_upgrade.insert(id, (row.present, row.v2val));
            if row.present {
                row.priority = DEFAULT_PRIORITY;
            }
        }
        self.outcome = "UpgradedV2ToV3";
    }

    fn apply_write_v3(&mut self, row: &'static str, v2val: &'static str, priority: &'static str) {
        assert!(self.home_open && self.open_version == 3 && self.disk_version == 3);
        self.rows.insert(
            row,
            RowState {
                present: true,
                v2val,
                priority,
            },
        );
        self.v3_edited_rows.insert(row);
        self.index_state = "Ready";
        self.outcome = "WroteV3";
    }

    fn apply_reopen_v3(&mut self) {
        assert!(!self.home_open && self.disk_version == 3);
        self.home_open = true;
        self.open_version = 3;
        self.index_state = "Cold";
        self.outcome = "ReopenedV3";
    }

    fn assert_invariants(&self) {
        assert!(self.transition_runs <= 1);
        if self.disk_version == 2 {
            assert!(self.rows.values().all(|row| row.priority == "None"));
        }
        if self.disk_version == 3 {
            assert_eq!(self.transition_runs, 1);
            assert!(matches!(self.index_state, "Cold" | "Ready"));
            if matches!(self.outcome, "UpgradedV2ToV3" | "WroteV3") {
                assert_eq!(self.index_state, "Ready");
            }
            for (&id, &(was_present, old_val)) in &self.v2_rows_at_upgrade {
                if was_present {
                    let row = &self.rows[id];
                    assert!(row.present, "row {id} lost across upgrade");
                    if !self.v3_edited_rows.contains(id) {
                        assert_eq!(row.v2val, old_val);
                        assert_eq!(row.priority, DEFAULT_PRIORITY);
                    }
                }
            }
        }
    }
}

fn push_v2_phase(rng: &mut Rng, ops: &mut Vec<Op>) {
    ops.push(Op::CreateV2);
    let v2_writes = 1 + rng.pick(3);
    for _ in 0..v2_writes {
        ops.push(Op::WriteV2 {
            row: ROW_IDS[rng.pick(ROW_IDS.len())],
            v2val: V2_VALUES[rng.pick(V2_VALUES.len())],
        });
    }
    ops.push(Op::CloseHome);
    if rng.pick(2) == 0 {
        ops.push(Op::OpenV3WithoutTransition);
    }
    if rng.pick(3) != 0 {
        ops.push(Op::OpenV3WithTransitionBudgetStop);
        ops.push(Op::ReopenV2);
        if rng.pick(2) == 0 {
            ops.push(Op::WriteV2 {
                row: ROW_IDS[rng.pick(ROW_IDS.len())],
                v2val: V2_VALUES[rng.pick(V2_VALUES.len())],
            });
        }
        ops.push(Op::CloseHome);
    }
    if rng.pick(2) == 0 {
        ops.push(Op::OpenV3WithoutTransition);
    }
}

fn push_v3_phase(rng: &mut Rng, ops: &mut Vec<Op>) {
    ops.push(Op::OpenV3WithTransition);
    let v3_writes = 1 + rng.pick(2);
    for idx in 0..v3_writes {
        let priority = if idx == 0 {
            "High"
        } else {
            PRIORITIES[rng.pick(PRIORITIES.len())]
        };
        ops.push(Op::WriteV3 {
            row: ROW_IDS[rng.pick(ROW_IDS.len())],
            v2val: V2_VALUES[rng.pick(V2_VALUES.len())],
            priority,
        });
    }
    ops.push(Op::CloseHome);
    ops.push(Op::OpenV2OnV3);
    ops.push(Op::ReopenV3);
    if rng.pick(2) == 0 {
        ops.push(Op::WriteV3 {
            row: ROW_IDS[rng.pick(ROW_IDS.len())],
            v2val: V2_VALUES[rng.pick(V2_VALUES.len())],
            priority: PRIORITIES[rng.pick(PRIORITIES.len())],
        });
    }
    ops.push(Op::CloseHome);
}

/// Generate a realistic, randomized sequence of operations exercising every
/// transition, refusal, budget-stop rollback, v3 edit, and idempotent reopen.
pub fn plan_seed_walk(seed: u64) -> Vec<Op> {
    let mut rng = Rng::new(seed);
    let mut ops = Vec::with_capacity(14);
    push_v2_phase(&mut rng, &mut ops);
    push_v3_phase(&mut rng, &mut ops);
    ops
}

/// NDJSON trace accumulator for `tools/tla/run_tlc.py trace`.
#[derive(Default)]
pub struct Trace {
    lines: Vec<String>,
}

impl Trace {
    pub fn record(&mut self, op: &Op, observed: &ObservedState) {
        let action_fields = match *op {
            Op::CreateV2 => "\"action\":\"CreateV2\"".to_owned(),
            Op::WriteV2 { row, v2val } => {
                format!("\"action\":\"WriteV2\",\"row\":\"{row}\",\"v2val\":\"{v2val}\"")
            }
            Op::CloseHome => "\"action\":\"CloseHome\"".to_owned(),
            Op::ReopenV2 => "\"action\":\"ReopenV2\"".to_owned(),
            Op::OpenV3WithoutTransition => "\"action\":\"OpenV3WithoutTransition\"".to_owned(),
            Op::OpenV3WithTransitionBudgetStop => {
                "\"action\":\"OpenV3WithTransitionBudgetStop\"".to_owned()
            }
            Op::OpenV3WithTransition => "\"action\":\"OpenV3WithTransition\"".to_owned(),
            Op::OpenV2OnV3 => "\"action\":\"OpenV2OnV3\"".to_owned(),
            Op::WriteV3 {
                row,
                v2val,
                priority,
            } => format!(
                "\"action\":\"WriteV3\",\"row\":\"{row}\",\"v2val\":\"{v2val}\",\"priority\":\"{priority}\""
            ),
            Op::ReopenV3 => "\"action\":\"ReopenV3\"".to_owned(),
        };
        let rows_json = ROW_IDS
            .iter()
            .map(|id| {
                let row = &observed.rows[id];
                format!(
                    "\"{id}\":{{\"present\":{},\"v2val\":\"{}\",\"priority\":\"{}\"}}",
                    row.present, row.v2val, row.priority
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        self.lines.push(format!(
            "{{{action_fields},\"postDiskVersion\":{},\"postHomeOpen\":{},\"postOpenVersion\":{},\"postIndexState\":\"{}\",\"postTransitionRuns\":{},\"postOutcome\":\"{}\",\"postRows\":{{{rows_json}}}}}",
            observed.disk_version,
            observed.home_open,
            observed.open_version,
            observed.index_state,
            observed.transition_runs,
            observed.outcome,
        ));
    }

    pub fn write(&self, dir: &std::path::Path, seed: u64) -> std::io::Result<std::path::PathBuf> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("seed-{seed}.ndjson"));
        std::fs::write(&path, self.lines.join("\n") + "\n")?;
        Ok(path)
    }
}
