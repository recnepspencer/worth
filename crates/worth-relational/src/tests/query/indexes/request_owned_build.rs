use super::*;
use crate::facade::indexes::{DerivedIndexExecutionDenial, DerivedIndexExecutionDenialKind};
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn index_builds_use_the_memory_ceiling_of_their_request() {
    let host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    let runtime = runtime_with_index_field_aspects();
    let created = create_entity_outcome(&setup, &runtime, "request-owned-index");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let index_ids: Vec<_> = (0..32)
        .map(|ordinal| {
            runtime
                .index_authority()
                .register(DerivedIndexDefinition {
                    index_id: DerivedIndexId(10_000 + ordinal),
                    name: format!("request-owned-index-{ordinal}"),
                    kind: DerivedIndexKind::EntityField {
                        field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                    },
                    branch_scoped: true,
                })
                .index_id
        })
        .collect();
    let request = DerivedIndexBuildRequest {
        source_commit_id: created.commit.commit_id,
        branch_id: BranchId("main".into()),
        index_ids,
    };
    let baseline = runtime
        .index_authority()
        .build_for_basis(&setup, request.clone(), &basis);
    assert!(baseline.execution_denial.is_none());
    assert!(baseline.failed_indexes.is_empty());
    assert_eq!(baseline.generations.len(), 32);
    let published = runtime.index_access().generations_snapshot();
    let version = runtime.current_version_id();
    // A 4 KiB request admits the singleton planning scan. Its 512-byte
    // descriptor ceiling cannot hold 32 index descriptors, before any index
    // payload is cloned or any replacement generation can be published.
    for exact_basis in [false, true] {
        let lease = test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Automatic,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::MIN, 4096, 1_000_000),
                ),
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .unwrap();
        let execution = ExecutionRequest::leased(&lease);
        let refused = if exact_basis {
            runtime
                .index_authority()
                .build_for_basis(&execution, request.clone(), &basis)
        } else {
            runtime
                .index_authority()
                .build_for_commit(&execution, request.clone())
        };
        assert_eq!(
            refused.execution_denial,
            Some(DerivedIndexExecutionDenial {
                kind: DerivedIndexExecutionDenialKind::Cause(
                    crate::execution::RelationalExecutionDenialCause::ResultCapacityExceeded,
                ),
                partition_identity: Some(1),
            })
        );
        assert!(refused.generations.is_empty());
        assert_eq!(refused.failed_indexes, request.index_ids);
        assert_eq!(runtime.current_version_id(), version);
        assert_eq!(runtime.index_access().generations_snapshot(), published);
    }
    release_test_commit_snapshot(&runtime, &created);
}
