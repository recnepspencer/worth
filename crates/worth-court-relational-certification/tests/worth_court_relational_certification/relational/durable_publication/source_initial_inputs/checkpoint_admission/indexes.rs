//! Index recovery uses actual image roots, including an authenticated before root.
use super::*;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey};
use worth_relational::facade::{identity::KindId, indexes::*};

#[test]
fn checkpoint_image_roots_build_cold_and_incremental_indexes_without_canonical_body_reads() {
    for incremental in [false, true] {
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
                name: "image-owned-name".into(),
                branch_scoped: true,
                kind: DerivedIndexKind::EntityField {
                    field_locator: locator.clone(),
                },
            })
            .unwrap();
        let first = create_entity_outcome(&request, &runtime, "before");
        let before = current(&request, &runtime);
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let lease = runtime.retain_component_basis(&basis).unwrap();
        if incremental {
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
            assert_eq!(built.generations.len(), 1);
        }
        drop(basis);
        let (receipt, expected, key, slots) = if incremental {
            let next = create_entity_outcome(&request, &runtime, "after");
            let entity = next.semantic_result().created_entities()[0].assigned;
            let receipt = next.commit.clone();
            release_test_commit_snapshot(&runtime, &next);
            (receipt, entity, "after", 0)
        } else {
            (
                first.commit.clone(),
                first.semantic_result().created_entities()[0].assigned,
                "before",
                first.changed_records.len() + 1,
            )
        };
        release_test_commit_snapshot(&runtime, &first);
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
        let prior = incremental.then(|| {
            let basis = authority
                .admit_recorded_component(&request, &recovered, &before)
                .unwrap()
                .into_basis();
            recovered
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .unwrap()
        });
        if let Some(prior) = &prior {
            let view = recovered.read_truth().project_snapshot(prior).unwrap();
            let pinned = recovered
                .read_truth()
                .pin_view_read_capability(&view, &[definition.index_id], &request)
                .unwrap();
            drop(pinned);
        }
        let (_, basis) = recovered
            .observe_branch(&recovered.main_branch_identity())
            .unwrap();
        let outcome = recovered
            .index_authority()
            .refresh_for_basis(
                DerivedIndexBuildRequest {
                    source_commit_id: receipt.commit_id,
                    branch_id: receipt.branch_id.clone(),
                    index_ids: vec![definition.index_id],
                },
                &basis,
                prior.as_ref(),
                DerivedIndexMaintenanceBudget {
                    maximum_work_units: None,
                    // One real entity slot and its partition; incremental recovery has
                    // no population-reconstruction allowance.
                    maximum_cold_record_slots: Some(slots),
                    maximum_derived_rows: None,
                },
                &request,
            )
            .unwrap_or_else(|refused| panic!("incremental={incremental}: {refused:?}"));
        assert_eq!(outcome.generations.len(), 1);
        if incremental {
            assert_eq!(outcome.work.cold_record_slots, 0);
            assert!(outcome.work.patch_records > 0);
        } else {
            assert_eq!(outcome.work.cold_record_slots, slots);
        }
        let snapshot = recovered
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        if incremental {
            let refused = recovered
                .index_authority()
                .refresh_for_basis(
                    DerivedIndexBuildRequest {
                        source_commit_id: receipt.commit_id,
                        branch_id: receipt.branch_id.clone(),
                        index_ids: vec![definition.index_id],
                    },
                    &basis,
                    Some(&snapshot),
                    DerivedIndexMaintenanceBudget {
                        maximum_work_units: None,
                        maximum_cold_record_slots: Some(0),
                        maximum_derived_rows: None,
                    },
                    &request,
                )
                .unwrap_err();
            assert_eq!(
                refused.kind,
                DerivedIndexMaintenanceDenialKind::BeforeRootMismatch
            );
        }
        let selected = recovered
            .index_access()
            .execute_bounded_entity_field_lookup(
                BoundedEntityFieldLookupRequest::new(
                    snapshot.clone(),
                    definition.index_id,
                    KindId(1),
                    locator,
                    AspectValue::String(key.into()),
                    1,
                )
                .unwrap(),
                BoundedIndexParityMode::Production,
                &request,
            )
            .unwrap();
        assert_eq!(selected.candidate_entity_ids(), &[expected]);
        recovered.snapshots().release_snapshot(&snapshot).unwrap();
        if let Some(prior) = prior {
            recovered.snapshots().release_snapshot(&prior).unwrap();
        }
        assert_eq!(
            medium.canonical_member_reads(),
            reads,
            "cold and incremental image index work must not open canonical bodies"
        );
    }
}
