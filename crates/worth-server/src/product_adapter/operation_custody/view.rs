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
    PreviouslyCommitted,
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
    Unseen,
    AlreadyCommitted,
    IntentDrift,
}
#[derive(Serialize, schemars::JsonSchema)]
pub enum CommitPosture {
    CleanupRequired(Box<Self>),
    ProductStale,
    ProductUnpublished,
    NoEffect,
    Stale,
    Cancelled,
    TimedOut,
    Denied,
    Aborted,
    Deferred,
    SettlementDeferred,
    Indeterminate,
}

pub(super) fn describe<C: SourceCommand>(
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
            let (recovered, receipt) = match done.evidence() {
                WorthQuerySourceCompletion::Fresh { receipt, .. } => (false, receipt),
                WorthQuerySourceCompletion::Recovered { receipt, .. } => (true, receipt),
            };
            View::Completed {
                recovered,
                outcome_identity: receipt.outcome_identity().map(worth_query_host::facade::primary_graph::WorthQueryApplicationCommitOutcomeIdentity::get),
                close_refusal: close.map(diagnostic::cleanup),
            }
        }
        Custody::Terminal(done) => View::Terminal {
            outcome: done.native_outcome().map(conclusion::<C>),
            keyed:done.native_keyed_resolution().map(|keyed|match keyed {
                worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution::Unseen=>KeyedResolution::Unseen,
                worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(_)=>KeyedResolution::AlreadyCommitted,
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
        Outcome::PreviouslyCommitted(_) => Conclusion::PreviouslyCommitted,
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
        Commit::CleanupRequired(cause) => {
            CommitPosture::CleanupRequired(Box::new(commit(cause.outcome())))
        }
        Commit::ProductStale(_) => CommitPosture::ProductStale,
        Commit::ProductUnpublished(_) => CommitPosture::ProductUnpublished,
        Commit::NoEffect(_) => CommitPosture::NoEffect,
        Commit::Stale(_) => CommitPosture::Stale,
        Commit::Cancelled => CommitPosture::Cancelled,
        Commit::TimedOut => CommitPosture::TimedOut,
        Commit::Denied(_) => CommitPosture::Denied,
        Commit::Aborted => CommitPosture::Aborted,
        Commit::Deferred(_) => CommitPosture::Deferred,
        Commit::SettlementDeferred(_) => CommitPosture::SettlementDeferred,
        Commit::Indeterminate(_) => CommitPosture::Indeterminate,
    }
}
