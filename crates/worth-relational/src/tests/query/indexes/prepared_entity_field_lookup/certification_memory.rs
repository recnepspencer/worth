//! Certification's original scope retains and charges the original caller error.
use super::*;
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn certification_keeps_owned_admission_error_and_refuses_its_real_heap() {
    let setup = crate::test_execution_policy::serial_request();
    let setup_execution = worth_execution::ExecutionRequest::serial(&setup);
    let runtime = runtime_with_index_field_aspects();
    let created = create_entity_outcome(&setup_execution, &runtime, "certification");
    let locator = aspect_field_locator(aspect_key("name"), field_key("name"));
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.certification.memory".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: locator.clone(),
        },
        branch_scoped: true,
    });
    let build = runtime.index_authority().build_for_commit(
        &setup_execution,
        DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());
    let view = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap();
    let prepared = runtime
        .index_access()
        .prepare_entity_field_lookup(&view, index.index_id, KindId(1), &locator, &setup_execution)
        .unwrap();
    for ceiling in [None, Some(64 * 1024)] {
        let lease = crate::tests::support::test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Serial,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::MIN, ceiling, None),
                ),
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .unwrap();
        let execution = worth_execution::ExecutionRequest::leased(&lease);
        let mut error = Some(vec![7_u8; 128 * 1024]);
        let original = error.as_ref().unwrap().as_ptr();
        let stopped = runtime
            .index_access()
            .execute_prepared_entity_field_lookup(
                &prepared,
                &string_aspect_value("certification"),
                1024,
                BoundedIndexParityMode::Certification,
                &execution,
                || {
                    if lease.charged_memory_bytes() > 0 {
                        Err(error.take().expect("one certification callback error"))
                    } else {
                        Ok(())
                    }
                },
            )
            .unwrap_err();
        assert!(
            error.is_none(),
            "the error came from the original paid certification callback"
        );
        match (ceiling, stopped) {
            (None, Stop::Admission(error)) => {
                assert_eq!(error.as_ptr(), original, "retain the original owned error");
                assert_eq!(error.len(), 128 * 1024);
            }
            (Some(_), Stop::Lookup(denial)) => assert!(matches!(
                denial.kind(),
                BoundedEntityFieldLookupDenialKind::Execution(
                    crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
                        ancestor: 0,
                        ..
                    }
                )
            )),
            (_, stopped) => panic!("wrong certification error custody: {stopped:?}"),
        }
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}
