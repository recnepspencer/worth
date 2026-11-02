//! Actual malformed histories and portable meaning refuse at their own boundary.
use super::*;
use fixture::*;
#[test]
fn ordered_event_inverse_validates_future_history_beyond_retirement_before_pruning() {
    with_event_probe(|runtime, control, host, _| {
        let execution = ExecutionRequest::serial(host);
        let w = seed(runtime, &execution);
        let [a, b, c, d, u] = w.events;
        let [x, _, _] = w.rows;
        let born = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("permissive-event-history")
                .push(link(5, b, a, "ba"))
                .push(link(5, c, b, "cb"))
                .push(link(5, d, c, "dc"))
                .push(link(2, a, x, "ax"))
                .push(link(3, b, x, "bx"))
                .push(link(5, d, u, "second-older-history")),
        );
        let invalid = Err(DerivedRelationReadDenial::Definition(
            DerivedRelationDenial::InvalidEventChain,
        ));
        assert_eq!(probe::ask(runtime, control, &execution, x, true), invalid);
        assert_eq!(probe::ask(runtime, control, &execution, d, false), invalid);
        // Delete the genuinely malformed second predecessor. With no new
        // introduction, retirement stops membership at b and its successors.
        *control.request.lock().unwrap() = None;
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("repair-actual-chain").push(delete(relation(
                &born,
                5,
                d,
                u,
                "second-older-history",
            ))),
        );
        assert_eq!(
            probe::ask(runtime, control, &execution, x, true),
            Ok(vec![a])
        );
        *control.request.lock().unwrap() = None;
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("lawful-newer-introduction")
                .push(link(2, c, x, "cx"))
                .push(link(5, u, b, "ub-lawful-fork"))
                .push(link(2, u, x, "ux-independent-introduction")),
        );
        let mut live = vec![a, c, d, u];
        live.sort();
        assert_eq!(probe::ask(runtime, control, &execution, x, true), Ok(live));
    });
}
#[test]
fn ordered_event_reader_rejects_cycles_and_preserves_actual_empty_terminal_support() {
    with_event_probe(|runtime, control, host, _| {
        let execution = ExecutionRequest::serial(host);
        let w = seed(runtime, &execution);
        let [a, b, c, _, u] = w.events;
        let [x, _, _] = w.rows;
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("cyclic-event-history")
                .push(link(5, a, b, "ab"))
                .push(link(5, b, c, "bc"))
                .push(link(5, c, a, "ca"))
                .push(link(2, a, x, "ax")),
        );
        for (anchor, reverse) in [(a, false), (x, true)] {
            assert_eq!(
                probe::ask(runtime, control, &execution, anchor, reverse),
                Err(DerivedRelationReadDenial::Definition(
                    DerivedRelationDenial::InvalidEventChain
                ))
            );
        }
        assert_eq!(
            probe::ask(runtime, control, &execution, u, false),
            Ok(vec![])
        );
    });
}
#[test]
fn ordered_event_registration_rejects_nonphysical_or_discontinuous_decoded_inputs() {
    let valid = registry::parts();
    let roundtrip: DerivedRelationDefinitionParts =
        serde_json::from_str(&serde_json::to_string(&valid).unwrap()).unwrap();
    assert!(registry::base()
        .register_derived_relations(vec![roundtrip])
        .is_ok());
    for mode in 0..3 {
        let mut parts = valid.clone();
        match mode {
            0 => {
                parts.nodes[3] = DerivedRelationNode::OrderedEvents {
                    predecessor: 3,
                    introduced: 1,
                    retired: 2,
                }
            }
            1 => {
                parts.nodes[0] = DerivedRelationNode::Stored {
                    relation: KindId(2),
                    from: KindId(1),
                    to: KindId(4),
                }
            }
            _ => parts.nodes[2] = DerivedRelationNode::Reverse { input: 1 },
        }
        let decoded = serde_json::from_str(&serde_json::to_string(&parts).unwrap()).unwrap();
        assert!(matches!(
            registry::base().register_derived_relations(vec![decoded]),
            Err(DerivedRelationDenial::InvalidExpression
                | DerivedRelationDenial::DiscontinuousEndpoints)
        ));
    }
}
#[test]
fn ordered_event_roles_change_schema_identity_and_legacy_leaf_wire_stays_literal() {
    let parts = registry::parts();
    let original = registry::base()
        .register_derived_relations(vec![parts.clone()])
        .unwrap();
    let mut reversed = parts.clone();
    reversed.nodes[3] = DerivedRelationNode::OrderedEvents {
        predecessor: 0,
        introduced: 2,
        retired: 1,
    };
    let reversed = registry::base()
        .register_derived_relations(vec![reversed])
        .unwrap();
    assert_ne!(
        original.authority_digest_bytes(),
        reversed.authority_digest_bytes(),
        "only ordered event roles changed; all physical leaf definitions are identical"
    );
    assert_eq!(
        serde_json::to_string(&parts.nodes[1]).unwrap(),
        r#"{"Stored":{"relation":2,"from":1,"to":4}}"#
    );
}

#[test]
fn ordered_event_original_cancellation_survives_completed_read_and_fresh_retry() {
    with_event_probe(|runtime, control, host, cancel| {
        let execution = ExecutionRequest::serial(host);
        let w = seed(runtime, &execution);
        let [a, b, _, _, _] = w.events;
        let [x, _, _] = w.rows;
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("cancel-event-history")
                .push(link(5, b, a, "ba"))
                .push(link(2, a, x, "ax")),
        );
        *control.request.lock().unwrap() = Some((b, false));
        *control.cancel_after_read.lock().unwrap() = Some(cancel.clone());
        let before = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0;
        let error = try_commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("stop-after-read").push(MutationIntent::Entity(
                EntityMutationIntent::Revalidate(RevalidateEntityIntent { entity_id: b }),
            )),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            TransactionCommitError::Execution {
                denial: CommitExecutionDenial {
                    kind: CommitExecutionDenialKind::Cause(
                        RelationalExecutionDenialCause::Cancelled
                    ),
                    ..
                },
                ..
            }
        ));
        let results = control.results.lock().unwrap();
        assert_eq!(
            results[0],
            Ok(vec![x]),
            "the actual ordered read completed before cancellation"
        );
        assert!(matches!(
            results.last().unwrap(),
            Err(DerivedRelationReadDenial::Structural(
                StructuralWalkDenialKind::Interrupted(CustomInvariantInterruption::Cancelled)
            ))
        ));
        drop(results);
        assert_eq!(
            runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .0,
            before
        );
        with_fresh_event_request(|fresh| {
            assert_eq!(
                probe::ask(runtime, control, &ExecutionRequest::serial(fresh), b, false),
                Ok(vec![x])
            )
        });
    });
}

#[test]
fn ordered_event_retired_row_generation_refuses_instead_of_becoming_empty_membership() {
    with_event_probe(|runtime, control, host, _| {
        let execution = ExecutionRequest::serial(host);
        let w = seed(runtime, &execution);
        let a = w.events[0];
        let x = w.rows[0];
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("row-generation-introduction").push(link(2, a, x, "ax")),
        );
        assert_eq!(
            probe::ask(runtime, control, &execution, x, true),
            Ok(vec![a])
        );
        *control.request.lock().unwrap() = None;
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("retire-referenced-row").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: x }),
            )),
        );
        assert_eq!(
            probe::ask_via(runtime, control, &execution, x, true, a),
            Err(DerivedRelationReadDenial::Structural(
                StructuralWalkDenialKind::Read(StructuralReadError::RecordUnavailable)
            ))
        );
    });
}

#[test]
fn ordered_event_complete_walk_obeys_the_original_declared_work_bound() {
    with_event_probe_limit(
        std::num::NonZeroU64::new(32_768),
        |runtime, control, host, _| {
            let execution = ExecutionRequest::serial(host);
            let w = seed(runtime, &execution);
            let a = w.events[0];
            let x = w.rows[0];
            let mut birth = WorkerIntentBatch::new("finite-funded-chain");
            for i in 0..128 {
                birth = birth.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw(format!("finite-{i}")),
                    fields: Default::default(),
                })));
            }
            let born = commit(runtime, &execution, birth);
            let mut previous = a;
            let mut links =
                WorkerIntentBatch::new("finite-funded-predecessors").push(link(2, a, x, "ax"));
            for i in 0..128 {
                let at = born
                    .created_entity(&created(1, &format!("finite-{i}")))
                    .unwrap();
                links = links.push(link(5, at, previous, &format!("previous-{i}")));
                previous = at;
            }
            commit(runtime, &execution, links);
            assert_eq!(
                probe::ask(runtime, control, &execution, a, false),
                Ok(vec![x]),
                "same declared bound admits a real short read"
            );
            *control.request.lock().unwrap() = Some((previous, false));
            control.results.lock().unwrap().clear();
            let head = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .0;
            let refused = try_commit(
                runtime,
                &execution,
                WorkerIntentBatch::new("bounded-complete-walk").push(MutationIntent::Entity(
                    EntityMutationIntent::Revalidate(RevalidateEntityIntent {
                        entity_id: previous,
                    }),
                )),
            );
            assert!(
                refused.is_err(),
                "the original reader work stop remains latched"
            );
            let results = control.results.lock().unwrap();
            assert!(
                !results.is_empty(),
                "actual event reader reached the bounded traversal"
            );
            assert!(results.iter().all(|r| matches!(
                r,
                Err(DerivedRelationReadDenial::Structural(
                    StructuralWalkDenialKind::Read(StructuralReadError::WorkBudgetExceeded)
                ))
            )));
            assert_eq!(
                runtime
                    .observe_branch(&runtime.main_branch_identity())
                    .unwrap()
                    .0,
                head
            );
        },
    );
}
