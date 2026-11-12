//! Index recovery uses actual image roots, including an authenticated before root.
use super::*;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey};
use worth_relational::facade::{identity::KindId, indexes::*};
#[test]
fn checkpoint_image_roots_restore_index_lookup_and_refuse_foreign_before_roots() {
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
        let (receipt, expected, key) = if incremental {
            let next = create_entity_outcome(&request, &runtime, "after");
            let entity = next.semantic_result().created_entities()[0].assigned;
            let receipt = next.commit.clone();
            release_test_commit_snapshot(&runtime, &next);
            (receipt, entity, "after")
        } else {
            (
                first.commit.clone(),
                first.semantic_result().created_entities()[0].assigned,
                "before",
            )
        };
        release_test_commit_snapshot(&runtime, &first);
        let image = capture_external(&runtime, &medium, &request);
        medium.file_checkpoint(image.checkpoint().clone());
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
        let build = DerivedIndexBuildRequest {
            source_commit_id: receipt.commit_id,
            branch_id: receipt.branch_id.clone(),
            index_ids: vec![definition.index_id],
        };
        if incremental {
            let refused = recovered
                .index_authority()
                .refresh_for_basis(
                    build.clone(),
                    &basis,
                    prior.as_ref(),
                    DerivedIndexMaintenanceBudget {
                        maximum_work_units: None,
                        maximum_cold_record_slots: None,
                        maximum_derived_rows: None,
                    },
                    &request,
                )
                .unwrap_err();
            assert_eq!(
                refused.kind,
                DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
            );
        }
        let reconstructed = recovered
            .index_authority()
            .reconstruct_for_basis(
                &request,
                build,
                &basis,
                DerivedIndexMaintenanceBudget {
                    maximum_work_units: None,
                    maximum_cold_record_slots: None,
                    maximum_derived_rows: None,
                },
            )
            .unwrap();
        assert_eq!(reconstructed.generations.len(), 1);
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
        if incremental {
            // The recovered image is a real predecessor, but has no transient delta.
            // A new publication supplies that delta for ordinary maintenance.
            drop(selected);
            drop(basis);
            let next = create_entity_outcome(&request, &recovered, "continued");
            let expected_next = next.semantic_result().created_entities()[0].assigned;
            let (_, next_basis) = recovered
                .observe_branch(&recovered.main_branch_identity())
                .unwrap();
            let published = recovered
                .index_access()
                .read_published_generation_for_commit(&request, definition.index_id, &next.commit)
                .unwrap()
                .unwrap();
            let refreshed = recovered
                .index_authority()
                .refresh_for_basis(
                    DerivedIndexBuildRequest {
                        source_commit_id: next.commit.commit_id,
                        branch_id: next.commit.branch_id.clone(),
                        index_ids: vec![definition.index_id],
                    },
                    &next_basis,
                    Some(&snapshot),
                    DerivedIndexMaintenanceBudget {
                        maximum_work_units: None,
                        maximum_cold_record_slots: Some(0),
                        maximum_derived_rows: None,
                    },
                    &request,
                )
                .unwrap();
            assert_eq!(refreshed.generations.len(), 1);
            assert_eq!(
                refreshed.generations[0].generation_id,
                published.generation_id
            );
            let retained = recovered
                .index_access()
                .read_published_generation_for_observation(
                    &request,
                    definition.index_id,
                    &next_basis.observation(),
                )
                .unwrap()
                .unwrap();
            assert!(std::sync::Arc::ptr_eq(&published, &retained));
            let selected = recovered
                .index_access()
                .execute_bounded_entity_field_lookup(
                    BoundedEntityFieldLookupRequest::new(
                        next.snapshot.clone(),
                        definition.index_id,
                        KindId(1),
                        worth_relational::facade::transactions::planned_single_field_locator(
                            AspectKey::new("name").unwrap(),
                            FieldKey::new("name").unwrap(),
                        ),
                        AspectValue::String("continued".into()),
                        1,
                    )
                    .unwrap(),
                    BoundedIndexParityMode::Production,
                    &request,
                )
                .unwrap();
            assert_eq!(selected.candidate_entity_ids(), &[expected_next]);
            release_test_commit_snapshot(&recovered, &next);
        }
        recovered.snapshots().release_snapshot(&snapshot).unwrap();
        if let Some(prior) = prior {
            recovered.snapshots().release_snapshot(&prior).unwrap();
        }
    }
}
