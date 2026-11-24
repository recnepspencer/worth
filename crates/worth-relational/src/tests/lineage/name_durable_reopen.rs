//! Arch 8, 18, 22: the durable persistent-name log is the only thing reopen
//! needs. After every commit boundary the derived index is corrupted and
//! discarded, and the index rebuilt from the persisted bytes alone equals
//! the live index and answers every resolution identically. A torn trailing
//! frame refuses typed and yields no index at all.

use worth_foundational::facade::AggregateIdentity;
use worth_relational_lineage::internal_port::lineage::index::batch::PersistentNameCommittedBatchFixturePort as _;

use crate::aggregate_identity::AggregateGroupLineageEvent;
use crate::facade::history::CommitId;
use crate::facade::lineage::{
    rebuild_name_index_from_durable_log, PersistentNameCommittedBatch,
    PersistentNameDurabilityError, PersistentNameIndexDenial, PersistentNameWireSection,
};

use super::name_durable_world::durable_naming_history;

#[test]
fn reopen_after_every_commit_boundary_rebuilds_the_exact_index_and_answers() {
    let history = durable_naming_history();
    let full = history.live_index(history.commits.len());
    let every_name = full.name_keys();
    let selections = history.selections();
    let mut compared = 0usize;
    for boundary in 0..=history.commits.len() {
        let live = history.live_index(boundary);
        let mut corrupted = live.clone();
        let next = CommitId(live.head().map_or(1, |head| head.0 + 1));
        corrupted
            .apply(&PersistentNameCommittedBatch::new(next).with_group_events([
                AggregateGroupLineageEvent::Born {
                    group: AggregateIdentity::from_bytes([0xee; 32]),
                },
            ]))
            .expect("a stray derived birth corrupts the in-memory index");
        assert_ne!(corrupted, live);
        drop(corrupted);

        let reopened = rebuild_name_index_from_durable_log(&history.persisted_through(boundary))
            .unwrap_or_else(|refusal| panic!("boundary {boundary} reopens: {refusal}"));
        assert_eq!(reopened, live, "boundary {boundary}: exact derived state");
        let head = live.head().map_or(0, |head| head.0);
        for name in &every_name {
            for selection in &selections {
                for basis in 0..=head + 1 {
                    let basis = CommitId(basis);
                    assert_eq!(
                        reopened.resolve(*name, basis, selection),
                        live.resolve(*name, basis, selection),
                        "boundary {boundary}: {name:?} at {basis:?} under {selection:?}"
                    );
                    compared += 1;
                }
            }
        }
    }
    assert!(compared > 2_000, "compared {compared} resolutions");
    let born = history
        .commits
        .iter()
        .map(|(batch, _)| {
            let births = batch
                .lineage_events_for_test()
                .iter()
                .filter(|e| e.name_birth().is_some());
            let groups = batch
                .group_events_for_test()
                .iter()
                .filter(|e| matches!(e, AggregateGroupLineageEvent::Born { .. }));
            births.count() + groups.count()
        })
        .sum::<usize>();
    assert_eq!(full.name_count(), born, "every persisted birth is held");
}

#[test]
fn torn_trailing_frames_and_headers_refuse_typed_without_an_index() {
    let history = durable_naming_history();
    let frames = history.frames();
    let header = history.header();
    for cut in 0..header.len() {
        assert!(
            matches!(
                rebuild_name_index_from_durable_log(&header[..cut]),
                Err(PersistentNameDurabilityError::Truncated {
                    section: PersistentNameWireSection::LogHeader,
                    ..
                })
            ),
            "header torn at {cut}"
        );
    }
    for (boundary, frame) in frames.iter().enumerate() {
        let prefix = history.persisted_through(boundary);
        let mut cuts = vec![1, 4, 5, 8, frame.len() / 2, frame.len() - 1];
        cuts.retain(|cut| *cut < frame.len());
        for cut in cuts {
            let mut torn = prefix.clone();
            torn.extend_from_slice(&frame[..cut]);
            let refusal = rebuild_name_index_from_durable_log(&torn)
                .expect_err("a torn trailing frame never yields an index");
            let section = if cut < 9 {
                PersistentNameWireSection::FrameHeader
            } else {
                PersistentNameWireSection::FramePayload
            };
            assert!(
                matches!(
                    refusal,
                    PersistentNameDurabilityError::Truncated { section: found, .. } if found == section
                ),
                "frame {boundary} torn at {cut}: {refusal:?}"
            );
        }
    }
}

#[test]
fn a_replayed_frame_refuses_as_non_monotonic_history() {
    let history = durable_naming_history();
    let frames = history.frames();
    let mut replayed = history.persisted_through(3);
    replayed.extend_from_slice(&frames[2]);
    assert_eq!(
        rebuild_name_index_from_durable_log(&replayed),
        Err(PersistentNameDurabilityError::IndexRefused {
            commit: history.commits[2].0.commit(),
            denial: PersistentNameIndexDenial::NonMonotonicCommit {
                head: history.commits[2].0.commit(),
                next: history.commits[2].0.commit(),
            },
        })
    );
}
