//! Pin the real bounded-index door's cold refusal and exact result prefix.
use super::*;
use crate::facade::indexes::{
    BoundedEntityFieldLookupDenialKind, BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexGenerationId, DerivedIndexId,
    DerivedIndexKind,
};
use worth_execution::ExecutionWorkCeiling;
use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};

#[test]
fn runtime_bounded_index_law_keeps_cold_stop_limit_and_generation() {
    let host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    for indexed in [false, true] {
        let runtime =
            runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
        let entities = ["first", "second", "third"]
            .map(|name| create_entity_in_partition(&setup, &runtime, name, PartitionId(7)));
        for entity in entities {
            update_entity(&setup, &runtime, entity, "shared");
        }
        let locator = AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            aspect_key("name"),
            CanonicalFieldPath::single(field_key("name")),
        );
        let definition = runtime.index_authority().register(DerivedIndexDefinition {
            index_id: DerivedIndexId(29),
            name: "read-law-bounded-name".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: locator.clone(),
            },
            branch_scoped: false,
        });
        if indexed {
            let built = runtime.index_authority().build_for_commit(
                &setup,
                DerivedIndexBuildRequest {
                    source_commit_id: runtime.history().latest_commit().unwrap().commit_id,
                    branch_id: BranchId("main".into()),
                    index_ids: vec![definition.index_id],
                },
            );
            assert!(built.failed_indexes.is_empty());
            assert_eq!(built.generations.len(), 1);
        }
        let snapshot = runtime.visibility_authority().snapshot();
        assert_eq!(
            runtime
                .read_truth()
                .query_plan_context(&snapshot)
                .unwrap()
                .version_id,
            VersionId(6)
        );
        for (value, limit, count, overflow) in [
            ("shared", 2, 2, true),
            ("shared", 3, 3, false),
            ("absent", 2, 0, false),
        ] {
            for (posture, width) in [
                (None, 1),
                (Some(ExecutionPosture::Serial), 1),
                (Some(ExecutionPosture::Automatic), 1),
                (Some(ExecutionPosture::Automatic), 4),
            ] {
                let lease = posture.map(|posture| {
                    test_execution_authority()
                        .request_lease(LeaseRequest {
                            policy: ExecutionRequestPolicy::new(
                                posture,
                                DeterminismContract::CanonicalBitwise,
                                ExecutionBudget::new(
                                    std::num::NonZeroUsize::new(width).unwrap(),
                                    None,
                                    Some(100_000),
                                ),
                            ),
                            deadline: None,
                            cancellation: CancellationToken::new(),
                        })
                        .unwrap()
                });
                let request = lease
                    .as_ref()
                    .map_or(ExecutionRequest::serial(&host), ExecutionRequest::leased);
                let input = BoundedEntityFieldLookupRequest::new(
                    snapshot.clone(),
                    definition.index_id,
                    KindId(1),
                    locator.clone(),
                    string_aspect_value(value),
                    limit,
                )
                .unwrap();
                let (outcome, report) = request
                    .run(ExecutionWorkCeiling::new(Some(100_000)), |_| {
                        runtime.index_access().execute_bounded_entity_field_lookup(
                            input,
                            BoundedIndexParityMode::Production,
                            &request,
                        )
                    })
                    .unwrap();
                // Serial backing stays at literal zero here until the serial-lease line lands.
                // This snapshot-input lane checks liveness but prices no positive
                // unit today; the admitted selected-view lane has its own quote.
                assert_eq!(report.charged_work(), 0);
                if indexed {
                    let outcome = outcome.unwrap();
                    assert_eq!(outcome.candidate_entity_ids(), &entities[..count]);
                    assert_eq!(outcome.examined_entry_count(), count);
                    assert_eq!(outcome.overflowed(), overflow);
                    // Exactly one generation was built from six fixture commits.
                    assert_eq!(outcome.generation_id(), DerivedIndexGenerationId(1));
                    assert_eq!(*outcome.retain_definition(), definition);
                } else {
                    let denial = outcome.unwrap_err();
                    assert_eq!(
                        denial.kind(),
                        BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable
                    );
                    assert_eq!(denial.examined_entry_count(), 0);
                }
            }
        }
    }
}
