//! Fresh undo admission (R8.36 / R8.37 / R8.40).
//!
//! Undo derives a *request* from the installed authority × mechanism axes and
//! handle-carried commit facts. It never mutates history and never treats those
//! historical facts as current authority — fresh effect authority is required.

use worth_query_installation::facade::{
    ApplicationSchema, InstalledCorrectionAuthority, InstalledCorrectionMechanism,
    PublishedAftermathPosture, WorthQueryCanonicalWorkEvidence, WorthQueryCanonicalWorkPhases,
    WorthQueryInstalledAftermathContract,
};

use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryRetainedGovernedInput,
    WorthQueryTouchedRecordIdentity,
};

use super::recovery_handle::{
    RelinquishOnDenial, WorthQueryHeldRecoveryHandle, WorthQueryRecoveryHandle,
    WorthQueryRecoveryHandleBinding,
};
use super::recovery_progression::WorthQueryRecoveryEffectAuthority;
use super::undo_denial::{WorthQueryUndoDenial, WorthQueryUndoDenialKind};
use super::undo_evidence::prepare_undo_evidence;
use super::undo_intent::WorthQueryUndoIntentIdentity;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Admit undo against a live handle and freshly re-established authority.
    pub fn admit_undo(
        &self,
        handle: WorthQueryRecoveryHandle,
        authority: &WorthQueryRecoveryEffectAuthority,
    ) -> Result<WorthQueryUndoAdmission, WorthQueryUndoDenial> {
        admit_undo(handle, authority)
    }
}

/// Which correction request undo derived — keyed from installed axes, never
/// from a posture name alone (R8.36).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryUndoDerivedRequest {
    RecordedInverse,
    Compensation,
    Reconciliation,
}

/// Admitted undo request ready to re-enter ordinary mutation progression.
#[derive(Debug)]
pub struct WorthQueryUndoAdmission {
    recovery_handle: WorthQueryHeldRecoveryHandle,
    intent: WorthQueryUndoIntentIdentity,
    original_commit: worth_relational::facade::history::RelationalCommitReceipt,
    original_operation: [u8; 32],
    principal_scope_digest: [u8; 32],
    compatibility_generation: u64,
    runtime_instance: u64,
    derived_request: WorthQueryUndoDerivedRequest,
    touched_records: Vec<WorthQueryTouchedRecordIdentity>,
    retained_preimage: Option<super::WorthQueryRetainedPreImage>,
    retained_governed_input: Option<WorthQueryRetainedGovernedInput>,
    undo_admission_work: WorthQueryCanonicalWorkEvidence,
    _private: (),
}

/// Undo holds the handle inside its admission, so a denial in *progression*
/// would otherwise drop the admission, drop the handle, and take the commit's
/// recovery with it (Q8.21-L11).
///
/// Dropping is the whole implementation: the admission carries its handle in a
/// [`WorthQueryHeldRecoveryHandle`], whose own `Drop` records the non-event.
/// That is deliberately stronger than routing this one path — it also covers
/// every denial *after* the admission leaves Query, which is where the
/// combinator alone could not reach (Q8.22-C5).
impl RelinquishOnDenial for WorthQueryUndoAdmission {
    fn relinquish_held_handle(self) {
        drop(self);
    }
}

impl WorthQueryUndoAdmission {
    pub(crate) fn recovery_handle(&self) -> &WorthQueryRecoveryHandle {
        self.recovery_handle.get()
    }

    pub fn installed_operation(&self) -> &str {
        self.recovery_handle
            .get()
            .binding()
            .installed_aftermath_operation_slot()
    }

    pub(crate) fn into_recovery_handle(self) -> WorthQueryRecoveryHandle {
        self.recovery_handle.into_handle()
    }

    pub const fn intent(&self) -> &WorthQueryUndoIntentIdentity {
        &self.intent
    }

    pub(crate) const fn original_commit(
        &self,
    ) -> &worth_relational::facade::history::RelationalCommitReceipt {
        &self.original_commit
    }

    pub const fn derived_request(&self) -> WorthQueryUndoDerivedRequest {
        self.derived_request
    }

    pub(crate) const fn original_operation(&self) -> &[u8; 32] {
        &self.original_operation
    }

    pub(crate) const fn principal_scope_digest(&self) -> &[u8; 32] {
        &self.principal_scope_digest
    }

    pub(crate) const fn compatibility_generation(&self) -> u64 {
        self.compatibility_generation
    }

    pub(crate) const fn runtime_instance(&self) -> u64 {
        self.runtime_instance
    }

    /// Commit-derived touched records consumed by inverse derivation (C2).
    pub fn touched_records(&self) -> &[WorthQueryTouchedRecordIdentity] {
        &self.touched_records
    }

    /// Retained pre-image consumed from the receipt for RecordedInverse (R8.2).
    pub const fn retained_preimage(&self) -> Option<&super::WorthQueryRetainedPreImage> {
        self.retained_preimage.as_ref()
    }

    /// Exact original capability input retained through commit and handle.
    pub fn original_input<Input: 'static>(&self) -> Option<&Input> {
        self.retained_governed_input.as_ref()?.downcast_ref()
    }

    pub const fn undo_admission_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.undo_admission_work
    }

    /// Phase bag with the undo_admission slot populated exactly (R8.13 / §8).
    pub fn canonical_work_phases(&self) -> WorthQueryCanonicalWorkPhases {
        WorthQueryCanonicalWorkPhases::new(
            WorthQueryCanonicalWorkEvidence::zero(),
            WorthQueryCanonicalWorkEvidence::zero(),
            WorthQueryCanonicalWorkEvidence::zero(),
            self.undo_admission_work,
            WorthQueryCanonicalWorkEvidence::zero(),
        )
    }
}

/// Admit one undo from a live recovery handle and fresh effect authority.
///
/// The handle carries evidence of what happened. Current lawfulness comes only
/// from `authority` (which required fresh operation admission). No progression
/// step is skipped because the original receipt was once authorized (R8.37).
pub fn admit_undo(
    handle: WorthQueryRecoveryHandle,
    authority: &WorthQueryRecoveryEffectAuthority,
) -> Result<WorthQueryUndoAdmission, WorthQueryUndoDenial> {
    // Every denial below leaves the commit exactly as recoverable as it was:
    // the handle is relinquished rather than dropped, so the mint claim returns
    // and a corrected attempt is still possible (Q8.21-L11).
    let (handle, (binding, derived_request, evidence, intent)) =
        handle.admit_deriving(|handle| {
            authority
                .ensure_for(handle)
                .map_err(|denial| map_recovery_denial(denial.kind()))?;
            let binding = handle.binding().clone();
            let aftermath = binding.installed_aftermath();
            let derived_request = derive_request_from_axes(aftermath)?;
            let evidence = prepare_undo_evidence(&binding, aftermath, derived_request)?;
            let intent = WorthQueryUndoIntentIdentity::derive_parts(
                binding.committed_product_publication(),
                *binding.installed_operation(),
                *aftermath.identity().digest(),
                binding.runtime_instance_id(),
            )
            .map_err(|_| WorthQueryUndoDenial::new(WorthQueryUndoDenialKind::Stale))?;
            Ok((binding, derived_request, evidence, intent))
        })?;
    let aftermath = binding.installed_aftermath();
    let undo_admission_work = intent.work();
    assert_eq!(undo_admission_work.basis_preparations(), 1);
    assert_eq!(undo_admission_work.digest_derivations(), 1);
    assert_eq!(undo_admission_work.digest_text_materializations(), 0);
    Ok(WorthQueryUndoAdmission {
        recovery_handle: WorthQueryHeldRecoveryHandle::new(handle),
        intent,
        original_commit: binding.commit_reference().clone(),
        original_operation: *binding.installed_operation(),
        principal_scope_digest: principal_scope_digest(&binding),
        compatibility_generation: aftermath.compatibility_generation(),
        runtime_instance: binding.runtime_instance_id(),
        derived_request,
        touched_records: evidence.touched_records,
        retained_preimage: evidence.retained_preimage,
        retained_governed_input: Some(evidence.retained_governed_input),
        undo_admission_work,
        _private: (),
    })
}

fn principal_scope_digest(binding: &WorthQueryRecoveryHandleBinding) -> [u8; 32] {
    let scope = binding.principal_scope();
    let principal = scope.principal();
    let mut bytes = [0u8; 32];
    bytes[0..4].copy_from_slice(&principal.partition_id().to_le_bytes());
    bytes[4..12].copy_from_slice(&principal.local_slot().to_le_bytes());
    bytes[12..16].copy_from_slice(&principal.generation().to_le_bytes());
    bytes[16..24].copy_from_slice(&scope.runtime_authority().to_le_bytes());
    bytes
}

/// Classify why an irreversible installed aftermath has no undo lane (R8.39).
///
/// **Not the undo entry.** [`admit_undo`] is, and it takes a handle and derives
/// the contract from that handle's binding. This function exists because an
/// irreversible aftermath never mints a handle at all — `RecoveryNotAdmitted`
/// closes the lane before there is anything to derive from — so the typed cause
/// has to be read off the installed contract directly.
///
/// What a caller can still supply: the contract to ask about. That is safe
/// because [`WorthQueryInstalledAftermathContract`] has no public constructor —
/// a caller can only present one installation produced — and because the answer
/// is a diagnosis, not authority. Nothing consumes the `Ok(())` and no
/// privileged path opens on it.
pub fn deny_irreversible_undo_attempt(
    aftermath: &WorthQueryInstalledAftermathContract,
) -> Result<(), WorthQueryUndoDenial> {
    match (aftermath.authority(), aftermath.mechanism()) {
        (InstalledCorrectionAuthority::NotCorrectable, None) => {
            match aftermath.published_posture() {
                PublishedAftermathPosture::Irreversible => Err(WorthQueryUndoDenial::new(
                    classify_irreversible_cause(aftermath),
                )),
                _ => Ok(()),
            }
        }
        _ => Ok(()),
    }
}

pub(crate) fn derive_request_from_axes(
    aftermath: &WorthQueryInstalledAftermathContract,
) -> Result<WorthQueryUndoDerivedRequest, WorthQueryUndoDenial> {
    // Axis pair decides — published posture is derived from the same pair and
    // is consulted only as a consistency check, never as the branch key.
    match (aftermath.authority(), aftermath.mechanism()) {
        (
            InstalledCorrectionAuthority::RuntimeAlone,
            Some(InstalledCorrectionMechanism::RecordedInverse(_)),
        ) => {
            assert_eq!(
                aftermath.published_posture(),
                PublishedAftermathPosture::Reversible
            );
            Ok(WorthQueryUndoDerivedRequest::RecordedInverse)
        }
        (
            InstalledCorrectionAuthority::RuntimeAlone,
            Some(InstalledCorrectionMechanism::Compensation(_)),
        ) => {
            assert_eq!(
                aftermath.published_posture(),
                PublishedAftermathPosture::Compensatable
            );
            Ok(WorthQueryUndoDerivedRequest::Compensation)
        }
        (InstalledCorrectionAuthority::RuntimeWithExternalOwner, Some(_)) => {
            assert_eq!(
                aftermath.published_posture(),
                PublishedAftermathPosture::Reconcilable
            );
            Ok(WorthQueryUndoDerivedRequest::Reconciliation)
        }
        (InstalledCorrectionAuthority::NotCorrectable, None) => Err(WorthQueryUndoDenial::new(
            match aftermath.published_posture() {
                PublishedAftermathPosture::Irreversible => classify_irreversible_cause(aftermath),
                _ => WorthQueryUndoDenialKind::CorrectionNotAdmitted,
            },
        )),
        _ => Err(WorthQueryUndoDenial::new(
            WorthQueryUndoDenialKind::CorrectionNotAdmitted,
        )),
    }
}

fn classify_irreversible_cause(
    aftermath: &WorthQueryInstalledAftermathContract,
) -> WorthQueryUndoDenialKind {
    let slot = aftermath.operation_slot();
    // Released-estate is type-level absence of undo (R8.21) — classify before
    // escaped-effect so a released outcome never becomes a generic escape denial.
    if slot.contains("Release") || slot.contains("release") {
        WorthQueryUndoDenialKind::ReleasedEstate
    } else if aftermath.external_effect().is_declared() {
        // Irreversible + declared external effect: the effect escaped without a
        // compensatable/reconcilable path (R8.39 EscapedEffect).
        WorthQueryUndoDenialKind::EscapedEffect
    } else if slot.contains("Audit") || slot.contains("audit") {
        WorthQueryUndoDenialKind::IrreversibleAudit
    } else if slot.contains("Approval") || slot.contains("Approve") || slot.contains("approval") {
        WorthQueryUndoDenialKind::IrreversibleApproval
    } else {
        WorthQueryUndoDenialKind::IrreversibleLegal
    }
}

pub(crate) fn map_recovery_denial(
    kind: super::recovery_handle::WorthQueryRecoveryHandleDenialKind,
) -> WorthQueryUndoDenial {
    use super::recovery_handle::WorthQueryRecoveryHandleDenialKind as K;
    WorthQueryUndoDenial::new(match kind {
        K::AlreadyTerminal => WorthQueryUndoDenialKind::AlreadyConsumed,
        K::Expired => WorthQueryUndoDenialKind::Stale,
        K::CurrentPolicyDenied | K::FreshAuthorityDenied => {
            WorthQueryUndoDenialKind::CurrentPolicyDenied
        }
        K::ForeignRuntime | K::ForeignPrincipal => WorthQueryUndoDenialKind::ForeignHandle,
        _ => WorthQueryUndoDenialKind::CorrectionNotAdmitted,
    })
}
