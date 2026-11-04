//! Distinct cold-recovered custody; it never fabricates a live Native result.
use crate::branch::ProductBranchObservation;
use crate::durability::{
    ActualWorldPublication, AdmittedWorldOpenAttempt, WorldDurableArtifact,
    WorldDurableMediumBinding,
};
use crate::identity::{
    CompositeCommitIdentity, CompositePublicationAttemptIdentity,
    ProductUnpublishedOwnerEffectsIdentity,
};
use std::sync::{Arc, Mutex};
use worth_proof::{DurablyCommitted, RelationalPublicationReadmitted};
use worth_relational::facade::durability::{
    ReadmittedRelationalAssociatedEffect, RelationalRecoveredPublicationCustody,
};

pub(crate) mod continuation;
mod release;
pub(crate) use continuation::PreparedRecoveredPublication;
pub use continuation::{WorldRecoveredContinuationDenial, WorldRecoveredContinuationOutcome};
pub use release::{WorldRecoveredReleaseOutcome, WorldReleasedRecoveredEffects};

#[derive(Debug)]
pub(crate) enum RecoveredNativeCustody {
    OpenOnly,
    Relational(Arc<RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>>),
    Prepared(Arc<crate::history::PreparedReadmittedPublicationRecord>),
    Released,
}

#[derive(Debug, Clone)]
pub(crate) enum RecoveredNativeObservation {
    Original(Arc<RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>>),
    Prepared(Arc<crate::history::PreparedReadmittedPublicationRecord>),
}
impl RecoveredNativeObservation {
    fn effect(&self) -> &ReadmittedRelationalAssociatedEffect {
        match self {
            Self::Original(native) => native.custody().effect(),
            Self::Prepared(record) => record.effect(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct RecoveredWorldPublicationRecord {
    pub(crate) identity: ProductUnpublishedOwnerEffectsIdentity,
    pub(crate) attempt: CompositePublicationAttemptIdentity,
    pub(crate) commit: CompositeCommitIdentity,
    pub(crate) open: AdmittedWorldOpenAttempt,
    pub(crate) continued: Option<crate::durability::AdmittedWorldDisposition>,
    pub(crate) native_identity: Option<worth_relational::facade::history::RelationalCommitIdentity>,
    pub(crate) expected: ProductBranchObservation,
    pub(crate) binding: Arc<WorldDurableMediumBinding>,
    pub(crate) native: Mutex<RecoveredNativeCustody>,
    pub(crate) claimed: Mutex<bool>,
    pub(crate) release: Mutex<Option<WorldDurableArtifact>>,
    pub(crate) committed_release: Mutex<Option<DurablyCommitted<ActualWorldPublication>>>,
    pub(crate) continuation: Mutex<Option<Box<PreparedRecoveredPublication>>>,
    pub(crate) metadata_bytes: usize,
}

/// One current-Home claim over the SAME original durable custody identities.
/// The catalog owns the Native capability independently of this caller.
#[derive(Debug)]
pub struct WorldRecoveredUnpublishedEffects {
    pub(crate) record: Arc<RecoveredWorldPublicationRecord>,
    pub(crate) catalog: super::catalog::ProductUnpublishedRecoveryCatalog,
    pub(crate) source: Option<RecoveredNativeObservation>,
}

impl WorldRecoveredUnpublishedEffects {
    pub fn identity(&self) -> &ProductUnpublishedOwnerEffectsIdentity {
        &self.record.identity
    }
    pub fn attempt_identity(&self) -> &CompositePublicationAttemptIdentity {
        &self.record.attempt
    }
    pub fn reserved_commit_identity(&self) -> &CompositeCommitIdentity {
        &self.record.commit
    }
    pub fn expected_head(&self) -> &ProductBranchObservation {
        &self.record.expected
    }
    pub fn recovery_handle(&self) -> super::ProductUnpublishedRecoveryHandle {
        super::ProductUnpublishedRecoveryHandle::new(
            self.record.identity.clone(),
            self.catalog.affinity(),
        )
    }
    pub fn next_actions(&self) -> &[super::ProductUnpublishedNextAction] {
        use super::ProductUnpublishedNextAction as Action;
        let lost_permit = self
            .record
            .continuation
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .is_some_and(|phase| phase.movement_recovery_required());
        if self.record.binding.publication_recovery_required() || lost_permit {
            return &[Action::Inspect, Action::ReopenRequired];
        }
        if self
            .record
            .release
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some()
        {
            &[Action::Inspect, Action::ReleaseObligations]
        } else if self.native_effect().is_none() {
            &[
                Action::Inspect,
                Action::ReleaseObligations,
                Action::CloseOwner,
            ]
        } else if self.continuation_already_committed()
            || self
                .record
                .continuation
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_some()
        {
            &[Action::Inspect, Action::ContinueOriginalPublication]
        } else {
            &[
                Action::Inspect,
                Action::ContinueOriginalPublication,
                Action::ReleaseObligations,
                Action::CloseOwner,
            ]
        }
    }
    pub fn claim_incarnation(&self) -> worth_foundational::facade::HomeIncarnation {
        self.record.binding.binding().incarnation()
    }
    pub fn native_receipt(
        &self,
    ) -> Option<worth_relational::facade::history::RelationalCommitReceipt> {
        self.native_effect()
            .map(|effect| effect.subject().receipt().clone())
    }
    /// Borrow the original source admission; this never manufactures a live result.
    pub fn native_effect(&self) -> Option<&ReadmittedRelationalAssociatedEffect> {
        self.source.as_ref().map(RecoveredNativeObservation::effect)
    }
    pub(crate) fn refresh_source(&mut self) {
        self.source = match &*self.record.native.lock().unwrap_or_else(|e| e.into_inner()) {
            RecoveredNativeCustody::OpenOnly => None,
            RecoveredNativeCustody::Relational(native) => {
                Some(RecoveredNativeObservation::Original(native.clone()))
            }
            RecoveredNativeCustody::Prepared(record) => {
                Some(RecoveredNativeObservation::Prepared(record.clone()))
            }
            RecoveredNativeCustody::Released => None,
        };
    }
    pub fn continuation_already_committed(&self) -> bool {
        self.record.continued.is_some()
            || self
                .record
                .continuation
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .as_ref()
                .is_some_and(|phase| phase.disposition_committed())
    }
}

impl Drop for WorldRecoveredUnpublishedEffects {
    fn drop(&mut self) {
        // End the immutable Native view before admitting another exclusive claim.
        self.source = None;
        *self
            .record
            .claimed
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = false;
    }
}

pub(crate) struct RecoveredWorldPublicationInputs {
    pub(crate) identity: ProductUnpublishedOwnerEffectsIdentity,
    pub(crate) attempt: CompositePublicationAttemptIdentity,
    pub(crate) commit: CompositeCommitIdentity,
    pub(crate) open: AdmittedWorldOpenAttempt,
    pub(crate) continued: Option<crate::durability::AdmittedWorldDisposition>,
    pub(crate) native_identity: Option<worth_relational::facade::history::RelationalCommitIdentity>,
    pub(crate) expected: ProductBranchObservation,
    pub(crate) binding: Arc<WorldDurableMediumBinding>,
    pub(crate) native:
        Option<RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>>,
}

impl RecoveredWorldPublicationRecord {
    pub(crate) fn new(input: RecoveredWorldPublicationInputs, metadata_bytes: usize) -> Arc<Self> {
        Arc::new(Self {
            identity: input.identity,
            attempt: input.attempt,
            commit: input.commit,
            open: input.open,
            continued: input.continued,
            native_identity: input.native_identity,
            expected: input.expected,
            binding: input.binding,
            native: Mutex::new(
                input
                    .native
                    .map_or(RecoveredNativeCustody::OpenOnly, |native| {
                        RecoveredNativeCustody::Relational(Arc::new(native))
                    }),
            ),
            claimed: Mutex::new(false),
            release: Mutex::new(None),
            committed_release: Mutex::new(None),
            continuation: Mutex::new(None),
            metadata_bytes,
        })
    }
}
