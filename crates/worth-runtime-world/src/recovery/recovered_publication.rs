//! Distinct cold-recovered custody; it never fabricates a live Native result.
use crate::branch::ProductBranchObservation;
use crate::durability::{
    ActualWorldPublication, WorldDurableArtifact, WorldDurableMediumBinding, WorldOpenAttemptRecord,
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

mod release;
pub use release::{WorldRecoveredReleaseOutcome, WorldReleasedRecoveredEffects};

#[derive(Debug)]
pub(crate) enum RecoveredNativeCustody {
    OpenOnly,
    Relational(RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>),
    Released,
}

#[derive(Debug)]
pub(crate) struct RecoveredWorldPublicationRecord {
    pub(crate) identity: ProductUnpublishedOwnerEffectsIdentity,
    pub(crate) attempt: CompositePublicationAttemptIdentity,
    pub(crate) commit: CompositeCommitIdentity,
    pub(crate) open: WorldOpenAttemptRecord,
    pub(crate) expected: ProductBranchObservation,
    pub(crate) binding: Arc<WorldDurableMediumBinding>,
    pub(crate) native: Mutex<RecoveredNativeCustody>,
    pub(crate) claimed: Mutex<bool>,
    pub(crate) release: Mutex<Option<WorldDurableArtifact>>,
    pub(crate) committed_release: Mutex<Option<DurablyCommitted<ActualWorldPublication>>>,
    pub(crate) metadata_bytes: usize,
}

/// One current-Home claim over the SAME original durable custody identities.
/// The catalog owns the Native capability independently of this caller.
#[derive(Debug)]
pub struct WorldRecoveredUnpublishedEffects {
    pub(crate) record: Arc<RecoveredWorldPublicationRecord>,
    pub(crate) catalog: super::catalog::ProductUnpublishedRecoveryCatalog,
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
    pub fn claim_incarnation(&self) -> worth_foundational::facade::HomeIncarnation {
        self.record.binding.binding().incarnation()
    }
    pub fn native_receipt(
        &self,
    ) -> Option<worth_relational::facade::history::RelationalCommitReceipt> {
        match &*self.record.native.lock().unwrap_or_else(|e| e.into_inner()) {
            RecoveredNativeCustody::Relational(native) => {
                Some(native.custody().effect().subject().receipt().clone())
            }
            RecoveredNativeCustody::OpenOnly => None,
            RecoveredNativeCustody::Released => unreachable!("a released claim is consumed"),
        }
    }
}

impl Drop for WorldRecoveredUnpublishedEffects {
    fn drop(&mut self) {
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
    pub(crate) open: WorldOpenAttemptRecord,
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
            expected: input.expected,
            binding: input.binding,
            native: Mutex::new(input.native.map_or(
                RecoveredNativeCustody::OpenOnly,
                RecoveredNativeCustody::Relational,
            )),
            claimed: Mutex::new(false),
            release: Mutex::new(None),
            committed_release: Mutex::new(None),
            metadata_bytes,
        })
    }
}
