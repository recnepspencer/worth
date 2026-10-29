//! A recorded outcome answers its retry before the caller's lease is opened:
//! every fresh-preparing lane resolves its key in the World's placement first,
//! and only a miss opens the funded call, whose refusal is the original one.
pub use std::num::NonZeroUsize;
pub use std::time::Duration;
pub use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
pub use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Advancement,
        WorthQueryManagedComputationInterruption as Interruption,
    },
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as RequestDenial,
        WorthQueryApplicationRetainedMutationOutcome as RetainedOutcome,
    },
    primary_graph::WorthQueryMutationHandlerWork,
    runtime::{
        CancellationSource, CancellationToken, ExecutionAllocationDenialKind,
        ExecutionAllocationPolicy as Policy, ExecutionResourceLease, LeaseRequest,
    },
};
pub use worth_relational::facade::history::CommitId;

pub use crate::facade::certification::document_retention_model::{
    host::{publish_on_first_program, DocumentRetentionRuntime, SEED_RETENTION},
    operator_identity::{authenticate_operator, request_scope},
    programs::RetentionProgramP0,
    readback::{observe_head, read_retention},
    retention_entry::{SetRetentionIntent, DOCUMENT_IDENTITY},
    schema::SetRetentionInput,
};

pub const RETENTION: u64 = 9;
/// Room for a whole fresh call; no court here reads an owner's edge.
pub const LEASE_MEMORY: u64 = 16 * 1024 * 1024;

pub fn lease_request(memory: u64, cancellation: CancellationToken) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                NonZeroUsize::new(1).unwrap(),
                Some(memory),
                Some(16_000_000),
            ),
        ),
        deadline: None,
        cancellation,
    }
}

/// The commit a recorded answer names, however the record was found.
pub fn recorded_commit<Denial: std::fmt::Debug, Result: std::fmt::Debug>(
    outcome: Outcome<Denial, Result>,
) -> CommitId {
    match outcome {
        Outcome::AlreadyCommitted(replayed) => replayed.commit_reference().commit_id,
        Outcome::PreviouslyCommitted(replayed) => replayed.commit_id(),
        other => panic!("a recorded key must answer with its commit: {other:?}"),
    }
}

/// Neither the zero-memory lease nor the stopped one can open a call, so a
/// lane that answers under them answered before opening; neither was charged.
pub fn assert_uncharged(leases: &[&ExecutionResourceLease<'_>]) {
    for lease in leases {
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}

pub const REPLAY_SAMPLES: usize = 15;

/// A host whose history records `population` committed keys, the first of
/// which the court replays.
#[path = "recorded_replay_ordering/lookup_growth.rs"]
pub mod lookup_growth;
