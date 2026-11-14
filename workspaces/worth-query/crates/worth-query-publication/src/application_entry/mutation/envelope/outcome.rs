//! Results are exposed only through a token issued by the committed request.
use super::*;
pub struct WorthQueryEditStageToken<Result, Denial = ()> {
    pub(super) brand: Arc<()>,
    pub(super) ordinal: usize,
    pub(super) marker: PhantomData<fn() -> (Result, Denial)>,
}
impl<R, D> WorthQueryEditStageToken<R, D> {
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorthQueryEditStageResultDenial {
    ForeignEnvelope,
    WrongStageResult,
}
pub struct WorthQueryEditEnvelopeReceipt {
    pub(super) receipt: WorthQueryApplicationCommitReceipt,
    pub(super) results: Vec<Box<dyn Any + Send>>,
    pub(super) brand: Arc<()>,
}
impl WorthQueryEditEnvelopeReceipt {
    pub fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }
    pub fn result<R: 'static, D>(
        &self,
        token: &WorthQueryEditStageToken<R, D>,
    ) -> Result<&R, WorthQueryEditStageResultDenial> {
        if !Arc::ptr_eq(&self.brand, &token.brand) {
            return Err(WorthQueryEditStageResultDenial::ForeignEnvelope);
        }
        self.results
            .get(token.ordinal)
            .and_then(|value| value.downcast_ref::<RetainedAuthoredStageResult<R>>())
            .map(|stage| &stage.result)
            .ok_or(WorthQueryEditStageResultDenial::WrongStageResult)
    }
    pub fn stage_identity<R: 'static, D>(
        &self,
        token: &WorthQueryEditStageToken<R, D>,
    ) -> Result<&ApplicationEditStageIdentity, WorthQueryEditStageResultDenial> {
        self.result(token)?;
        Ok(&self.results[token.ordinal]
            .downcast_ref::<RetainedAuthoredStageResult<R>>()
            .expect("checked stage type")
            .identity)
    }
    pub fn stage_receipt<R: 'static, D>(
        &self,
        token: &WorthQueryEditStageToken<R, D>,
    ) -> Result<&WorthQueryApplicationCommitReceipt, WorthQueryEditStageResultDenial> {
        self.result(token)?;
        self.results[token.ordinal]
            .downcast_ref::<RetainedAuthoredStageResult<R>>()
            .expect("checked stage type")
            .receipt
            .as_ref()
            .ok_or(WorthQueryEditStageResultDenial::WrongStageResult)
    }
    pub fn stage_count(&self) -> usize {
        self.results.len()
    }
}
/// Committed carries live stage results. Canonically readmitted replay carries
/// recovered commit evidence; no handler runs and no live receipt or result is
/// reconstructed (milestone 3, D1675/D1676, superseding D1637).
pub enum WorthQueryEditEnvelopeOutcome {
    /// Resolution-only entry found no record; it executed no supplier/handler.
    Unseen,
    Committed(WorthQueryEditEnvelopeReceipt),
    AlreadyCommitted(WorthQueryApplicationCommitReceipt),
    IdempotencyIntentDrift,
    RecoveredCommitted(WorthQueryRecoveredApplicationCommitReceipt),
    Uncommitted(WorthQueryApplicationUncommitted),
    DomainDenied(WorthQueryEditStageDomainDenial),
    StageExecutionDenied(WorthQueryEditStageExecutionDenial),
    Cancelled {
        stage: usize,
    },
    DeadlineExceeded {
        stage: usize,
    },
    AdmissionBasisChanged(DeclaredEditStage),
    StageAfterProducer(DeclaredEditDependency),
    CyclicStages(DeclaredEditScopeCycle),
    Empty,
}

/// A domain denial belongs to the request and its exact authored stage; the
/// original Native cleanup/recovery outcome remains owned alongside it.
pub struct WorthQueryEditStageDomainDenial {
    pub(super) failure: AuthoredStageDomainDenial,
    pub(super) brand: Arc<()>,
    pub(super) outcome: Option<WorthQueryApplicationUncommitted>,
}
impl WorthQueryEditStageDomainDenial {
    pub fn commit_outcome(&self) -> Option<&WorthQueryApplicationUncommitted> {
        self.outcome.as_ref()
    }
    /// Transfer the original cleanup/recovery owner after projecting the denial.
    pub fn into_commit_outcome(self) -> Option<WorthQueryApplicationUncommitted> {
        self.outcome
    }
    pub fn identity(&self) -> &ApplicationEditStageIdentity {
        &self.failure.identity
    }
    pub fn binding(&self) -> &'static str {
        self.failure.binding
    }
    pub fn denial<R, D: 'static>(
        &self,
        token: &WorthQueryEditStageToken<R, D>,
    ) -> Result<&D, WorthQueryEditStageResultDenial> {
        if !Arc::ptr_eq(&self.brand, &token.brand)
            || token.ordinal != self.failure.identity.ordinal()
        {
            return Err(WorthQueryEditStageResultDenial::ForeignEnvelope);
        }
        self.failure
            .denial
            .downcast_ref()
            .ok_or(WorthQueryEditStageResultDenial::WrongStageResult)
    }
}

/// The exact execution or interruption cause, beside its original abort outcome.
pub struct WorthQueryEditStageExecutionDenial {
    pub(super) failure: AuthoredStageExecutionDenial,
    pub(super) outcome: WorthQueryApplicationUncommitted,
}
impl WorthQueryEditStageExecutionDenial {
    pub fn identity(&self) -> &ApplicationEditStageIdentity {
        &self.failure.identity
    }
    pub fn binding(&self) -> &'static str {
        self.failure.binding
    }
    pub fn cause(&self) -> &AuthoredStageExecutionCause {
        &self.failure.cause
    }
    pub fn commit_outcome(&self) -> &WorthQueryApplicationUncommitted {
        &self.outcome
    }
    /// Transfer the original cleanup/recovery owner after projecting the cause.
    pub fn into_commit_outcome(self) -> WorthQueryApplicationUncommitted {
        self.outcome
    }
}

impl std::fmt::Debug for WorthQueryEditEnvelopeOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unseen => f.write_str("Unseen"),
            Self::Committed(value) => f
                .debug_struct("Committed")
                .field("receipt", &value.receipt)
                .field("stage_count", &value.results.len())
                .finish_non_exhaustive(),
            Self::AlreadyCommitted(value) => {
                f.debug_tuple("AlreadyCommitted").field(value).finish()
            }
            Self::IdempotencyIntentDrift => f.write_str("IdempotencyIntentDrift"),
            Self::RecoveredCommitted(value) => {
                f.debug_tuple("RecoveredCommitted").field(value).finish()
            }
            Self::Uncommitted(value) => f.debug_tuple("Uncommitted").field(value).finish(),
            Self::DomainDenied(value) => f
                .debug_struct("DomainDenied")
                .field("stage", &value.failure.identity)
                .field("binding", &value.failure.binding)
                .field("abort", &value.outcome)
                .finish_non_exhaustive(),
            Self::StageExecutionDenied(value) => f
                .debug_struct("StageExecutionDenied")
                .field("failure", &value.failure)
                .field("abort", &value.outcome)
                .finish(),
            Self::Cancelled { stage } => f.debug_struct("Cancelled").field("stage", stage).finish(),
            Self::DeadlineExceeded { stage } => f
                .debug_struct("DeadlineExceeded")
                .field("stage", stage)
                .finish(),
            Self::AdmissionBasisChanged(value) => {
                f.debug_tuple("AdmissionBasisChanged").field(value).finish()
            }
            Self::StageAfterProducer(value) => {
                f.debug_tuple("StageAfterProducer").field(value).finish()
            }
            Self::CyclicStages(value) => f.debug_tuple("CyclicStages").field(value).finish(),
            Self::Empty => f.write_str("Empty"),
        }
    }
}

pub(super) fn settled<D: Send + 'static, R: 'static>(
    stage: usize,
    identity: ApplicationEditStageIdentity,
    binding: &'static str,
    brand: Arc<()>,
    outcome: crate::application_entry::mutation::WorthQueryApplicationMutationOutcome<D, R>,
) -> WorthQueryEditEnvelopeOutcome {
    use crate::application_entry::mutation::WorthQueryApplicationMutationOutcome as Outcome;
    match outcome {
        Outcome::AlreadyCommitted(receipt) => {
            WorthQueryEditEnvelopeOutcome::AlreadyCommitted(receipt)
        }
        Outcome::IdempotencyIntentDrift => WorthQueryEditEnvelopeOutcome::IdempotencyIntentDrift,
        Outcome::RecoveredCommitted(receipt) => {
            WorthQueryEditEnvelopeOutcome::RecoveredCommitted(receipt)
        }
        Outcome::Commit(cause) => WorthQueryEditEnvelopeOutcome::Uncommitted(cause),
        Outcome::Cancelled => WorthQueryEditEnvelopeOutcome::Cancelled { stage },
        Outcome::DeadlineExceeded => WorthQueryEditEnvelopeOutcome::DeadlineExceeded { stage },
        Outcome::DomainDenied(denial) => {
            WorthQueryEditEnvelopeOutcome::DomainDenied(WorthQueryEditStageDomainDenial {
                failure: AuthoredStageDomainDenial {
                    identity,
                    binding,
                    denial: Box::new(denial),
                },
                brand,
                outcome: None,
            })
        }
        Outcome::Committed { .. } => unreachable!("preparation never publishes a candidate"),
    }
}
