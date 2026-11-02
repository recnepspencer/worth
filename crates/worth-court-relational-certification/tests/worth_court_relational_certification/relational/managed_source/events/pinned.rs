//! Receipt-bearing pinned reads use the genuine published event graph and issuer.
use super::*;
use fixture::*;
use std::cell::Cell;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

fn pin(runtime: &RelationalRuntime, execution: &ExecutionRequest<'_, '_>) -> RelationalPinnedRead {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    runtime
        .read_truth()
        .pin_read_capability(&snapshot, &[], execution)
        .unwrap()
}
fn path(pin: &RelationalPinnedRead) -> DerivedRelationHandle {
    pin.schema()
        .derived_relation(&registry::parts().identity)
        .unwrap()
}
fn history(runtime: &RelationalRuntime, execution: &ExecutionRequest<'_, '_>) -> World {
    let world = seed(runtime, execution);
    let [a, b, c, d, _] = world.events;
    let [x, _, _] = world.rows;
    commit(
        runtime,
        execution,
        WorkerIntentBatch::new("pinned-event-history")
            .push(link(5, b, a, "ba"))
            .push(link(5, c, b, "cb"))
            .push(link(5, d, c, "dc"))
            .push(link(2, a, x, "ax"))
            .push(link(3, b, x, "bx"))
            .push(link(2, c, x, "cx")),
    );
    world
}
fn shape(reverse: bool) -> DerivedRelationReadShape {
    DerivedRelationReadShape::Adjacency { reverse }
}
#[test]
fn controlled_pinned_events_keep_exact_issuer_rosters_and_empty_negative_support() {
    with_event_probe(|runtime, _, host, _| {
        let execution = ExecutionRequest::serial(host);
        let w = history(runtime, &execution);
        let pin = pin(runtime, &execution);
        let path = path(&pin);
        let [a, _, c, d, u] = w.events;
        let [x, _, _] = w.rows;
        for (root, reverse, mut expected) in [
            (d, false, vec![x]),
            (x, true, vec![a, c, d]),
            (u, false, vec![]),
        ] {
            expected.sort();
            let calls = Cell::new(0);
            let control = || {
                calls.set(calls.get() + 1);
                Ok(())
            };
            let actual = pin
                .read_derived_source(
                    &execution,
                    PinnedDerivedRelationScope::new(&path, root, shape(reverse))
                        .with_control(&control),
                )
                .unwrap();
            assert_eq!(
                actual.data().endpoints(),
                expected
                    .iter()
                    .copied()
                    .map(NativeMemberKey::Entity)
                    .collect::<Vec<_>>()
            );
            assert!(calls.get() > 2);
            let old = pin
                .derived_source(&execution, &path, root, shape(reverse), None)
                .unwrap();
            assert_eq!(actual.data().endpoints(), old.data().endpoints());
            assert_eq!(
                actual.data().entity_dependencies(),
                old.data().entity_dependencies()
            );
            assert_eq!(
                actual.data().physical_relations(),
                old.data().physical_relations()
            );
            assert_eq!(
                actual.data().adjacency_dependencies(),
                old.data().adjacency_dependencies()
            );
            assert_eq!(actual.data().reads(), old.data().reads());
            assert_eq!(actual.work_units(), old.work_units());
            let events: &[EntityId] = if reverse {
                &w.events[..4]
            } else if root == u {
                &w.events[4..]
            } else {
                &w.events[..4]
            };
            for event in events {
                for relation in if reverse { &[3, 5][..] } else { &[2, 3, 5][..] } {
                    assert!(actual.data().adjacency_dependencies().contains(
                        &DerivedRelationAdjacency {
                            anchor: *event,
                            relation: KindId(*relation),
                            outgoing: true,
                        }
                    ));
                }
            }
            if root == u {
                assert!(actual.data().physical_relations().is_empty());
                assert_eq!(actual.data().entity_dependencies(), &[(u, KindId(1))]);
                assert_eq!(actual.data().adjacency_dependencies().len(), 3);
            }
        }
    });
}
#[test]
fn controlled_pinned_refusals_preserve_first_stop_and_acquired_receipts_through_final_handoff() {
    with_event_probe(|runtime, _, host, _| {
        let execution = ExecutionRequest::serial(host);
        let w = history(runtime, &execution);
        let pin = pin(runtime, &execution);
        let path = path(&pin);
        let root = w.events[3];
        let count = Cell::new(0usize);
        let count_control = || {
            count.set(count.get() + 1);
            Ok(())
        };
        let full = pin
            .read_derived_source(
                &execution,
                PinnedDerivedRelationScope::new(&path, root, shape(false))
                    .with_control(&count_control),
            )
            .unwrap();
        let total = count.get();
        assert!(total > 20);
        let mut prior_work = 0;
        for stop_at in [1, total / 2, total - 1, total] {
            let calls = Cell::new(0);
            let first = Cell::new(None);
            let control = || {
                calls.set(calls.get() + 1);
                if calls.get() == stop_at {
                    first.set(Some("original-query-adapter-stop"));
                    Err(Cause::NestedStopped)
                } else {
                    Ok(())
                }
            };
            let denied = pin
                .read_derived_source(
                    &execution,
                    PinnedDerivedRelationScope::new(&path, root, shape(false))
                        .with_control(&control),
                )
                .err()
                .expect("refusal cannot publish a partial or completed body");
            assert_eq!(first.get(), Some("original-query-adapter-stop"));
            assert_eq!(
                denied.kind,
                DerivedRelationReadDenial::Request(Cause::NestedStopped)
            );
            assert_eq!(
                calls.get(),
                stop_at,
                "no control callback after the first refusal"
            );
            assert!(denied.work >= prior_work);
            prior_work = denied.work;
            if stop_at == 1 {
                assert_eq!(denied.reads, DerivedRelationReads::default());
                assert_eq!(denied.work, 0);
            } else if stop_at == total / 2 {
                assert!(denied.reads.adjacency_lists > 0);
                assert!(
                    denied.reads.adjacency_lists < full.data().reads().adjacency_lists,
                    "mid-acquisition stop must interrupt physical rosters"
                );
            } else if stop_at >= total - 1 {
                assert_eq!(denied.reads, full.data().reads());
                assert_eq!(denied.work, full.work_units());
                assert!(denied.reads.adjacency_lists > 0 && denied.reads.relation_records > 0);
            }
        }
        let retry = pin
            .read_derived_source(
                &execution,
                PinnedDerivedRelationScope::new(&path, root, shape(false)),
            )
            .unwrap();
        assert_eq!(retry.data().endpoints(), &[w.rows[0].into()]);
    });
}
#[test]
fn controlled_pinned_limit_foreign_and_malformed_refusals_keep_actual_work() {
    with_event_probe(|runtime, _, host, _| {
        let execution = ExecutionRequest::serial(host);
        let w = history(runtime, &execution);
        let pin = pin(runtime, &execution);
        let path = path(&pin);
        let root = w.events[3];
        let limited = pin
            .read_derived_source(
                &execution,
                PinnedDerivedRelationScope::new(&path, root, shape(false)).with_work_limit(Some(0)),
            )
            .err()
            .unwrap();
        assert_eq!(
            limited.kind,
            DerivedRelationReadDenial::Definition(DerivedRelationDenial::WorkBudgetExceeded)
        );
        assert!(
            limited.work > 0,
            "the limit-crossing checkpoint was already admitted"
        );
        assert_eq!(limited.reads, DerivedRelationReads::default());
        let (_, _, foreign) = registry::profile();
        let denied = pin
            .read_derived_source(
                &execution,
                PinnedDerivedRelationScope::new(&foreign, root, shape(false)),
            )
            .err()
            .unwrap();
        assert_eq!(
            denied.kind,
            DerivedRelationReadDenial::Definition(DerivedRelationDenial::ForeignDefinition)
        );
        assert!(denied.work > 0);
        assert_eq!(denied.reads, DerivedRelationReads::default());
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("bad-pinned-predecessor").push(link(
                5,
                root,
                w.events[4],
                "du-second-predecessor",
            )),
        );
        let bad_pin = self::pin(runtime, &execution);
        let denied = bad_pin
            .read_derived_source(
                &execution,
                PinnedDerivedRelationScope::new(&path, root, shape(false)),
            )
            .err()
            .unwrap();
        assert_eq!(
            denied.kind,
            DerivedRelationReadDenial::Definition(DerivedRelationDenial::InvalidEventChain)
        );
        assert!(denied.work > limited.work);
        assert!(denied.reads.adjacency_lists > 0 && denied.reads.relation_records > 0);
        // The original immutable pin still proves its complete lawful history.
        assert_eq!(
            pin.derived_relations_from(&execution, &path, root, None)
                .unwrap()
                .data()
                .endpoints(),
            &[w.rows[0].into()]
        );
    });
}
#[test]
fn controlled_pinned_original_cancellation_refuses_without_erasing_completed_or_fresh_truth() {
    with_event_probe(|runtime, _, host, cancel| {
        let execution = ExecutionRequest::serial(host);
        let w = history(runtime, &execution);
        let pin = pin(runtime, &execution);
        let path = path(&pin);
        let full = pin
            .read_derived_source(
                &execution,
                PinnedDerivedRelationScope::new(&path, w.events[3], shape(false)),
            )
            .unwrap();
        let control = || {
            cancel.cancel();
            Ok(())
        };
        let denied = pin
            .read_derived_source(
                &execution,
                PinnedDerivedRelationScope::new(&path, w.events[3], shape(false))
                    .with_control(&control),
            )
            .err()
            .unwrap();
        let expected =
            DerivedRelationReadDenial::Request(execution.check_live().unwrap_err().into());
        assert_eq!(denied.kind, expected);
        assert_eq!(denied.reads, DerivedRelationReads::default());
        assert_eq!(full.data().endpoints(), &[w.rows[0].into()]);
        with_fresh_event_request(|fresh| {
            let request = ExecutionRequest::serial(fresh);
            let retry = pin
                .read_derived_source(
                    &request,
                    PinnedDerivedRelationScope::new(&path, w.events[3], shape(false)),
                )
                .unwrap();
            assert_eq!(retry.data().endpoints(), &[w.rows[0].into()]);
        });
    });
}

#[test]
fn controlled_pinned_stale_generation_retains_receipts_and_the_old_lawful_pin() {
    with_event_probe(|runtime, _, host, _| {
        let execution = ExecutionRequest::serial(host);
        let w = history(runtime, &execution);
        let old = pin(runtime, &execution);
        let path = path(&old);
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("pinned-retired-row").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent {
                    entity_id: w.rows[0],
                }),
            )),
        );
        let fresh = pin(runtime, &execution);
        let denied = fresh
            .read_derived_source(
                &execution,
                PinnedDerivedRelationScope::new(&path, w.rows[0], shape(true)),
            )
            .err()
            .unwrap();
        assert_eq!(
            denied.kind,
            DerivedRelationReadDenial::Definition(DerivedRelationDenial::RecordUnavailable)
        );
        assert_eq!(denied.reads, DerivedRelationReads::default());
        assert!(denied.work > 0);
        let before = old
            .read_derived_source(
                &execution,
                PinnedDerivedRelationScope::new(&path, w.rows[0], shape(true)),
            )
            .unwrap();
        let mut expected = [w.events[0].into(), w.events[2].into(), w.events[3].into()];
        expected.sort();
        assert_eq!(before.data().endpoints(), &expected);
    });
}
