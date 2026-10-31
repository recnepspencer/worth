use super::*;
use worth_relational::facade::durability::RelationalCheckpointArtifactPolicy;
pub fn capture(
    execution: &ExecutionRequest<'_, '_>,
    source: &RelationalRuntime,
) -> worth_relational::facade::durability::RelationalNativeCheckpoint {
    source
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap()
}

pub struct Original {
    pub source: RelationalRuntime,
    pub medium: Arc<TestDurableMedium>,
    pub before: Recorded,
    pub after: Recorded,
    pub association: Association,
}
pub fn original(execution: &ExecutionRequest<'_, '_>, empty: bool) -> Original {
    original_with_retained_successor(execution, empty, true)
}
pub fn original_with_retained_successor(
    execution: &ExecutionRequest<'_, '_>,
    empty: bool,
    retain_after: bool,
) -> Original {
    let medium = TestDurableMedium::new();
    let source = durable_runtime(&medium);
    if !empty {
        create_entity(execution, &source, "original-predecessor");
    }
    let before = current(execution, &source);
    let (_, basis) = source
        .observe_branch(&source.main_branch_identity())
        .unwrap();
    let before_lease = source.retain_component_basis(&basis).unwrap();
    drop(basis);
    let association = Association::from_bytes(b"checkpoint-original-external-custody").unwrap();
    let performed = perform(execution, &source, &association, "original-successor");
    let after_basis = performed.next_basis().clone();
    let result = source.settle_performed_publication(performed).unwrap();
    release_test_commit_snapshot(&source, &result);
    let after = source
        .owner_component_services()
        .basis_port()
        .export_recorded_retained_component(execution, &after_basis)
        .unwrap();
    let after_lease = retain_after.then(|| source.retain_component_basis(&after_basis).unwrap());
    drop(after_basis);
    create_entity(execution, &source, "unrelated-body");
    create_entity(execution, &source, "checkpoint-current-head");
    let checkpoint = capture(execution, &source);
    // The original unique leases may disappear after capture without weakening
    // the exact header/member custody already captured from those observations.
    source.release_component_basis(before_lease).unwrap();
    if let Some(lease) = after_lease {
        source.release_component_basis(lease).unwrap();
    }
    medium.file_checkpoint(checkpoint);
    Original {
        source,
        medium,
        before,
        after,
        association,
    }
}
