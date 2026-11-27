//! The real name index's ingestion and resolution as a conformance trace for
//! the proved Lean model (`proofs/lean/persistent_naming`).
//!
//! Group lineage batches are applied to a `PersistentNameBranchIndex`: births,
//! retirements, revivals, splits and merges, declared carriers, two writes of
//! one name in one commit, and three batches refused part way (a carrier
//! declaration for an unborn name, a duplicate birth). Each batch records the
//! touched names' real timelines before and after, and the timeline
//! operations its events and carriers mean, in ingestion order (`IndexDelta`
//! models no event semantics; the checker applies these operations to the
//! recorded prior timelines and compares the result with Rust's); each carrier change records its prior, declared,
//! departed, joined and installed members (`carrierUpdate`). Over the final
//! index every name's `state_at` is recorded at every basis
//! (`TimelineSearch`, `NameResolution.stateAt`), and resolutions from every
//! name at every basis record their probes, frontier pops, edge visits and
//! live descendants (`NameResolution.resolve`). With
//! `WORTH_PERSISTENT_NAMING_TRACE_DIR=<dir>` the lines are written to
//! `<dir>/name_index.ndjson`, which `lake exe conformance` checks.
use std::fmt::Write as _;

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent as Event;
use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::lineage::data::name_resolution::{
    PersistentNameKey, PersistentNameSelection,
};
use crate::contracts::lineage::data::naming::{
    PersistentNameIdentity, PersistentNameIdentityOwnerPort as _,
};

use super::member_groups::carrier_delta;
use super::{
    Carriers, NameEntry, NameState, PersistentNameBranchIndex, PersistentNameCommittedBatch,
};

fn group(n: u8) -> AggregateIdentity {
    AggregateIdentity::from_bytes([n; 32])
}

fn key(n: u8) -> PersistentNameKey {
    PersistentNameKey::Group(group(n))
}

fn member(n: u8) -> PersistentNameIdentity {
    PersistentNameIdentity::from_minted_digest([n; 32])
}

fn byte(key: &PersistentNameKey) -> u8 {
    let PersistentNameKey::Group(group) = key else {
        panic!("group names only");
    };
    group.bytes()[0]
}

fn entry<T>(
    index: &PersistentNameBranchIndex,
    n: u8,
    read: impl FnOnce(&NameEntry) -> T,
) -> Option<T> {
    index.names.get(&key(n)).unwrap().as_deref().map(read)
}

/// A timeline as `[[commit, state]]`, `1` for live and `0` for ended.
fn timeline(entry: &NameEntry) -> String {
    let states: Vec<_> = entry
        .timeline
        .iter()
        .map(|(c, s)| [c.0, u64::from(matches!(s, NameState::Live(_)))])
        .collect();
    format!("{states:?}")
}

fn edges(entry: &NameEntry) -> String {
    let rows: Vec<String> = entry
        .forward_edges
        .iter()
        .map(|(c, to)| format!("[{},{:?}]", c.0, to.iter().map(byte).collect::<Vec<_>>()))
        .collect();
    format!("[{}]", rows.join(","))
}

fn timelines(index: &PersistentNameBranchIndex, keys: &[u8]) -> String {
    let rows: Vec<String> = keys
        .iter()
        .map(|n| {
            format!(
                "[{n},{}]",
                entry(index, *n, timeline).unwrap_or_else(|| "null".into())
            )
        })
        .collect();
    format!("[{}]", rows.join(","))
}

fn carriers(index: &PersistentNameBranchIndex, n: u8) -> Carriers {
    let live = entry(index, n, |e| {
        e.current_live().map(|live| live.carriers.clone())
    });
    live.flatten().unwrap_or_default()
}

fn bytes<'m>(members: impl IntoIterator<Item = &'m PersistentNameIdentity>) -> Vec<u8> {
    members.into_iter().map(|m| m.as_bytes()[0]).collect()
}

/// One batch: its commit, group events, declared carriers and the timeline
/// operations (`["insert"|"set", name, state]`) ingestion performs, in order.
struct Batch {
    commit: u64,
    events: Vec<Event>,
    declared: Vec<(u8, Vec<u8>)>,
    ops: &'static str,
    keys: &'static [u8],
    refused: bool,
}

fn apply(index: &mut PersistentNameBranchIndex, batch: &Batch, trace: &mut String) {
    let mut committed = PersistentNameCommittedBatch::new(CommitId(batch.commit))
        .with_group_events(batch.events.iter().copied());
    let mut changes = Vec::new();
    for (n, declared) in &batch.declared {
        let declared: Vec<_> = declared.iter().map(|m| member(*m)).collect();
        committed = committed.with_carriers(key(*n), declared.iter().copied());
        let prior = carriers(index, *n);
        let (left, joined) = carrier_delta(&prior, declared.iter());
        let rows = [
            bytes(&prior),
            bytes(&declared),
            bytes(&left),
            bytes(&joined),
        ];
        changes.push((*n, rows));
    }
    let prior = timelines(index, batch.keys);
    let result = index.apply(&committed);
    assert_eq!(result.is_err(), batch.refused, "{result:?}");
    let post = timelines(index, batch.keys);
    if batch.refused {
        assert_eq!(
            prior, post,
            "a refused batch leaves every touched name unchanged"
        );
    }
    let _ = writeln!(
        trace,
        r#"{{"kind":"index_batch","commit":{},"ops":{},"refused":{},"prior":{prior},"post":{post}}}"#,
        batch.commit, batch.ops, batch.refused
    );
    for (n, [prior, declared, left, joined]) in changes.into_iter().filter(|_| !batch.refused) {
        let post = bytes(&carriers(index, n));
        let _ = writeln!(
            trace,
            r#"{{"kind":"carriers","prior":{prior:?},"declared":{declared:?},"left":{left:?},"joined":{joined:?},"post":{post:?}}}"#
        );
    }
}

fn batches() -> Vec<Batch> {
    let born = |n| Event::Born { group: group(n) };
    let retired = |n| Event::Retired { group: group(n) };
    let revived = |n| Event::Revived { group: group(n) };
    let split = |source, fragment| Event::SplitFrom {
        fragment: group(fragment),
        source: group(source),
    };
    let merged = |absorbed, keeper| Event::MergedInto {
        absorbed: group(absorbed),
        keeper: group(keeper),
    };
    let b = |commit, events, declared, ops, keys, refused| Batch {
        commit,
        events,
        declared,
        ops,
        keys,
        refused,
    };
    vec![
        b(
            1,
            vec![born(1), born(2), born(3)],
            vec![],
            r#"[["insert",1,1],["insert",2,1],["insert",3,1]]"#,
            &[1, 2, 3],
            false,
        ),
        b(
            2,
            vec![retired(1), born(4), split(2, 4)],
            vec![],
            r#"[["insert",4,1],["set",1,0]]"#,
            &[1, 2, 4],
            false,
        ),
        // Refused at a carrier declaration for a name never born.
        b(
            3,
            vec![revived(1), retired(3)],
            vec![(9, vec![1])],
            r#"[["set",1,1],["set",3,0],["set",9,1]]"#,
            &[1, 3, 9],
            true,
        ),
        // A revival and a carrier change write one name twice in one commit.
        b(
            3,
            vec![revived(1)],
            vec![(1, vec![3]), (2, vec![1, 2])],
            r#"[["set",1,1],["set",1,1],["set",2,1]]"#,
            &[1, 2],
            false,
        ),
        b(
            4,
            vec![retired(2), merged(2, 4)],
            vec![],
            r#"[["set",2,0]]"#,
            &[2, 4],
            false,
        ),
        b(
            5,
            vec![retired(4), born(7), born(8), merged(4, 7), merged(4, 8)],
            vec![],
            r#"[["insert",7,1],["insert",8,1],["set",4,0]]"#,
            &[4, 7, 8],
            false,
        ),
        // Refused at a duplicate birth, the least event of the batch.
        b(
            6,
            vec![born(6), born(4)],
            vec![],
            r#"[["insert",4,1]]"#,
            &[4, 6],
            true,
        ),
        b(
            6,
            vec![retired(7), merged(7, 8)],
            vec![],
            r#"[["set",7,0]]"#,
            &[7, 8],
            false,
        ),
        b(
            7,
            vec![],
            vec![(1, vec![2, 4])],
            r#"[["set",1,1]]"#,
            &[1],
            false,
        ),
    ]
}

/// Every name's rows as the resolution model reads them.
fn index_rows(index: &PersistentNameBranchIndex, names: &[u8]) -> String {
    let rows: Vec<String> = names
        .iter()
        .filter_map(|n| entry(index, *n, |e| format!("[{n},{},{}]", timeline(e), edges(e))))
        .collect();
    format!("[{}]", rows.join(","))
}

#[test]
fn name_index_ingestion_and_resolution_trace_against_the_proved_model() {
    let mut index = PersistentNameBranchIndex::new(BranchId("main".into()), None);
    let mut trace = String::new();
    for batch in batches() {
        apply(&mut index, &batch, &mut trace);
    }
    let names = [1, 2, 3, 4, 6, 7, 8, 9];
    let rows = index_rows(&index, &names);
    for n in names {
        for basis in 0..=8 {
            if let Some(found) = entry(&index, n, |e| {
                e.state_at(CommitId(basis))
                    .map(|(since, s)| [since.0, u64::from(matches!(s, NameState::Live(_)))])
            }) {
                let found = found.map_or("null".into(), |f| format!("{f:?}"));
                let row = entry(&index, n, timeline).unwrap();
                let _ = writeln!(
                    trace,
                    r#"{{"kind":"state_at","timeline":{row},"basis":{basis},"found":{found}}}"#
                );
            }
            let outcome = index.resolve(
                key(n),
                CommitId(basis),
                &PersistentNameSelection::AllDescendants,
            );
            let metrics = outcome.metrics;
            let mut live: Vec<u8> = outcome
                .footprint
                .live
                .iter()
                .map(|target| byte(&target.key()))
                .collect();
            live.sort_unstable();
            let _ = writeln!(
                trace,
                r#"{{"kind":"resolve","names":{rows},"held":{n},"basis":{basis},"probes":{},"events":{},"edges":{},"live":{live:?}}}"#,
                metrics.index_probes, metrics.event_visits, metrics.edge_visits
            );
        }
    }
    if let Ok(dir) = std::env::var("WORTH_PERSISTENT_NAMING_TRACE_DIR") {
        std::fs::write(format!("{dir}/name_index.ndjson"), trace).unwrap();
    }
}
