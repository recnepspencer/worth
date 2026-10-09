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
    let world = installed_authorization_world(true);
    let (owned, _, facts) = reads(&world, "account-2");
    // This isolates owner checkpoint selector admission. Typed write/delete agreement
    // is protected by retire_role_requires_the_matching_delete_effect.
    let retirement = restored_role(owned, WorthQueryApplicationOutputPosture::Retire);
    let preservation = restored_role(owned, WorthQueryApplicationOutputPosture::Preserve);
    let changed = [RecordRef::Entity(owned)];
    assert!(
        sealed_rebase(&world, facts.clone(), &retirement, &changed).is_empty(),
        "an exact changed record cannot manufacture retirement of a live entity"
    );
    delete(&world, owned);
    let exact = sealed_rebase(&world, facts.clone(), &retirement, &changed);
    assert_eq!(exact.len(), facts.len());
    assert!(selection(&world, &exact));
    for (correspondence, changed) in [
        (&retirement, Vec::new()),
        (
            &retirement,
            vec![RecordRef::Entity(reads(&world, "account-1").0)],
        ),
        (&preservation, vec![RecordRef::Entity(owned)]),
    ] {
        let unqualified = sealed_rebase(&world, facts.clone(), correspondence, &changed);
        assert!(!unqualified
            .iter()
            .any(|fact| matches!(fact, Fact::RetiredOutputEntity { .. })));
        assert!(
            !selection(&world, &unqualified),
            "a previous retirement and another commit's changes cannot rebase this commit"
        );
    }
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
    world: &AuthorizationWorld,
    facts: Vec<Fact>,
    correspondence: &WorthQueryApplicationOutputCorrespondence,
    changed: &[RecordRef],
) -> std::sync::Arc<[Fact]> {
    let selected = world.selected_product();
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(|runtime| {
            exact(super::super::super::rebase_output(
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
