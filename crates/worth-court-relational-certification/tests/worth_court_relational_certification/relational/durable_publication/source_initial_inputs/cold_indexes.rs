//! The second Complete carries actual covered and replayed-tail index owners.
use super::*;
use worth_relational::facade::indexes::*;
#[test]
fn cold_complete_recovery_selects_live_tail_and_builds_its_actual_index() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&policy);
    let (runtime, medium) = initialized(&request);
    let definition = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "cold-original-name".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: worth_relational::facade::transactions::planned_single_field_locator(
                    worth_foundational::facade::AspectKey::new("name").unwrap(),
                    worth_foundational::facade::FieldKey::new("name").unwrap(),
                ),
            },
        })
        .unwrap();
    let first = create_entity_outcome(&request, &runtime, "covered-index-value");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let built = runtime.index_authority().build_for_basis(
        &request,
        DerivedIndexBuildRequest {
            source_commit_id: first.commit.commit_id,
            branch_id: first.commit.branch_id.clone(),
            index_ids: vec![definition.index_id],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    assert!(!built.generations.is_empty());
    drop(basis);
    release_test_commit_snapshot(&runtime, &first);
    let image = capture_external(&runtime, &medium, &request);
    let members = selected_members(&image, &request);
    medium.file_checkpoint(image.checkpoint().clone());
    let later = create_entity_outcome(&request, &runtime, "live-tail-index-value");
    let tail_commit = later.commit.commit_id.0;
    release_test_commit_snapshot(&runtime, &later);
    let input = home_image_input(&medium, &image, &request).unwrap();
    drop((image, runtime));
    let mut recovered = receiver(&medium);
    let plan = recovered
        .durability()
        .recovery_plan_from_source(
            &request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    recovered
        .durability_recovery()
        .recover_with_authority(&request, plan)
        .unwrap();
    let (descriptor, basis) = recovered
        .observe_branch(&recovered.main_branch_identity())
        .unwrap();
    let worth_foundational::FoundationalBranchTarget::Basis(target) =
        descriptor.reference().target()
    else {
        panic!("the replayed tail selects a real commit");
    };
    assert_eq!(target.selected_commit_id(), tail_commit);
    let built = recovered.index_authority().build_for_basis(
        &request,
        DerivedIndexBuildRequest {
            source_commit_id: later.commit.commit_id,
            branch_id: descriptor.branch_id().clone(),
            index_ids: vec![definition.index_id],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    assert!(built.basis_denial.is_none(), "{built:?}");
    assert!(built.execution_denial.is_none(), "{built:?}");
    assert!(!built.generations.is_empty());
    assert!(built
        .generations
        .iter()
        .all(|generation| generation.source_commit_id.0 == tail_commit));
    drop(basis);
    let next = capture_selected_external(&recovered, &medium, &members, &request);
    assert_eq!(next.source_artifact().covered().0, 2);
}
