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

mod continuation;
mod continuation_loan;
mod release;
pub(crate) use continuation::{
    ContinuationCommon, OriginalContinuationAdvance, OriginalHomeTransitionContinuation,
    PreparedOriginalContinuation,
};
pub(crate) use continuation_loan::RecoveredNativeContinuationLoan;
pub use release::{WorldRecoveredReleaseOutcome, WorldReleasedRecoveredEffects};

#[derive(Debug)]
pub(crate) enum RecoveredNativeCustody {
    OpenOnly,
    Relational(RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>),
    Continuing(Box<OriginalHomeTransitionContinuation>),
    Working,
    Released,
}

#[derive(Debug)]
pub(crate) struct RecoveredWorldPublicationRecord {
    pub(crate) identity: ProductUnpublishedOwnerEffectsIdentity,
    pub(crate) attempt: CompositePublicationAttemptIdentity,
    pub(crate) commit: CompositeCommitIdentity,
    pub(crate) open: AdmittedWorldOpenAttempt,
    pub(crate) expected: ProductBranchObservation,
    pub(crate) binding: Arc<WorldDurableMediumBinding>,
    pub(crate) native: Mutex<RecoveredNativeCustody>,
    pub(crate) control: Mutex<Option<Arc<crate::publication::HomeTransitionEffectControlState>>>,
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
    /// Encode only this admitted original's association. This is descriptive
    /// meaning, not permission to create or repeat an owner effect.
    pub fn caller_association_commitment(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: &worth_foundational::facade::HomeProgramTransitionPlanDescription,
        step: worth_foundational::facade::HomeProgramTransitionStepAssociation,
    ) -> Result<[u8; 32], crate::durability::WorldDurableMediumDenial> {
        crate::publication::home_transition_caller_association_commitment(
            execution,
            plan,
            step,
            &self.record.open.association,
        )
    }

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
    /// Actual retained progress, not a physical fate or permission to advance.
    pub fn home_transition_effect_progress(
        &self,
    ) -> Result<
        Option<crate::publication::WorldHomeTransitionEffectProgress>,
        crate::recovery::RuntimeWorldRecoveryDenial,
    > {
        match &*self
            .record
            .native
            .lock()
            .unwrap_or_else(|cause| cause.into_inner())
        {
            RecoveredNativeCustody::OpenOnly | RecoveredNativeCustody::Relational(_) => Ok(None),
            RecoveredNativeCustody::Continuing(work) => Ok(Some(work.progress())),
            RecoveredNativeCustody::Working => {
                Err(crate::recovery::RuntimeWorldRecoveryDenial::Busy)
            }
            RecoveredNativeCustody::Released => {
                Err(crate::recovery::RuntimeWorldRecoveryDenial::MissingRecord)
            }
        }
    }
    pub fn native_receipt(
        &self,
    ) -> Option<worth_relational::facade::history::RelationalCommitReceipt> {
        match &*self.record.native.lock().unwrap_or_else(|e| e.into_inner()) {
            RecoveredNativeCustody::Relational(native) => {
                Some(native.custody().effect().subject().receipt().clone())
            }
            RecoveredNativeCustody::OpenOnly => None,
            RecoveredNativeCustody::Continuing(original) => Some(original.native_receipt().clone()),
            RecoveredNativeCustody::Working => {
                unreachable!("a claim cannot observe its consuming advance")
            }
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
    pub(crate) open: AdmittedWorldOpenAttempt,
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
            control: Mutex::new(None),
            release: Mutex::new(None),
            committed_release: Mutex::new(None),
            metadata_bytes,
        })
    }
}
