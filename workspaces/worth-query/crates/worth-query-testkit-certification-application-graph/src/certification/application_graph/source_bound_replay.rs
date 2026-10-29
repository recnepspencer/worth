//! Real source-bound mutation retry under a stopped zero-allocation policy.
pub use std::num::NonZeroUsize;
pub use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
pub use worth_query_host::facade::{
    application_contribution::WorthQueryApplicationProducerProvider,
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as Denial,
    },
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryInvariantProjectionDenialKind as InvariantKind,
        WorthQueryMutationHandlerWork, WorthQueryOperationProjectionDenialKind as ProjectionKind,
    },
    runtime::{
        ExecutionAllocationDenialKind, ExecutionAllocationPolicy as Policy, LeaseDenial,
        LeaseRequest,
    },
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::{
    assessment_output::{
        OrdinaryRetentionAssessmentIntent, RetentionAssessmentProducer, RetentionAssessmentProvider,
    },
    host::{publish, SEED_RETENTION},
    operator_identity::{authenticate_operator, request_scope},
    readback::{observe_head, read_retention},
    retention_entry::{DocumentRetentionRead, DOCUMENT_IDENTITY},
    schema::DocumentRetentionSchema,
};

#[path = "source_bound_replay/program.rs"]
pub mod program;
