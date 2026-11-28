//! Applied managed reads see earlier writes and retire with the candidate.
use super::*;
use worth_query_decl::facade::application_program::ApplicationNoOutputGraph;

#[test]
fn producer_computation_reads_earlier_peer_edits_and_refusal_preserves_the_index() {
    lifecycle::run(|| {
        for refuses in [false, true] {
            let (world, _) = fresh_world();
            author(&world, "peer", "disable", 0x3100_0010);
            author(&world, "peer", "remove", 0x3100_0011);
            let scope = authentication::request_scope();
            let principal = principal(&world, &scope);
            let request = world
                .runtime()
                .request(&principal, &scope)
                .on_branch(world.current_world());
            let index = request
                .query(find("peer", ""))
                .open_index::<prefix::PrefixIndex>()
                .unwrap();
            let head = request
                .retain_current_read()
                .unwrap()
                .selected_commit()
                .clone();
            let before = request.query(find("peer", "")).execute().unwrap();
            assert_eq!(
                before
                    .rows()
                    .iter()
                    .map(|row| row.identity.as_str())
                    .collect::<Vec<_>>(),
                ["disable", "remove"]
            );

            let mut envelope = request
                .edit_envelope()
                .idempotency(0x3100_0012u64)
                .runtime_stages(&world)
                .ordered_authored()
                .unwrap();
            envelope
                .push_stage::<ApplicationNoOutputGraph, _>(Edit::Item {
                    directory: "peer".into(),
                    identity: "added".into(),
                    name: "added".into(),
                })
                .unwrap();
            envelope
                .push_stage::<ApplicationNoOutputGraph, _>(Edit::SetEnabled {
                    directory: "peer".into(),
                    item: "disable".into(),
                    enabled: false,
                })
                .unwrap();
            envelope
                .push_stage::<ApplicationNoOutputGraph, _>(Edit::RetireItem {
                    directory: "peer".into(),
                    item: "remove".into(),
                })
                .unwrap();
            envelope
                .push_stage::<program::Root, _>(source::operation::RequestItem {
                    directory: "catalog".into(),
                    name: if refuses { "taken" } else { "producer-output" }.into(),
                })
                .unwrap();
            OBSERVED.with(|all| all.borrow_mut().clear());
            PRODUCER_PROBE.set(Some(ProducerProbe::Peer("peer")));
            THROUGH_COMPUTATION.set(true);
            let outcome = envelope
                .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            PRODUCER_PROBE.set(None);
            THROUGH_COMPUTATION.set(false);
            assert_eq!(
                OBSERVED.with(|all| std::mem::take(&mut *all.borrow_mut())),
                vec![Ok(vec!["added".to_owned()])]
            );
            match outcome {
                WorthQueryEditEnvelopeOutcome::Committed(_) if !refuses => {}
                WorthQueryEditEnvelopeOutcome::Uncommitted(
                    WorthQueryApplicationUncommitted::Denied(denial),
                ) if refuses => {
                    let Some(InvariantViolationFields::CustomInvariantViolation {
                        identity, ..
                    }) = denial.custom_invariant_denial()
                    else {
                        panic!("expected the installed name invariant: {denial:?}");
                    };
                    assert_eq!(identity.rule_id.as_str(), "AtMostOneTakenName");
                }
                WorthQueryEditEnvelopeOutcome::Uncommitted(cause) => {
                    panic!("peer edit did not commit: {cause:?}")
                }
                WorthQueryEditEnvelopeOutcome::StageExecutionDenied(cause) => panic!(
                    "peer stage {} refused: {:?}",
                    cause.binding(),
                    cause.cause()
                ),
                other => panic!(
                    "unexpected peer edit outcome: {:?}",
                    std::mem::discriminant(&other)
                ),
            }
            let after = request.query(find("peer", "")).execute().unwrap();
            if refuses {
                assert_eq!(
                    request.retain_current_read().unwrap().selected_commit(),
                    &head
                );
                assert_eq!(after.rows(), before.rows());
            } else {
                assert_eq!(
                    after
                        .rows()
                        .iter()
                        .map(|row| row.identity.as_str())
                        .collect::<Vec<_>>(),
                    ["added"]
                );
            }
            index.close();
        }
    });
}

#[test]
fn producer_computation_reads_new_ordered_scope_with_resident_or_evicted_predecessor() {
    use super::super::super::ordered_decision;
    lifecycle::run(|| {
        let run_birth = |evicted| {
            let (world, provider) = fresh_world();
            ordered_decision::SEED_ENTRY.set(true);
            author(&world, "catalog", "parent-row", 0x3100_0020);
            ordered_decision::SEED_ENTRY.set(false);
            let scope = authentication::request_scope();
            let principal = principal(&world, &scope);
            let request = world
                .runtime()
                .request(&principal, &scope)
                .on_branch(world.current_world());
            let find_parent = || ordered_decision::Find {
                directory: "catalog".into(),
                prefix: String::new(),
            };
            let index = request
                .query(find_parent())
                .open_index::<ordered_decision::Index>()
                .unwrap();
            if evicted {
                request.query(find_parent()).evict_index(&index).unwrap();
            }
            ORDERED_SUCCESSOR.set(Some(("successor", "child-row")));
            THROUGH_COMPUTATION.set(true);
            let result = run(
                &world,
                &provider,
                ProducerProbe::OrderedOwn,
                None,
                &scope,
                0x3100_0021,
            );
            assert_eq!(
                result.observed,
                vec![Ok(vec!["child-row".to_owned(), "parent-row".to_owned()])],
                "{:?}",
                result.refusal
            );
            assert!(result.performed);
            index.close();
            result.observed
        };
        assert_eq!(run_birth(false), run_birth(true));
    });
}
