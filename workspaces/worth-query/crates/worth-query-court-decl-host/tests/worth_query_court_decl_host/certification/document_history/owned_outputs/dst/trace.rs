//! NDJSON trace records for `specs/tla/tree_based_undo/TreeBasedUndoTrace.tla`.
//!
//! With `WORTH_DST_TRACE_DIR` set, each seed writes `seed-<n>.ndjson`, one
//! record per call. `run_tlc.py trace` validates the file: every record must
//! be a transition the model allows, with the outcome the harness saw, and
//! reach the post-state the harness read. Only real observations are written;
//! the oracle's states never are.
use super::reference::{Action, Intent, Observed, Outcome, Stage, Writer, NAMES};
use serde_json::{json, Map, Value};
use std::io::Write;
use std::path::PathBuf;

pub struct Trace {
    records: Vec<Value>,
    /// The document's selection, kept once the document can no longer be read
    /// (the model's `current` does not change after divergence).
    current: u32,
}

/// The directory traces go to, if any.
pub fn directory() -> Option<PathBuf> {
    std::env::var_os("WORTH_DST_TRACE_DIR").map(PathBuf::from)
}

fn stage(stage: Stage) -> i64 {
    match stage {
        Stage::Skip => -1,
        Stage::Retire => 0,
        Stage::Set(value) => value as i64,
    }
}

fn per_entity<T: Into<Value> + Copy>(values: [T; 2]) -> Value {
    let mut record = Map::new();
    record.insert(NAMES[0].into(), values[0].into());
    record.insert(NAMES[1].into(), values[1].into());
    Value::Object(record)
}

fn set(members: [bool; 2]) -> Value {
    let names: Vec<&str> = (0..2).filter(|e| members[*e]).map(|e| NAMES[e]).collect();
    json!({ "__set__": names })
}

fn key(key: u32) -> String {
    format!("k{key}")
}

fn outcome(outcome: &Outcome) -> &'static str {
    match outcome {
        Outcome::Recorded { .. } => "Recorded",
        Outcome::AlreadyRecorded { .. } => "AlreadyRecorded",
        Outcome::IntentDrift => "IntentDrift",
        Outcome::StageRefused => "StageRefused",
        Outcome::SourceRefused => "SourceRefused",
        Outcome::HeadDiverged => "HeadDiverged",
        Outcome::Empty => "Empty",
        Outcome::Stopped => "Stopped",
        Outcome::Environment(cause) => cause.trace_name(),
        Outcome::Moved => "Moved",
        Outcome::NavigationRefused => "Refused",
        Outcome::OutOfBand => "OutOfBand",
        Outcome::NoPendingRelease => "NoPendingRelease",
        Outcome::Released => "Closed",
        Outcome::Discarded => "Discarded",
        Outcome::Begun => "Begun",
        Outcome::Closed => "Closed",
    }
}

/// The envelope fields of a record: kind, stages, source and key.
fn envelope(fields: &mut Map<String, Value>, k: u32, intent: Option<&Intent>) {
    let (kind, stages, source, entity) = match intent {
        Some(Intent::Edit(stages)) => ("edit", *stages, 0, "none"),
        Some(Intent::Correction { source, entity, stages }) => {
            ("correction", *stages, *source, NAMES[*entity])
        }
        None => ("edit", [Stage::Skip; 2], 0, "none"),
    };
    fields.insert("kind".into(), kind.into());
    fields.insert("stages".into(), per_entity(stages.map(stage)));
    fields.insert("src".into(), source.into());
    fields.insert("ent".into(), entity.into());
    fields.insert("key".into(), key(k).into());
}

impl Trace {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            current: 0,
        }
    }

    /// One record: the call, its outcome, and the post-state read after it.
    /// `nodes` is the number of steps the document holds, the root included.
    pub fn push(&mut self, action: &Action, result: &Outcome, observed: &Observed, nodes: u32) {
        let mut fields = Map::new();
        let name = match action {
            Action::Envelope { key, intent, .. } => {
                envelope(&mut fields, *key, Some(intent));
                "Envelope"
            }
            Action::Refused { key, .. } | Action::Empty { key } => {
                envelope(&mut fields, *key, None);
                "Envelope"
            }
            Action::Undo => "Undo",
            Action::Redo(node) | Action::Checkout(node) => {
                fields.insert("node".into(), (*node).into());
                if matches!(action, Action::Redo(_)) { "Redo" } else { "Checkout" }
            }
            Action::OutOfBand(stages) => {
                fields.insert("stages".into(), per_entity(stages.map(stage)));
                "OutOfBand"
            }
            Action::Begin(writer) => {
                let class = match writer {
                    Writer::Edit => "edit",
                    Writer::Gesture => "gesture",
                };
                fields.insert("class".into(), class.into());
                "Begin"
            }
            Action::Commit { key: k, stages } => {
                fields.insert("stages".into(), per_entity(stages.map(stage)));
                fields.insert("key".into(), key(*k).into());
                "Commit"
            }
            Action::Maintain => "Maintain",
            Action::Discard => "Discard",
            Action::Close => "Close",
        };
        if let Outcome::AlreadyRecorded { node } = result {
            fields.insert("step".into(), (*node).into());
        }
        if let Some(current) = observed.current {
            self.current = current;
        }
        fields.insert("action".into(), name.into());
        fields.insert("outcome".into(), outcome(result).into());
        fields.insert("postCurrent".into(), self.current.into());
        fields.insert("postNextNode".into(), nodes.into());
        // Unreadable once close retired the branch: left out, as the trace
        // module allows a closed document.
        if let (Some(branch), Some((out_a, out_b))) = (&observed.branch, observed.outputs) {
            fields.insert("postLive".into(), set(branch.live));
            fields.insert("postGen".into(), per_entity(branch.generation));
            fields.insert("postVal".into(), per_entity(branch.value));
            fields.insert("postOutA".into(), per_entity(out_a));
            fields.insert("postOutB".into(), per_entity(out_b));
        }
        // Unobservable while a writer holds the document: left out, as the
        // trace module allows.
        if let Some(diverged) = observed.head_diverged {
            fields.insert("postHeadDiverged".into(), diverged.into());
        }
        fields.insert("postClosed".into(), observed.closed.into());
        // Readable whatever holds the document (ruling (u)).
        let reversals: Vec<Value> = observed
            .reversals
            .iter()
            .map(|fact| json!({ "step": fact.step, "src": fact.source, "ent": NAMES[fact.entity] }))
            .collect();
        fields.insert("postReversals".into(), json!({ "__set__": reversals }));
        self.records.push(Value::Object(fields));
    }

    /// Record each stopped call's point, one per call in order.
    pub fn set_points(&mut self, points: &[Option<&'static str>]) {
        for (record, point) in self.records.iter_mut().zip(points) {
            if let (Value::Object(fields), Some(point)) = (record, point) {
                fields.insert("point".into(), json!(point));
            }
        }
    }

    /// Write `seed-<seed>.ndjson` under `directory`.
    pub fn write(&self, directory: &std::path::Path, seed: u64) {
        std::fs::create_dir_all(directory).unwrap();
        let path = directory.join(format!("seed-{seed}.ndjson"));
        let mut file = std::fs::File::create(&path).unwrap();
        for record in &self.records {
            writeln!(file, "{record}").unwrap();
        }
    }
}
