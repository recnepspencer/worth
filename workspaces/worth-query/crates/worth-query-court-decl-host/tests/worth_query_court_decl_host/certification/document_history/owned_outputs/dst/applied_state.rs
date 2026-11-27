//! The applied-state digest, the S5 cutover oracle (TBU 2E plan §R3.3). After
//! every call of the default document sweep, the document's records are read
//! through public queries on a fresh request. That covers the members of model
//! `a` and the output rows of its tree. For each record the digest holds whether
//! it is live, its lineage and generation, and its value. Lineage is the record's
//! slot and generation its full identity; both are numbered densely in first-seen
//! order per document, so seeds that share a host do not depend on each other's
//! allocations. The schema declares no persistent names, so there is no live
//! name to digest. Event logs are left out on purpose.
//!
//! The digests the master build applied are pinned in
//! `golden/applied_state.ndjson`, one line per seed and corrections setting.
//! The document sweep compares against them only when it runs the default
//! sweep: the seed base, 64 seeds, 16 choices, no scenario.
//! `WORTH_DST_GOLDEN_WRITE=1` rewrites the lines for the sweep's corrections
//! setting instead.
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use worth_relational::facade::identity::EntityId;

thread_local!(static SWEEP: RefCell<Option<Vec<(u64, Vec<String>)>>> = const { RefCell::new(None) });

/// Collect digests on this thread until [`check`].
pub fn arm() {
    SWEEP.with(|sweep| *sweep.borrow_mut() = Some(Vec::new()));
}

/// The digests that follow belong to `seed`.
pub fn begin(seed: u64) {
    SWEEP.with(|sweep| {
        if let Some(seeds) = sweep.borrow_mut().as_mut() {
            seeds.push((seed, Vec::new()));
        }
    });
}

/// Whether a sweep is collecting, so readers skip the extra reads otherwise.
pub fn armed() -> bool {
    SWEEP.with(|sweep| sweep.borrow().is_some())
}

/// One call's digest for the current seed.
pub fn record(digest: String) {
    SWEEP.with(|sweep| {
        if let Some((_, steps)) = sweep
            .borrow_mut()
            .as_mut()
            .and_then(|seeds| seeds.last_mut())
        {
            steps.push(digest);
        }
    });
}

/// Dense first-seen numbers for one document's lineages and generations.
#[derive(Default)]
pub struct Numbering {
    lineages: BTreeMap<(u64, u64), u32>,
    generations: BTreeMap<EntityId, u32>,
}

impl Numbering {
    /// `key=L<lineage>g<generation>:<value>` for a live record, `key=-` otherwise.
    pub fn label(&mut self, key: &str, row: Option<(EntityId, u64)>) -> String {
        let Some((id, value)) = row else {
            return format!("{key}=-");
        };
        let next = self.lineages.len() as u32 + 1;
        let lineage = *self
            .lineages
            .entry((id.partition_id.0 as u64, id.local_slot.0))
            .or_insert(next);
        let next = self.generations.len() as u32 + 1;
        let generation = *self.generations.entry(id).or_insert(next);
        format!("{key}=L{lineage}g{generation}:{value}")
    }
}

fn golden() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "tests/worth_query_court_decl_host/certification/document_history/owned_outputs/dst/golden/applied_state.ndjson",
    )
}

fn lines() -> Vec<(bool, u64, Vec<String>)> {
    let text = std::fs::read_to_string(golden()).unwrap_or_default();
    text.lines()
        .map(|line| {
            let value: Value = serde_json::from_str(line).expect("a golden line is JSON");
            let seed = value["seed"]
                .as_str()
                .and_then(|seed| u64::from_str_radix(seed.trim_start_matches("0x"), 16).ok());
            let steps = value["steps"].as_array().map(|steps| {
                steps
                    .iter()
                    .map(|step| step.as_str().unwrap().to_owned())
                    .collect()
            });
            (
                value["corrections"].as_bool().unwrap(),
                seed.unwrap(),
                steps.unwrap(),
            )
        })
        .collect()
}

/// Compare the collected sweep with the pinned master digests for
/// `corrections`, or with `WORTH_DST_GOLDEN_WRITE=1` replace them.
pub fn check(corrections: bool) {
    let swept = SWEEP
        .with(|sweep| sweep.borrow_mut().take())
        .expect("armed before the sweep");
    let mut pinned = lines();
    if std::env::var("WORTH_DST_GOLDEN_WRITE").as_deref() == Ok("1") {
        pinned.retain(|(setting, _, _)| *setting != corrections);
        pinned.extend(
            swept
                .into_iter()
                .map(|(seed, steps)| (corrections, seed, steps)),
        );
        pinned.sort_by_key(|(setting, seed, _)| (*setting, *seed));
        let text: String = pinned
            .iter()
            .map(|(setting, seed, steps)| {
                format!(
                    "{}\n",
                    json!({"corrections": setting, "seed": format!("{seed:#x}"), "steps": steps})
                )
            })
            .collect();
        std::fs::create_dir_all(golden().parent().unwrap()).unwrap();
        std::fs::write(golden(), text).unwrap();
        eprintln!("applied-state golden rewritten for corrections={corrections}");
        return;
    }
    let pinned: Vec<(u64, Vec<String>)> = pinned
        .into_iter()
        .filter(|(setting, _, _)| *setting == corrections)
        .map(|(_, seed, steps)| (seed, steps))
        .collect();
    assert!(!pinned.is_empty(), "no applied-state golden for corrections={corrections}; capture it on master with WORTH_DST_GOLDEN_WRITE=1");
    let seeds =
        |sweep: &[(u64, Vec<String>)]| sweep.iter().map(|(seed, _)| *seed).collect::<Vec<_>>();
    assert_eq!(
        seeds(&swept),
        seeds(&pinned),
        "the sweep's seeds must be the pinned ones"
    );
    for ((seed, got), (_, want)) in swept.iter().zip(&pinned) {
        if let Some(step) =
            (0..got.len().max(want.len())).find(|&step| got.get(step) != want.get(step))
        {
            panic!(
                "applied state differs from master: seed {seed:#x} (corrections={corrections}) at call {step}\n  master: {:?}\n  now:    {:?}",
                want.get(step),
                got.get(step)
            );
        }
    }
}
