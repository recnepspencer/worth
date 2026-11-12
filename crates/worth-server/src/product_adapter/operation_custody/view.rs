//! Non-authoritative wire descriptions borrow real native custody; nothing here executes or settles.
use super::{
    contract::WorthServerQuerySourceAdapter as SourceCommand,
    custody::Custody,
    diagnostic::{self, WorthServerSourceDiagnostic},
};
use serde::Serialize;
use worth_query_host::facade::declaration::application_operation::ApplicationMutationBinding;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryOperationCloseDenial,
        WorthQuerySourceCompletion, WorthQuerySourceOnlyDenial,
        WorthQuerySourceOnlyPosture as Posture,
    },
    primary_graph::WorthQueryApplicationUncommitted as Commit,
};

#[derive(Serialize, schemars::JsonSchema)]
#[serde(tag = "phase")]
pub enum View {
    Preparing {
        preparation: super::contract::WorthServerSourcePreparationPhase,
        refused: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    Active {
        posture: SourcePosture,
        outcome: Option<Conclusion>,
        refusal: Option<WorthServerSourceDiagnostic>,
        close_refusal: Option<WorthServerSourceDiagnostic>,
    },
    Completed {
        recovered: bool,
        outcome_identity: Option<u64>,
        committed_basis: Option<String>,
        close_refusal: Option<WorthServerSourceDiagnostic>,
    },
    Terminal {
        outcome: Option<Conclusion>,
        keyed: Option<KeyedResolution>,
        admission_refusal: Option<WorthServerSourceDiagnostic>,
        advancement_failed: bool,
        close_refusal: Option<WorthServerSourceDiagnostic>,
    },
}
#[derive(Serialize, schemars::JsonSchema)]
pub enum SourcePosture {
    Initial,
    ProductUnpublished,
    RecoveryPerformed,
    RetainedDeliveryPending,
    ProvedAborted,
    RetryableNoEffect,
    Indeterminate,
    Terminal,
}
#[derive(Serialize, schemars::JsonSchema)]
pub enum Conclusion {
    Committed,
    AlreadyCommitted,
    RecoveredCommitted,
    IdempotencyIntentDrift,
    DomainDenied {
        diagnostic: WorthServerSourceDiagnostic,
    },
    Cancelled,
    DeadlineExceeded,
    Commit(CommitPosture),
}
#[derive(Serialize, schemars::JsonSchema)]
pub enum KeyedResolution {
    RecoveredCommitted,
    ReleasedOriginal,
    ProductUnpublished,
    ProductPreparationPending,
    Unseen,
    IntentDrift,
}
#[derive(Serialize, schemars::JsonSchema)]
pub enum CommitPosture {
    Committed,
    RecoveredCommitted,
    ProductPreparationPending,
    ProductPreparationRefused,
    CleanupRequired(Box<Self>),
    ProductStale,
    ProductUnpublished,
    NoEffect,
    OutputAdopted,
    Stale,
    Cancelled,
    TimedOut,
    Denied,
    Aborted,
    Deferred,
    Indeterminate,
}

pub(super) fn describe<C: SourceCommand>(
    application: &C,
    custody: &Custody<C>,
    refusal: Option<&WorthQuerySourceOnlyDenial>,
    close: Option<&WorthQueryOperationCloseDenial>,
) -> View {
    match custody {
        Custody::Preparing(owner)=>{let (preparation,refused,reason)=C::preparation(owner); View::Preparing{preparation,refused,reason}},
        Custody::Active(operation) => View::Active {
            posture: source(&operation.posture()),
            outcome:operation.native_retry_outcome().map(conclusion::<C>),
            refusal: refusal.map(diagnostic::refusal),
            close_refusal: close.map(diagnostic::cleanup),
        },
        Custody::Completed(done) => {
            let (recovered, committed_basis, outcome_identity) = match done.evidence() {
                WorthQuerySourceCompletion::Fresh { receipt, .. } => (false,
                    application.program().runtime().application_commit_basis_token(receipt).ok(),
                    receipt.outcome_identity().map(worth_query_host::facade::primary_graph::WorthQueryApplicationCommitOutcomeIdentity::get)),
                WorthQuerySourceCompletion::Recovered { receipt, .. } => (true,
                    application.program().runtime().application_commit_basis_token(receipt).ok(),
                    receipt.outcome_identity().map(worth_query_host::facade::primary_graph::WorthQueryApplicationCommitOutcomeIdentity::get)),
                WorthQuerySourceCompletion::Recorded { receipt, .. } =>
                    (true, None, Some(receipt.outcome_identity().get())),
            };
            View::Completed { recovered, committed_basis, outcome_identity,
                close_refusal: close.map(diagnostic::cleanup), }
        }
        Custody::Terminal(done) => View::Terminal {
            outcome: done.native_outcome().map(conclusion::<C>),
            keyed:done.native_keyed_resolution().map(|keyed|match keyed {
                worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution::ReleasedOriginal(_)=>KeyedResolution::ReleasedOriginal,
                worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution::ProductUnpublished(_)=>KeyedResolution::ProductUnpublished,
                worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution::ProductPreparationPending(_)=>KeyedResolution::ProductPreparationPending,
                worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution::Unseen=>KeyedResolution::Unseen,
                worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution::RecoveredCommitted(_)=>KeyedResolution::RecoveredCommitted,
                worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution::IntentDrift=>KeyedResolution::IntentDrift,
            }),
            admission_refusal: done.native_admission_denial().map(diagnostic::mutation),
            advancement_failed: done.native_advancement_failure().is_some(),
            close_refusal: close.map(diagnostic::cleanup),
        },
    }
}
fn conclusion<C: SourceCommand>(
    outcome: &Outcome<
        <C::Binding as ApplicationMutationBinding<C::Schema>>::Denial,
        <C::Binding as ApplicationMutationBinding<C::Schema>>::Result,
    >,
) -> Conclusion {
    match outcome {
        Outcome::Committed { .. } => Conclusion::Committed,
        Outcome::AlreadyCommitted(_) => Conclusion::AlreadyCommitted,
        Outcome::RecoveredCommitted(_) => Conclusion::RecoveredCommitted,
        Outcome::IdempotencyIntentDrift => Conclusion::IdempotencyIntentDrift,
        Outcome::DomainDenied(cause) => Conclusion::DomainDenied {
            diagnostic: C::domain_diagnostic(cause),
        },
        Outcome::Cancelled => Conclusion::Cancelled,
        Outcome::DeadlineExceeded => Conclusion::DeadlineExceeded,
        Outcome::Commit(posture) => Conclusion::Commit(commit(posture)),
    }
}
fn source(posture: &Posture) -> SourcePosture {
    match posture {
        Posture::Initial => SourcePosture::Initial,
        Posture::ProductUnpublished => SourcePosture::ProductUnpublished,
        Posture::RecoveryPerformed => SourcePosture::RecoveryPerformed,
        Posture::RetainedDeliveryPending => SourcePosture::RetainedDeliveryPending,
        Posture::ProvedAborted => SourcePosture::ProvedAborted,
        Posture::RetryableNoEffect => SourcePosture::RetryableNoEffect,
        Posture::Indeterminate => SourcePosture::Indeterminate,
        Posture::Terminal => SourcePosture::Terminal,
    }
}
fn commit(posture: &Commit) -> CommitPosture {
    match posture {
        Commit::ProductPreparationPending(_) => CommitPosture::ProductPreparationPending,
        Commit::ProductPreparationRefused(_) => CommitPosture::ProductPreparationRefused,
        Commit::CleanupRequired(cause) => {
            CommitPosture::CleanupRequired(Box::new(full_commit(cause.outcome())))
        }
        Commit::ProductStale(_) => CommitPosture::ProductStale,
        Commit::ProductUnpublished(_) => CommitPosture::ProductUnpublished,
        Commit::NoEffect(_) => CommitPosture::NoEffect,
        Commit::OutputAdopted(_) => CommitPosture::OutputAdopted,
        Commit::Stale(_) => CommitPosture::Stale,
        Commit::Cancelled => CommitPosture::Cancelled,
        Commit::TimedOut => CommitPosture::TimedOut,
        Commit::Denied(_) => CommitPosture::Denied,
        Commit::Aborted => CommitPosture::Aborted,
        Commit::Deferred(_) => CommitPosture::Deferred,
        Commit::Indeterminate(_) => CommitPosture::Indeterminate,
    }
}

fn full_commit(
    posture: &worth_query_host::facade::primary_graph::WorthQueryApplicationCommitOutcome,
) -> CommitPosture {
    use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitOutcome as Full;
    match posture {
        Full::Committed(_) => CommitPosture::Committed,
        Full::RecoveredCommitted(_) => CommitPosture::RecoveredCommitted,
        Full::CleanupRequired(cause) => {
            CommitPosture::CleanupRequired(Box::new(full_commit(cause.outcome())))
        }
        Full::ProductPreparationPending(_) => CommitPosture::ProductPreparationPending,
        Full::ProductPreparationRefused(_) => CommitPosture::ProductPreparationRefused,
        Full::ProductStale(_) => CommitPosture::ProductStale,
        Full::ProductUnpublished(_) => CommitPosture::ProductUnpublished,
        Full::NoEffect(_) => CommitPosture::NoEffect,
        Full::Stale(_) => CommitPosture::Stale,
        Full::Cancelled => CommitPosture::Cancelled,
        Full::TimedOut => CommitPosture::TimedOut,
        Full::Denied(_) => CommitPosture::Denied,
        Full::Aborted => CommitPosture::Aborted,
        Full::Deferred(_) => CommitPosture::Deferred,
        Full::Indeterminate(_) => CommitPosture::Indeterminate,
    }
}
