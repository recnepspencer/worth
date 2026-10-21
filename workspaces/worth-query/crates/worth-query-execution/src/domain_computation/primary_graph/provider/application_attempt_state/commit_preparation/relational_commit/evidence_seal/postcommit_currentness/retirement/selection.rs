use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole;
use crate::domain_computation::primary_graph::{
    tests::fixture::Account, WorthQueryApplicationOutputCorrespondence,
    WorthQueryApplicationOutputPosture,
};
use crate::equality_type_id::EqualityTypeId as TypeId;
use worth_relational::facade::transactions::RecordRef;

#[test]
fn checkpoint_retirement_requires_this_commits_changed_entity_and_deleted_snapshot() {
    let world = installed_authorization_world(true);
    let (owned, _, facts) = world
        .application
        .with_host_advancement(|phase| {
            reads(
                &phase,
                &phase.borrowed_request_for_test(),
                &world,
                "account-2",
            )
        })
        .expect("the fixture read owns its installed host call");
    // This isolates owner checkpoint selector admission. Typed write/delete agreement
    // is protected by retire_role_requires_the_matching_delete_effect.
    let retirement = restored_role(owned, WorthQueryApplicationOutputPosture::Retire);
    let preservation = restored_role(owned, WorthQueryApplicationOutputPosture::Preserve);
    let changed = [RecordRef::Entity(owned)];
    world
        .application
        .with_host_advancement(|phase| {
            let borrowed = phase.borrowed_request_for_test();
            let execution = &borrowed;
            assert!(
                sealed_rebase(
                    &phase,
                    execution,
                    &world,
                    facts.clone(),
                    &retirement,
                    &changed
                )
                .is_empty(),
                "an exact changed record cannot manufacture retirement of a live entity"
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let borrowed = phase.borrowed_request_for_test();
            delete(&borrowed, &world, owned)
        })
        .expect("the fixture read owns its installed host call");
    let exact = world
        .application
        .with_host_advancement(|phase| {
            let borrowed = phase.borrowed_request_for_test();
            let execution = &borrowed;
            sealed_rebase(
                &phase,
                execution,
                &world,
                facts.clone(),
                &retirement,
                &changed,
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(exact.len(), facts.len());
    world
        .application
        .with_host_advancement(|phase| {
            assert!(selection(
                &phase,
                &phase.borrowed_request_for_test(),
                &world,
                &exact
            ))
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let borrowed = phase.borrowed_request_for_test();
            let execution = &borrowed;
            for (correspondence, changed) in [
                (&retirement, Vec::new()),
                (
                    &retirement,
                    vec![RecordRef::Entity(
                        reads(
                            &phase,
                            &phase.borrowed_request_for_test(),
                            &world,
                            "account-1",
                        )
                        .0,
                    )],
                ),
                (&preservation, vec![RecordRef::Entity(owned)]),
            ] {
                let unqualified = sealed_rebase(
                    &phase,
                    execution,
                    &world,
                    facts.clone(),
                    correspondence,
                    &changed,
                );
                assert!(!unqualified
                    .iter()
                    .any(|fact| matches!(fact, Fact::RetiredOutputEntity { .. })));
                assert!(
                    !selection(
                        &phase,
                        &phase.borrowed_request_for_test(),
                        &world,
                        &unqualified
                    ),
                    "a previous retirement and another commit's changes cannot rebase this commit"
                );
            }
        })
        .expect("the fixture read owns its installed host call")
}

fn restored_role(
    entity: EntityId,
    posture: WorthQueryApplicationOutputPosture,
) -> WorthQueryApplicationOutputCorrespondence {
    struct CheckpointBinding;
    struct CheckpointOutputs;
    WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
        TypeId::of::<CheckpointBinding>(),
        TypeId::of::<CheckpointOutputs>(),
        std::iter::empty(),
        &[WorthQueryCheckpointOutputRole {
            role: "output".into(),
            posture,
            entity_name: "Account".into(),
            entity,
        }],
        |_| Some(TypeId::of::<Account>()),
        &worth_execution::ExecutionRequest::serial(&worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        )),
    )
    .unwrap()
}

fn sealed_rebase(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    facts: Vec<Fact>,
    correspondence: &WorthQueryApplicationOutputCorrespondence,
    changed: &[RecordRef],
) -> crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload {
    let selected = world.selected_product(&(phase_custody).test_host_request());
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(execution, |runtime| {
            exact(super::super::super::rebase_output(execution, runtime, selected.application_basis().snapshot_handle(), super::super::super::PreparedSourceFactRebase::admit(
                    facts,
                    [].into(),
                    StorageControl::new(
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                        None,
                    ),
                )
                .unwrap(), correspondence, changed, true, crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(execution.allocation_policy(), None)))
        })
}
