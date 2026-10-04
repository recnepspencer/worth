//! Publication exclusion and the single-member fence for one terminal head
//! retired. The fence admits exactly the registered tag-9 member: no record
//! append, drop or residue cleanup can borrow its slot.

use std::marker::PhantomData;
use std::sync::{Arc, MutexGuard};

use worth_store_physical_format::{ReleaseCustodyHeadKeyV1, RootPublicationCell};

use super::super::blob_claim::BlobClaimRegistry;
use super::super::release_capacity::{
    CheckpointAttestedTerminalHead, ReleaseCertificateCapacityDenial, ReleaseHeadCapacityCharge,
    TerminalHeadAttestationDenial,
};
use super::super::{PhysicalBlobSessionClaim, PhysicalCurrentRootOwner, PhysicalCurrentRootState};
use super::{
    PhysicalReclaimAttempt, PhysicalReclaimAttemptId, ReclaimFenceState, ReclaimPhase,
    ReclaimPurpose,
};
use crate::physical_runtime::{
    durability::PhysicalBlobSessionClaimDenial, stability::TerminalHeadNoReaderOrRecoveryHold,
    PhysicalMutationIdentity, PhysicalProtectedRootObservation, ProvenNoEffectPhysicalMutation,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) enum TerminalHeadRetirementAdmissionDenial {
    Claim(PhysicalBlobSessionClaimDenial),
    AlreadyFenced,
    SourceRootChanged,
    PendingPublication,
    DisplacedExtentOutstanding,
    Head(TerminalHeadAttestationDenial),
    ExternalProtectedReader,
    Capacity(ReleaseCertificateCapacityDenial),
    EntropyUnavailable,
}

/// Sealed: issued only under the publication-state lock, after the selected
/// root was proved unchanged with no pending publication, no competing fence,
/// no outstanding displaced extent and this session's claim promoted. The
/// installed fence keeps it true until the attempt ends.
pub(in crate::physical_runtime) struct TerminalHeadPublicationExcluded {
    key: ReleaseCustodyHeadKeyV1,
    root: RootPublicationCell,
}

impl TerminalHeadPublicationExcluded {
    pub(in crate::physical_runtime) const fn key(&self) -> ReleaseCustodyHeadKeyV1 {
        self.key
    }

    pub(in crate::physical_runtime) const fn root(&self) -> RootPublicationCell {
        self.root
    }

    #[cfg(test)]
    pub(in crate::physical_runtime) const fn fixture(
        key: ReleaseCustodyHeadKeyV1,
        root: RootPublicationCell,
    ) -> Self {
        Self { key, root }
    }
}

/// Witness that the publication-state lock is held for `'held`. Only this
/// admission builds one, from its own guard, so a check that takes it cannot
/// run outside the lock.
pub(in crate::physical_runtime) struct PublicationStateLockHeld<'held>(PhantomData<&'held ()>);

impl<'held> PublicationStateLockHeld<'held> {
    const fn of(_guard: &'held MutexGuard<'_, PhysicalCurrentRootState>) -> Self {
        Self(PhantomData)
    }

    #[cfg(test)]
    pub(in crate::physical_runtime) const fn fixture() -> Self {
        Self(PhantomData)
    }
}

/// The root owner's three contributions, issued together under one hold of
/// the publication-state lock, plus the fence attempt that keeps them true.
/// The facts never leave the attempt: the authority holds this value whole.
pub(in crate::physical_runtime) struct AdmittedTerminalHeadRetirement {
    attempt: PhysicalReclaimAttempt,
    publication: TerminalHeadPublicationExcluded,
    hold: TerminalHeadNoReaderOrRecoveryHold,
    head: CheckpointAttestedTerminalHead,
}

impl AdmittedTerminalHeadRetirement {
    pub(in crate::physical_runtime) const fn publication(
        &self,
    ) -> &TerminalHeadPublicationExcluded {
        &self.publication
    }

    pub(in crate::physical_runtime) const fn attempt(&self) -> &PhysicalReclaimAttempt {
        &self.attempt
    }

    pub(in crate::physical_runtime) const fn hold(&self) -> &TerminalHeadNoReaderOrRecoveryHold {
        &self.hold
    }

    pub(in crate::physical_runtime) const fn head(&self) -> &CheckpointAttestedTerminalHead {
        &self.head
    }
}

/// The first two exclusions, decided from the fence slot and the two root
/// cells alone. Another session's reclaim can install its fence, or publish
/// a root, after the inspecting reader was captured. Returns the one root
/// the retirement is fenced and proven at.
fn unfenced_source(
    fence: Option<&ReclaimFenceState>,
    selected: RootPublicationCell,
    inspected: RootPublicationCell,
) -> Result<RootPublicationCell, TerminalHeadRetirementAdmissionDenial> {
    if fence.is_some() {
        return Err(TerminalHeadRetirementAdmissionDenial::AlreadyFenced);
    }
    if selected != inspected {
        return Err(TerminalHeadRetirementAdmissionDenial::SourceRootChanged);
    }
    Ok(inspected)
}

/// The attempt's identity and this session's claim promoted in the owner's
/// registry: the last denials that precede the funding. The promotion is
/// process-local arbitration and ends with the claim, so a caller denied
/// after it leaves nothing behind by dropping the claim.
fn promoted_attempt(
    claim: &mut PhysicalBlobSessionClaim,
    registry: &Arc<BlobClaimRegistry>,
) -> Result<PhysicalReclaimAttemptId, TerminalHeadRetirementAdmissionDenial> {
    use TerminalHeadRetirementAdmissionDenial as Denial;
    let mut id = [0_u8; 16];
    getrandom::fill(&mut id).map_err(|_| Denial::EntropyUnavailable)?;
    if id == [0; 16] {
        return Err(Denial::EntropyUnavailable);
    }
    claim.promote_reclaim(registry).map_err(Denial::Claim)?;
    Ok(PhysicalReclaimAttemptId(id))
}

impl ReclaimFenceState {
    pub(super) fn accepts_terminal_head_retirement(
        &self,
        mutation: PhysicalMutationIdentity,
    ) -> bool {
        matches!(
            self.phase,
            ReclaimPhase::BeforeEffect | ReclaimPhase::RetirementEffect
        ) && self.retirement_mutation == Some(mutation)
    }

    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn is_published_terminal_head_retirement(
        &self,
        mutation: PhysicalMutationIdentity,
    ) -> bool {
        self.purpose == ReclaimPurpose::TerminalHeadRetirement
            && self.phase == ReclaimPhase::RetirementPublished
            && self.retirement_mutation == Some(mutation)
    }
}

impl PhysicalReclaimAttempt {
    pub(in crate::physical_runtime) fn register_terminal_head_retirement(
        &self,
        mutation: PhysicalMutationIdentity,
    ) -> bool {
        let mut state = self.lock_fence();
        let Some(fence) = state.as_mut().filter(|fence| fence.id == self.id) else {
            return false;
        };
        if fence.purpose != ReclaimPurpose::TerminalHeadRetirement
            || fence.phase != ReclaimPhase::BeforeEffect
            || fence.retirement_mutation.is_some()
        {
            return false;
        }
        fence.retirement_mutation = Some(mutation);
        true
    }

    /// After this point a dropped attempt keeps its fence: only a proved
    /// no-effect outcome or the selected-ledger commit can release it.
    pub(in crate::physical_runtime) fn mark_terminal_head_retirement_effect(&self) -> bool {
        let mut state = self.lock_fence();
        let Some(fence) = state.as_mut().filter(|fence| fence.id == self.id) else {
            return false;
        };
        if fence.purpose != ReclaimPurpose::TerminalHeadRetirement
            || fence.phase != ReclaimPhase::BeforeEffect
            || fence.retirement_mutation.is_none()
        {
            return false;
        }
        fence.phase = ReclaimPhase::RetirementEffect;
        true
    }

    pub(in crate::physical_runtime) fn prove_terminal_head_retirement_no_effect(
        &self,
        proof: &ProvenNoEffectPhysicalMutation,
    ) -> bool {
        let mut state = self.lock_fence();
        let Some(fence) = state.as_mut().filter(|fence| fence.id == self.id) else {
            return false;
        };
        if fence.purpose != ReclaimPurpose::TerminalHeadRetirement
            || fence.phase != ReclaimPhase::RetirementEffect
            || fence.retirement_mutation != Some(proof.mutation_identity())
        {
            return false;
        }
        fence.phase = ReclaimPhase::BeforeEffect;
        fence.retirement_mutation = None;
        true
    }
}

impl PhysicalCurrentRootOwner {
    /// One hold of the publication-state lock proves publication exclusion,
    /// attests the head, takes the no-hold fact, promotes the claim, funds
    /// the pending event and installs the fence. Funding is the last step
    /// that can deny, so every denial precedes any ledger or fence effect.
    pub(in crate::physical_runtime) fn admit_terminal_head_retirement(
        &self,
        claim: &mut PhysicalBlobSessionClaim,
        inspector: PhysicalProtectedRootObservation,
        key: ReleaseCustodyHeadKeyV1,
        charge: ReleaseHeadCapacityCharge,
    ) -> Result<AdmittedTerminalHeadRetirement, TerminalHeadRetirementAdmissionDenial> {
        use TerminalHeadRetirementAdmissionDenial as Denial;
        const NO_LEDGER: Denial =
            Denial::Head(TerminalHeadAttestationDenial::SelectedLedgerUnavailable);
        let mut guard = self.lock_publication_state();
        let mut fence = self.lock_reclaim();
        let source = unfenced_source(
            fence.as_ref(),
            guard.current_root.root_cell(),
            inspector.root(),
        )?;
        // The key's own head state answers first; the Store-wide exclusions
        // below would otherwise mask it behind the drop that made the head.
        let head = guard
            .release_ledger
            .selected()
            .ok_or(NO_LEDGER)?
            .attest_terminal_head(&guard.current_root, key)
            .map_err(Denial::Head)?;
        if self.publication.pending_len() != 0 {
            return Err(Denial::PendingPublication);
        }
        if self.publication.has_outstanding_displaced_extent() {
            return Err(Denial::DisplacedExtentOutstanding);
        }
        let hold = self
            .read_protection
            .attest_no_terminal_head_hold(&PublicationStateLockHeld::of(&guard), inspector, &head)
            .ok_or(Denial::ExternalProtectedReader)?;
        let id = promoted_attempt(claim, &self.blob_claims)?;
        guard
            .release_ledger
            .selected_mut()
            .ok_or(NO_LEDGER)?
            .fund_terminal_head_retirement(
                &self.release_allocation,
                self.recovery_allocation,
                key,
                charge,
            )
            .map_err(Denial::Capacity)?;
        *fence = Some(ReclaimFenceState {
            id,
            expected_root: source,
            manifest_mutation: None,
            reservation_mutation: None,
            drop_mutation: None,
            retirement_mutation: None,
            drop_records: Vec::new(),
            displaced: Vec::new(),
            phase: ReclaimPhase::BeforeEffect,
            purpose: ReclaimPurpose::TerminalHeadRetirement,
            _capacity: None,
            _recovered_reservations: Vec::new(),
            release_certificate_pending: None,
        });
        Ok(AdmittedTerminalHeadRetirement {
            attempt: PhysicalReclaimAttempt {
                fence: Arc::clone(&self.reclaim),
                state: Arc::downgrade(&self.state),
                id,
            },
            publication: TerminalHeadPublicationExcluded { key, root: source },
            hold,
            head,
        })
    }
}

#[cfg(test)]
#[path = "terminal_head_retirement/tests.rs"]
mod tests;
