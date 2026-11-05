//! Retained index reconstruction consumes actual existing basis custody.
use super::*;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey};
use worth_relational::facade::{identity::KindId, indexes::*};

#[test]
fn checkpoint_image_retained_commit_reconstruction_uses_its_own_root_without_canonical_body_reads()
{
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let (runtime, medium) = initialized(&request);
    let locator = worth_relational::facade::transactions::planned_single_field_locator(
        AspectKey::new("name").unwrap(),
        FieldKey::new("name").unwrap(),
    );
    let definition = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "image-retained-name".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: locator.clone(),
            },
        })
        .unwrap();
    let first = create_entity_outcome(&request, &runtime, "historical");
    let receipt = first.commit.clone();
    let expected = first.semantic_result().created_entities()[0].assigned;
    let slots = first.changed_records.len() + 1;
    let before = current(&request, &runtime);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let lease = runtime.retain_component_basis(&basis).unwrap();
    drop(basis);
    let second = create_entity_outcome(&request, &runtime, "current");
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
    let image = capture_external(&runtime, &medium, &request);
    medium.file_checkpoint(image.checkpoint().clone());
    let reads = medium.canonical_member_reads();
    let input = home_image_input(&medium, &image, &request).unwrap();
    runtime.release_component_basis(lease).unwrap();
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
    let (_, authority) = recovered
        .durability_recovery()
        .recover_with_authority(&request, plan)
        .unwrap();
    let basis = authority
        .admit_recorded_component(&request, &recovered, &before)
        .unwrap()
        .into_basis();
    let snapshot = recovered
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    assert_ne!(
        recovered
            .observe_branch(&recovered.main_branch_identity())
            .unwrap()
            .1
            .observation()
            .commit_id(),
        Some(receipt.commit_id)
    );
    let budget = DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(100_000),
        maximum_cold_record_slots: Some(slots),
        maximum_derived_rows: Some(1),
    };
    let mut foreign_branch = receipt.branch_id.clone();
    foreign_branch.0 = "not-the-author".into();
    let denied = recovered
        .index_authority()
        .reconstruct_for_commit(
            &request,
            DerivedIndexBuildRequest {
                source_commit_id: receipt.commit_id,
                branch_id: foreign_branch,
                index_ids: vec![definition.index_id],
            },
            budget,
        )
        .unwrap_err();
    assert_eq!(
        denied.kind,
        DerivedIndexMaintenanceDenialKind::CommitMismatch
    );
    let cold_denied = recovered
        .index_authority()
        .reconstruct_for_commit(
            &request,
            DerivedIndexBuildRequest {
                source_commit_id: receipt.commit_id,
                branch_id: receipt.branch_id.clone(),
                index_ids: vec![definition.index_id],
            },
            DerivedIndexMaintenanceBudget {
                maximum_cold_record_slots: Some(0),
                ..budget
            },
        )
        .unwrap_err();
    assert_eq!(
        cold_denied.kind,
        DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
    );
    let outcome = recovered
        .index_authority()
        .reconstruct_for_commit(
            &request,
            DerivedIndexBuildRequest {
                source_commit_id: receipt.commit_id,
                branch_id: receipt.branch_id,
                index_ids: vec![definition.index_id],
            },
            budget,
        )
        .unwrap();
    assert_eq!(outcome.generations.len(), 1);
    assert_eq!(outcome.work.cold_record_slots, slots);
    for (key, ids) in [("historical", vec![expected]), ("current", vec![])] {
        let selected = recovered
            .index_access()
            .execute_bounded_entity_field_lookup(
                BoundedEntityFieldLookupRequest::new(
                    snapshot.clone(),
                    definition.index_id,
                    KindId(1),
                    locator.clone(),
                    AspectValue::String(key.into()),
                    1,
                )
                .unwrap(),
                BoundedIndexParityMode::Production,
                &request,
            )
            .unwrap();
        assert_eq!(selected.candidate_entity_ids(), ids.as_slice());
    }
    recovered.snapshots().release_snapshot(&snapshot).unwrap();
    assert_eq!(
        medium.canonical_member_reads(),
        reads,
        "retained image reconstruction must not demand covered canonical bodies"
    );
}

#[test]
fn sparse_retained_registry_refuses_unpaid_physical_scan_before_cold_reconstruction() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let (runtime, _) = initialized(&request);
    let locator = worth_relational::facade::transactions::planned_single_field_locator(
        AspectKey::new("name").unwrap(),
        FieldKey::new("name").unwrap(),
    );
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "sparse-retained-name".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: locator,
            },
        })
        .unwrap();
    let first = create_entity_outcome(&request, &runtime, "survivor");
    let receipt = first.commit.clone();
    let (_, held) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut admitted = Vec::new();
    for ordinal in 0..64 {
        admitted.push(create_entity_outcome(
            &request,
            &runtime,
            &format!("retained-{ordinal}"),
        ));
    }
    assert!(
        runtime
            .branch_basis_cost_counters()
            .retained_basis_registry_entries
            >= 65
    );
    for outcome in admitted {
        release_test_commit_snapshot(&runtime, &outcome);
    }
    release_test_commit_snapshot(&runtime, &first);
    assert_eq!(
        runtime
            .branch_basis_cost_counters()
            .retained_basis_registry_entries,
        1
    );
    let published_before = runtime
        .index_access()
        .read_latest_generation(&request, index.index_id, &receipt.branch_id)
        .unwrap()
        .map(|generation| generation.generation_id);
    let refused = runtime
        .index_authority()
        .reconstruct_for_commit(
            &request,
            DerivedIndexBuildRequest {
                source_commit_id: receipt.commit_id,
                branch_id: receipt.branch_id.clone(),
                index_ids: vec![index.index_id],
            },
            DerivedIndexMaintenanceBudget {
                // Logical live population is one. The previously populated hash table
                // still has at least65 buckets; this cannot pay its physical traversal.
                maximum_work_units: Some(32),
                maximum_cold_record_slots: Some(0),
                maximum_derived_rows: Some(0),
            },
        )
        .unwrap_err();
    assert_eq!(
        refused.kind,
        DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded
    );
    let published_after = runtime
        .index_access()
        .read_latest_generation(&request, index.index_id, &receipt.branch_id)
        .unwrap()
        .map(|generation| generation.generation_id);
    assert_eq!(
        published_after, published_before,
        "refused scan publishes no generation"
    );
    drop(held);
}
