//! Reference Schedule Oracle, deterministic RNG, and passive-observation NDJSON trace emitter.

use std::collections::{BTreeMap, BTreeSet};

use super::super::{adapters::FiredOccurrence, schema::ReminderRow};

pub(super) const INTENTS: [&str; 2] = ["i1", "i2"];

pub(super) struct Rng(u64);
impl Rng {
    pub(super) fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(0x9E37_79B9_7F4A_7C15))
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub(super) fn range(&mut self, low: u64, high_inclusive: u64) -> u64 {
        low + (self.next_u64() % (high_inclusive - low + 1))
    }
    pub(super) fn pick(&mut self, bound: usize) -> usize {
        (self.next_u64() as usize) % bound
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct OracleIntent {
    pub(super) lifecycle: &'static str,
    pub(super) rev: u64,
    pub(super) due: u64,
    pub(super) interval: u64,
    pub(super) max_fires: u64,
    pub(super) fired_count: u64,
}
impl OracleIntent {
    pub(super) fn absent() -> Self {
        Self {
            lifecycle: "Absent",
            rev: 0,
            due: 0,
            interval: 0,
            max_fires: 0,
            fired_count: 0,
        }
    }
}

pub(super) struct ScheduleOracle {
    pub(super) intents: BTreeMap<&'static str, OracleIntent>,
    fired_keys: BTreeSet<(String, u64)>,
    total_fired: usize,
}
impl ScheduleOracle {
    pub(super) fn new() -> Self {
        let mut intents = BTreeMap::new();
        for id in INTENTS {
            intents.insert(id, OracleIntent::absent());
        }
        Self {
            intents,
            fired_keys: BTreeSet::new(),
            total_fired: 0,
        }
    }

    pub(super) fn due_active_count(&self, now: u64) -> usize {
        self.intents
            .values()
            .filter(|i| i.lifecycle == "Active" && i.due <= now)
            .count()
    }

    pub(super) fn record_fire(&mut self, fire: &FiredOccurrence, observed_now: u64) {
        let entry = self
            .intents
            .get_mut(fire.id.as_str())
            .expect("known intent id");
        assert_eq!(entry.lifecycle, "Active", "only Active intent may fire");
        assert!(
            entry.due <= observed_now,
            "NoPrematureFire violated: due={} > now={observed_now}",
            entry.due
        );
        assert_eq!(fire.rev, entry.rev, "fired revision must match active rev");
        assert_eq!(fire.due, entry.due, "fired due must match active due");
        assert!(
            self.fired_keys.insert((fire.id.clone(), fire.rev)),
            "NoDoubleFire violated for ({}, {})",
            fire.id,
            fire.rev
        );
        self.total_fired += 1;
        entry.fired_count += 1;
        assert_eq!(fire.fired_count_after, entry.fired_count);
        assert!(
            entry.fired_count <= entry.max_fires,
            "TerminalIntentsStopFiring violated"
        );
        entry.rev += 1;
        if entry.fired_count == entry.max_fires || entry.interval == 0 {
            entry.lifecycle = "Completed";
        } else {
            entry.due += entry.interval;
        }
    }

    pub(super) fn verify_against_rows(&self, rows: &BTreeMap<String, ReminderRow>) {
        let observed_total: u64 = INTENTS.iter().map(|id| rows[*id].fired_count).sum();
        assert_eq!(
            observed_total as usize, self.total_fired,
            "total committed fired_count across Relational rows mismatch"
        );
        for id in INTENTS {
            let expected = &self.intents[id];
            let actual = &rows[id];
            assert_eq!(tla_lifecycle(&actual.lifecycle), expected.lifecycle);
            assert_eq!(actual.revision, expected.rev);
            assert_eq!(actual.due, expected.due);
            assert_eq!(actual.fired_count, expected.fired_count);
        }
    }
}

pub(super) fn passive_total_fired(rows: &BTreeMap<String, ReminderRow>) -> usize {
    INTENTS
        .iter()
        .map(|id| rows[*id].fired_count as usize)
        .sum()
}

pub(super) fn tla_lifecycle(raw: &str) -> &'static str {
    match raw {
        "active" => "Active",
        "completed" => "Completed",
        "cancelled" => "Cancelled",
        _ => "Absent",
    }
}

pub(super) fn empty_rows() -> BTreeMap<String, ReminderRow> {
    INTENTS
        .iter()
        .map(|&id| {
            (
                id.to_owned(),
                ReminderRow {
                    identity: id.to_owned(),
                    revision: 0,
                    due: 0,
                    interval: 0,
                    max_fires: 0,
                    fired_count: 0,
                    lifecycle: "absent".to_owned(),
                },
            )
        })
        .collect()
}

pub(super) struct TraceEmitter {
    lines: Vec<String>,
}
impl TraceEmitter {
    pub(super) fn new() -> Self {
        Self { lines: Vec::new() }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn push(
        &mut self,
        action: &str,
        extras: serde_json::Value,
        app_state: &str,
        clock: u64,
        observed_clock: u64,
        auth_valid: bool,
        last_status: &str,
        total_fired: usize,
        rows: &BTreeMap<String, ReminderRow>,
    ) {
        let mut obj = extras.as_object().cloned().unwrap_or_default();
        obj.insert("action".into(), serde_json::json!(action));
        obj.insert("postAppState".into(), serde_json::json!(app_state));
        obj.insert("postClock".into(), serde_json::json!(clock));
        obj.insert(
            "postObservedClock".into(),
            serde_json::json!(observed_clock),
        );
        obj.insert("postAuthValid".into(), serde_json::json!(auth_valid));
        obj.insert("postLastPumpStatus".into(), serde_json::json!(last_status));
        obj.insert("postTotalFired".into(), serde_json::json!(total_fired));
        let mut intents_map = serde_json::Map::new();
        for id in INTENTS {
            let row = &rows[id];
            intents_map.insert(
                id.to_owned(),
                serde_json::json!({
                    "lifecycle": tla_lifecycle(&row.lifecycle),
                    "rev": row.revision,
                    "due": row.due,
                    "firedCount": row.fired_count,
                }),
            );
        }
        obj.insert("postIntents".into(), serde_json::Value::Object(intents_map));
        self.lines.push(serde_json::Value::Object(obj).to_string());
    }

    pub(super) fn push_reset(&mut self) {
        let empty = empty_rows();
        self.push(
            "Reset",
            serde_json::json!({}),
            "Open",
            0,
            0,
            true,
            "Idle",
            0,
            &empty,
        );
    }

    pub(super) fn flush_if_configured(&self) {
        let Ok(dir) = std::env::var("WORTH_DST_TRACE_DIR") else {
            return;
        };
        std::fs::create_dir_all(&dir).expect("create WORTH_DST_TRACE_DIR");
        let path = std::path::Path::new(&dir).join("temporal_catchup.ndjson");
        let body = format!("{}\n", self.lines.join("\n"));
        std::fs::write(&path, body).expect("write temporal_catchup.ndjson");
    }
}
