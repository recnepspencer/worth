//! Real installed mutation entry, physical custody, refusal, and replay.
pub use std::num::NonZeroUsize;
pub use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
pub use worth_query_execution::facade::provider_session::WorthQueryDecisionReadSetDenialKind;
use worth_query_host::facade::runtime::{ExecutionAllocationDenialClass, WorkCeilingDenial};
pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as RequestDenial,
    },
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryApplicationCommitDenialKind as CommitKind,
        WorthQueryApplicationCommitDenialStage as CommitStage, WorthQueryApplicationUncommitted,
        WorthQueryInvariantProjectionDenialKind as InvariantKind, WorthQueryMutationHandlerWork,
        WorthQueryOperationProjectionDenialKind as ProjectionKind,
    },
    runtime::{
        CancellationToken, ExecutionAllocationDenial, ExecutionAllocationPolicy as Policy,
        LeaseDenial, LeaseRequest,
    },
};

pub use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::{publish_on_first_program, SEED_RETENTION},
    operator_identity::{authenticate_operator, request_scope},
    readback::{observe_head, read_retention},
    retention_entry::{SetRetentionIntent, DOCUMENT_IDENTITY},
    schema::SetRetentionInput,
};

pub const PAYLOAD_BUDGET: u64 = 64 * 1024;
pub const RETENTION: u64 = 9;

pub fn lease_request(memory: u64, cancellation: CancellationToken) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), Some(memory), Some(1)),
        ),
        deadline: None,
        cancellation,
    }
}

pub fn assert_memory_refusal(cause: &ExecutionAllocationDenial) {
    assert_eq!(
        cause.classify(),
        ExecutionAllocationDenialClass::Request(WorkCeilingDenial::Admission(
            LeaseDenial::MemoryExhausted(
                worth_query_host::facade::runtime::MemoryLimitDenial::Capacity {
                    requested: cause.requested_payload_bytes().unwrap(),
                    admitted: 0,
                    level: worth_query_host::facade::runtime::MemoryLimitLevel::Policy {
                        ancestor: 0
                    }
                }
            )
        )),
    );
    // Preserve the owner's checked Layout quote. Private completed-fact counts
    // are not exposed here, so this court does not invent a byte/cardinality oracle.
    let quote = cause
        .requested_payload_bytes()
        .expect("checked owner quote survives");
    assert!(quote > 0);
}
