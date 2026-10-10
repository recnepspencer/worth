use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole;
use crate::domain_computation::primary_graph::{
    tests::fixture::Account, WorthQueryApplicationOutputCorrespondence,
    WorthQueryApplicationOutputPosture,
};
use std::any::TypeId;
use worth_relational::facade::transactions::RecordRef;

#[test]
fn checkpoint_retirement_requires_this_commits_changed_entity_and_deleted_snapshot() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let (owned, _, facts) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            reads(&phase, execution, &world, "account-2")
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
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
            delete(&host_custody.borrowed_request_for_test(), &world, owned)
        })
        .expect("the fixture read owns its installed host call");
    let exact = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
            let host_custody = phase.test_host_request();
            assert!(selection(&phase, execution, &world, &exact))
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            for (correspondence, changed) in [
                (&retirement, Vec::new()),
                (
                    &retirement,
                    vec![RecordRef::Entity(
                        reads(&phase, execution, &world, "account-1").0,
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
                    !selection(&phase, execution, &world, &unqualified),
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
        BTreeSet::new(),
        vec![WorthQueryCheckpointOutputRole {
            role: "output".into(),
            posture,
            entity_name: "Account".into(),
            entity,
        }],
        |_| Some(TypeId::of::<Account>()),
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
) -> std::sync::Arc<[Fact]> {
    let selected = world.selected_product(&(phase_custody).test_host_request());
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(execution, |runtime| {
            exact(super::super::super::rebase_output(
                execution,
                runtime,
                selected.application_basis().snapshot_handle(),
                super::super::super::PreparedSourceFactRebase::admit(
                    facts,
                    [].into(),
                    StorageControl::new(
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                        None,
                    ),
                )
                .unwrap(),
                correspondence,
                changed,
                true,
                64,
                &mut admission(),
            ))
        })
}
