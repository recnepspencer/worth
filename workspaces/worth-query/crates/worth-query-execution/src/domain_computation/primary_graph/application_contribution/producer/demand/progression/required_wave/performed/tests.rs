//! The readiness owner never grants a second attempt to rescue optional capacity.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::execution::commit_receipt;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryProducerApplicability, WorthQueryProducerLifecyclePosture,
};
use crate::domain_computation::primary_graph::application_output_demand::{
    ReadyCompletion, WorthQueryAcceptedOutputAuthority, WorthQueryCompletedOutputDemand,
    WorthQueryOutputReadinessDeliveryEvidence,
};
use crate::domain_computation::primary_graph::{
    output_lineage::own_write_fixture::with_committed_own_write, WorthQueryApplicationCommitOutcome,
};
use crate::domain_computation::{
    execution_runtime::WorthQueryInvalidationResourceDenial,
    primary_graph::tests::fixture::AuthorizationWorld,
};
use worth_relational::facade::identity::{EntityId, PartitionId};

#[test]
fn refused_registration_capacity_does_not_select_a_member_twice_in_one_advance() {
    with_committed_own_write(|world, receipt, _, resources| {
        assert_refused_registration_remains_performed(world, receipt, resources);
    });
}

fn assert_refused_registration_remains_performed(
    world: &AuthorizationWorld,
    receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    resources: crate::domain_computation::execution_runtime::WorthQueryInvalidationResources,
) {
    let runtime = &world.application;
    let mut admission = runtime.demand_request_admission();
    let (shared, positioned) = super::super::selection::select_required_basis(
        runtime,
        runtime.current_world(),
        &mut admission,
    )
    .unwrap();
    let input = SelectedDecisionInput {
        shared: &shared,
        positioned: &positioned,
        runtime,
    };
    let source = WorthQueryObservedSourceEpoch::new(
        [1; 32],
        [2; 32],
        EntityId::new(PartitionId::main(), 1, 1),
        shared
            .selected()
            .product()
            .observation()
            .lifecycle_incarnation(),
        0,
        [3; 32],
    );
    let mut performed = PerformedMembers::start();
    let key = WorthQueryOutputDemandKey::new(
        "unit-output-family",
        "test producer".to_owned(),
        WorthQueryProducerApplicability::new("test", WorthQueryProducerLifecyclePosture::Initial),
        source.clone(),
    );
    let mut attempts = 0;
    // Selection alone has no authoritative effect. Abandoning its permission
    // must still leave this member eligible for its first publication.
    let _abandoned = performed
        .fresh(
            &key,
            source.clone(),
            &DecisionInput::Selected(&input),
            &mut admission,
        )
        .unwrap_or_else(|_| panic!("the first selection is admitted"))
        .expect("an unperformed member is Fresh");
    if let Some(permission) = performed
        .fresh(
            &key,
            source.clone(),
            &DecisionInput::Selected(&input),
            &mut admission,
        )
        .unwrap_or_else(|_| panic!("Fresh readiness comparison is admitted"))
    {
        attempts += 1;
        // The effect has completed before optional registration. The wave's
        // mark changes only this state; selection alone did not perform it.
        let publication = commit_receipt(
            "published fixture",
            WorthQueryApplicationCommitOutcome::Committed(receipt),
        )
        .unwrap_or_else(|_| panic!("the genuine commit retains its publication fact"));
        // The same handoff seals production publications before checkpointing.
        let permission = performed
            .attach_successor(permission, &key, &mut admission)
            .unwrap();
        performed.publication(permission, publication, |member| member.performed());
        // Optional registration draws from this actual shared index ledger.
        // Exhaust it after selection, then return its exact capacity refusal.
        let maximum = resources.installation().maximum_retained_bytes;
        let held = resources
            .reserve_retained_capacity(maximum - resources.retained_capacity_bytes())
            .unwrap();
        assert_eq!(
            resources.reserve_retained_capacity(1).err(),
            Some(
                WorthQueryInvalidationResourceDenial::RetentionCapacityExhausted {
                    requested: 1,
                    retained: maximum,
                    maximum,
                }
            )
        );
        drop(held);
    }
    if performed
        .fresh(
            &key,
            source.clone(),
            &DecisionInput::Selected(&input),
            &mut admission,
        )
        .unwrap_or_else(|_| panic!("Fresh readiness comparison is admitted"))
        .is_some()
    {
        attempts += 1;
    }
    assert_eq!(
        attempts, 1,
        "returning optional capacity cannot grant a second attempt"
    );
    // A later advancement has a new record and may try this member again.
    assert!(PerformedMembers::start()
        .fresh(
            &key,
            source,
            &DecisionInput::Selected(&input),
            &mut admission
        )
        .unwrap_or_else(|_| panic!("later advancement readiness is admitted"))
        .is_some());
}

#[test]
fn a_ready_and_its_successor_share_one_performed_member() {
    for (producer, grant_from_predecessor) in ["test producer", "replacement producer"]
        .into_iter()
        .flat_map(|producer| [false, true].map(|predecessor| (producer, predecessor)))
    {
        with_committed_own_write(|world, receipt, _, _| {
            let runtime = &world.application;
            let mut admission = runtime.demand_request_admission();
            let (shared, positioned) = super::super::selection::select_required_basis(
                runtime,
                runtime.current_world(),
                &mut admission,
            )
            .unwrap();
            let input = SelectedDecisionInput {
                shared: &shared,
                positioned: &positioned,
                runtime,
            };
            let source_with_identity = |identity| {
                WorthQueryObservedSourceEpoch::new(
                    [1; 32],
                    [2; 32],
                    EntityId::new(PartitionId::main(), 1, 1),
                    shared
                        .selected()
                        .product()
                        .observation()
                        .lifecycle_incarnation(),
                    0,
                    identity,
                )
            };
            let source = source_with_identity([3; 32]);
            let key = |producer: &str, lifecycle| {
                WorthQueryOutputDemandKey::new(
                    "unit-output-family",
                    producer.to_owned(),
                    WorthQueryProducerApplicability::new("test", lifecycle),
                    source.clone(),
                )
            };
            let selected = key("test producer", WorthQueryProducerLifecyclePosture::Initial);
            let successor = key(producer, WorthQueryProducerLifecyclePosture::Preserve);
            let mut performed = PerformedMembers::start();
            let permission = performed
                .fresh(
                    &selected,
                    source.clone(),
                    &DecisionInput::Selected(&input),
                    &mut admission,
                )
                .unwrap_or_else(|_| panic!("selected member admitted"))
                .unwrap();
            let completion = ReadyCompletion::for_test(WorthQueryCompletedOutputDemand {
                authority: WorthQueryAcceptedOutputAuthority::Committed(receipt.clone()),
                readiness: WorthQueryOutputReadinessDeliveryEvidence::for_test(),
                resources: None,
            });
            let publication = commit_receipt(
                "published successor",
                WorthQueryApplicationCommitOutcome::Committed(receipt),
            )
            .unwrap_or_else(|_| panic!("genuine successor publication"));
            let permission = performed
                .attach_successor(permission, &successor, &mut admission)
                .unwrap();
            performed.publication(permission, publication, |member| member.performed());
            for lookup in [&successor, &selected] {
                assert!(
                    performed
                        .fresh(
                            lookup,
                            source.clone(),
                            &DecisionInput::Selected(&input),
                            &mut admission,
                        )
                        .unwrap_or_else(|_| panic!("readiness comparison admitted"))
                        .is_none(),
                    "a published member must be found through either key: {producer}"
                );
            }
            assert_eq!(performed.entries.len(), 1);
            // The explicit successor join does not merge another binding's
            // independent Initial demand on this same source occurrence.
            let independent = key(
                "independent producer",
                WorthQueryProducerLifecyclePosture::Initial,
            );
            assert!(performed
                .fresh(
                    &independent,
                    source.clone(),
                    &DecisionInput::Selected(&input),
                    &mut admission,
                )
                .unwrap_or_else(|_| panic!("distinct member admitted"))
                .is_some());
            assert_eq!(performed.entries.len(), 2);

            // Capture uses the successor key, just as the Ready rejoin does.
            performed
                .capture(
                    &successor,
                    &Err(captured_input::FullVerificationReason::CheckpointRestore),
                    &mut admission,
                )
                .unwrap();
            assert!(
                matches!(
                    performed
                        .entries
                        .iter()
                        .find(|entry| entry.names(&successor))
                        .unwrap()
                        .input,
                    CapturedDecisionInput::Withheld
                ),
                "the first capture must retain the withheld comparison"
            );
            let later_candidate = runtime
                .primary_provider
                .graph
                .output_lineage
                .lock()
                .unwrap()
                .resolve_required_settlement(
                    runtime.runtime.authority_identity().as_u64(),
                    &runtime.installed_schema.binding_identity(),
                    &completion,
                    &mut admission,
                )
                .unwrap();
            assert!(
                matches!(later_candidate, Ok(Some(_))),
                "the later capture has an exact accepted candidate"
            );
            performed
                .capture(&selected, &later_candidate, &mut admission)
                .unwrap();
            assert!(
                matches!(
                    performed
                        .entries
                        .iter()
                        .find(|entry| entry.names(&selected))
                        .unwrap()
                        .input,
                    CapturedDecisionInput::Withheld
                ),
                "a later accepted capture must not overwrite the withheld comparison"
            );
            for lookup in [&successor, &selected] {
                assert!(
                    performed
                        .fresh(
                            lookup,
                            source.clone(),
                            &DecisionInput::Selected(&input),
                            &mut admission,
                        )
                        .unwrap_or_else(|_| panic!("withheld comparison admitted"))
                        .is_none(),
                    "a published member with withheld comparison is refused through either key"
                );
            }
            let mut changed_identity = source.runtime_idempotency_identity();
            changed_identity[0] ^= 1;
            let changed_source = source_with_identity(changed_identity);
            assert!(source.same_occurrence(&changed_source));
            assert!(!source.same_semantic_source(&changed_source));
            let lookup = if grant_from_predecessor {
                &selected
            } else {
                &successor
            };
            assert!(performed
                .fresh(
                    lookup,
                    changed_source,
                    &DecisionInput::Selected(&input),
                    &mut admission,
                )
                .unwrap_or_else(|_| panic!("changed semantic source admitted"))
                .is_some());
        });
    }
}
