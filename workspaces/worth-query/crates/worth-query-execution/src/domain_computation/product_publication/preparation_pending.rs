//! Original Query preparation retained across an actual durable World open.
use super::WorthQueryProductUnpublishedRecovery;
use crate::domain_computation::primary_graph::QueryPreparingApplicationCommit;
use worth_runtime_world::facade::{
    ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryPort, WorldOpenAttemptRefusal,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProductPreparationStage {
    Opening,
    CheckedOut,
    OpenPending,
    OpenUnresolved,
    OpenEvidenceRequired,
    BindingNativeProposal,
    PreparingNative,
    PreparingQueryCandidate,
    CompletingNative,
    Publishing,
    PublicationPending,
    PublicationRecoveryRequired,
    RecoveryOnly,
}

/// Original preparation remains in the existing Query custody, with the
/// original returned terminal retaining its checked-out guard. Duplicate
/// observations cannot take that guard. This move-only terminal carries exact
/// recovery access, not a performed result, no-effect proof, or retry right.
#[must_use]
pub struct WorthQueryProductPreparationPending {
    stage: WorthQueryProductPreparationStage,
    recovery: WorthQueryProductUnpublishedRecovery,
    preparation: Option<crate::domain_computation::primary_graph::QueryDurablePreparationGuard>,
}
impl WorthQueryProductPreparationPending {
    pub(in crate::domain_computation) fn new(
        stage: WorthQueryProductPreparationStage,
        handle: ProductUnpublishedRecoveryHandle,
        recovery: RuntimeWorldRecoveryPort,
        disposition: crate::domain_computation::primary_graph::WorthQueryUnpublishedIdempotencyDisposition,
    ) -> Self {
        Self {
            stage,
            recovery: WorthQueryProductUnpublishedRecovery::new(handle, recovery, disposition),
            preparation: None,
        }
    }
    pub(in crate::domain_computation) fn retaining_preparation(
        mut self,
        preparation: crate::domain_computation::primary_graph::QueryDurablePreparationGuard,
    ) -> Self {
        self.preparation = Some(preparation);
        self
    }
    pub const fn stage(&self) -> WorthQueryProductPreparationStage {
        self.stage
    }
    pub fn record_handle(&self) -> &ProductUnpublishedRecoveryHandle {
        self.recovery.record_handle()
    }
    /// Abandons only the actual retained World phase into its SAME catalog.
    /// This explicit handoff runs ordinary owner destructors after Query locks
    /// are released. It is not a deadline-bounded mutation-return conversion.
    /// Busy custody returns this original terminal unchanged. A persistent live
    /// open requires ending the current Home and reopening under a new lease;
    /// this handoff does not release its business key or authorize handler replay.
    pub fn try_into_recovery(mut self) -> Result<WorthQueryProductUnpublishedRecovery, Self> {
        // Only this explicit, unbudgeted handoff restores the original checkout.
        drop(self.preparation.take());
        if self.recovery.abandon_preparing_world().is_err() {
            return Err(self);
        }
        Ok(self.recovery)
    }
}
impl std::fmt::Debug for WorthQueryProductPreparationPending {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryProductPreparationPending")
            .field("stage", &self.stage)
            .field("recovery", &self.recovery)
            .finish()
    }
}

/// A terminal owner refusal before Native preparation, retaining the same
/// unprepared proposal, Query evidence and completion. No World recovery is
/// invented when no recoverable catalog entry exists. There is no retry door.
/// The exact business entry is released, but this carrier may still own partial
/// successor reservations. Discarding it releases those resources through
/// ordinary, unbudgeted owner destruction; a fresh attempt can remain resource
/// blocked while the carrier is held. That is not an idempotency key fence.
#[derive(Debug)]
pub enum WorthQueryProductPreparationRefusal {
    World(WorldOpenAttemptRefusal),
    QuerySuccessorObservation(&'static str),
    QueryPreparationReservationRefused,
    ConditionalPublicationRequired,
}
#[must_use]
pub struct WorthQueryProductPreparationRefused {
    cause: WorthQueryProductPreparationRefusal,
    _query: Box<QueryPreparingApplicationCommit>,
}
impl WorthQueryProductPreparationRefused {
    pub(in crate::domain_computation) fn new(
        cause: WorthQueryProductPreparationRefusal,
        query: QueryPreparingApplicationCommit,
    ) -> Self {
        assert!(
            query.is_unprepared(),
            "terminal refusal retains the original unprepared proposal"
        );
        Self {
            cause,
            _query: Box::new(query),
        }
    }
    pub fn reason(&self) -> &WorthQueryProductPreparationRefusal {
        &self.cause
    }
}
impl std::fmt::Debug for WorthQueryProductPreparationRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryProductPreparationRefused")
            .field("cause", &self.cause)
            .finish_non_exhaustive()
    }
}
