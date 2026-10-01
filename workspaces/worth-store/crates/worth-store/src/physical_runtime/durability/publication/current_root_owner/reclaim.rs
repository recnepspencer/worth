use std::sync::{Arc, Mutex, Weak};

mod attempt;
mod displaced_batch;
mod manifest_residue;
pub(in crate::physical_runtime) use manifest_residue::{
    AdmittedManifestResidueRetirement, ManifestResidueDisplacement, ManifestResidueProof,
    PhysicalReconciledReclaimDescriptorFate, SelectedOriginalDropProof,
};
mod released;
mod settlement;
pub(in crate::physical_runtime) use released::AdmittedReleasedGenerationDrop;

use worth_store_physical_format::{
    FailedIngestReclaimBasisV1, PersistedRecordIdentity, RootPublicationCell,
};

use super::release_capacity::ReleaseCertificatePending;
use super::{
    PhysicalBlobSessionClaim, PhysicalCurrentRootOwner, PhysicalCurrentRootState,
    PhysicalRootPublicationIdentity,
};
use crate::physical_runtime::{
    durability::{
        retention::DisplacedCapacityLease, PhysicalBlobSessionClaimDenial,
        PhysicalRecoveredOriginalDropNoDurableEffect, PhysicalRootPublicationTransition,
        PhysicalRootPublicationTransitionDenial,
    },
    PhysicalMutationIdentity, PhysicalProtectedRootObservation, PhysicalRecordReader,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalBlobReclaimAdmissionDenial {
    Claim(PhysicalBlobSessionClaimDenial),
    SourceRootChanged,
    ExternalProtectedReader,
    PendingPublication,
    AlreadyFenced,
    Capacity,
    EntropyUnavailable,
    AttemptCollision,
    SelectedResidueInvalid,
    RouteUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PhysicalReclaimAttemptId([u8; 16]);

/// Store-issued, single-owner capability for one fenced reclaim operation.
/// Drop releases only a pre-effect fence; uncertainty remains inspection-bound.
pub(in crate::physical_runtime) struct PhysicalReclaimAttempt {
    fence: Arc<Mutex<Option<ReclaimFenceState>>>,
    state: Weak<Mutex<PhysicalCurrentRootState>>,
    id: PhysicalReclaimAttemptId,
}

impl PhysicalReclaimAttempt {
    pub(super) fn certificate_fence(&self) -> Arc<Mutex<Option<ReclaimFenceState>>> {
        Arc::clone(&self.fence)
    }
}

/// Only the selected failed-residue scanner supplies the contents. The reader
/// stays alive throughout both publications, retaining its exact root pin.
pub(in crate::physical_runtime) struct AdmittedFailedIngestDrop {
    _reader: PhysicalRecordReader,
    attempt: PhysicalReclaimAttempt,
    basis: FailedIngestReclaimBasisV1,
    dropped: Vec<PersistedRecordIdentity>,
    displaced: Vec<crate::physical_runtime::durability::DisplacedArtifact>,
    remaining: u64,
}

pub(super) fn valid_drop_set(
    dropped: &[PersistedRecordIdentity],
    displaced: &[crate::physical_runtime::durability::DisplacedArtifact],
) -> bool {
    !dropped.is_empty()
        && dropped.len() <= 1024
        && !dropped.windows(2).any(|pair| pair[0] >= pair[1])
        && displaced.len() == dropped.len()
        && !displaced
            .windows(2)
            .any(|pair| pair[0].artifact >= pair[1].artifact)
}

impl AdmittedFailedIngestDrop {
    pub(in crate::physical_runtime) fn new(
        reader: PhysicalRecordReader,
        attempt: PhysicalReclaimAttempt,
        basis: FailedIngestReclaimBasisV1,
        dropped: Vec<PersistedRecordIdentity>,
        displaced: Vec<crate::physical_runtime::durability::DisplacedArtifact>,
        remaining: u64,
    ) -> Option<Self> {
        if !valid_drop_set(&dropped, &displaced)
            || dropped.contains(&basis.declaration_record())
            || dropped.contains(&basis.abandoned_record())
        {
            return None;
        }
        Some(Self {
            _reader: reader,
            attempt,
            basis,
            dropped,
            displaced,
            remaining,
        })
    }

    pub(in crate::physical_runtime) fn attempt(&self) -> &PhysicalReclaimAttempt {
        &self.attempt
    }

    pub(in crate::physical_runtime) const fn basis(&self) -> FailedIngestReclaimBasisV1 {
        self.basis
    }

    pub(in crate::physical_runtime) fn dropped(&self) -> &[PersistedRecordIdentity] {
        &self.dropped
    }

    pub(in crate::physical_runtime) fn displaced(
        &self,
    ) -> &[crate::physical_runtime::durability::DisplacedArtifact] {
        &self.displaced
    }

    pub(in crate::physical_runtime) const fn remaining(&self) -> u64 {
        self.remaining
    }

    pub(in crate::physical_runtime) fn complete(self) -> bool {
        self.attempt.complete()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReclaimPhase {
    BeforeEffect,
    ManifestEffect,
    ManifestPublished,
    ReserveEffect,
    ReservePublished,
    DropEffect,
    DropPublished,
    ResidueEffect,
    ResiduePublished,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReclaimPurpose {
    PayloadDrop,
    ManifestResidue,
}

pub(super) struct ReclaimFenceState {
    pub(super) id: PhysicalReclaimAttemptId,
    pub(super) expected_root: RootPublicationCell,
    manifest_mutation: Option<PhysicalMutationIdentity>,
    reservation_mutation: Option<PhysicalMutationIdentity>,
    drop_mutation: Option<PhysicalMutationIdentity>,
    drop_records: Vec<PersistedRecordIdentity>,
    displaced: Vec<crate::physical_runtime::durability::DisplacedArtifact>,
    phase: ReclaimPhase,
    purpose: ReclaimPurpose,
    _capacity: DisplacedCapacityLease,
    _recovered_reservations: Vec<PhysicalRecoveredOriginalDropNoDurableEffect>,
    pub(super) release_certificate_pending: Option<ReleaseCertificatePending>,
}

impl ReclaimFenceState {
    pub(super) fn matches_attempt(&self, attempt: [u8; 16]) -> bool {
        self.id.0 == attempt
    }

    pub(super) fn is_pre_effect_payload_drop(&self) -> bool {
        self.purpose == ReclaimPurpose::PayloadDrop && self.phase == ReclaimPhase::BeforeEffect
    }

    pub(super) fn is_reservation_published_payload_drop(&self) -> bool {
        self.purpose == ReclaimPurpose::PayloadDrop && self.phase == ReclaimPhase::ReservePublished
    }

    pub(super) fn is_selected_drop_mutation(&self, mutation: PhysicalMutationIdentity) -> bool {
        self.purpose == ReclaimPurpose::PayloadDrop
            && self.phase == ReclaimPhase::DropPublished
            && self.drop_mutation == Some(mutation)
    }

    pub(super) fn selected_root_cell(&self) -> RootPublicationCell {
        self.expected_root
    }

    pub(super) fn accepts(&self, mutation: PhysicalMutationIdentity) -> bool {
        if self.purpose != ReclaimPurpose::PayloadDrop {
            // The root-only WAL meaning is not installed yet. No record append
            // may borrow this fence's mutation slot in the meantime.
            return false;
        }
        match self.phase {
            ReclaimPhase::BeforeEffect | ReclaimPhase::ManifestEffect => {
                self.manifest_mutation == Some(mutation)
            }
            ReclaimPhase::ManifestPublished | ReclaimPhase::ReserveEffect => {
                self.reservation_mutation == Some(mutation)
            }
            ReclaimPhase::ReservePublished | ReclaimPhase::DropEffect => {
                self.drop_mutation == Some(mutation)
            }
            ReclaimPhase::DropPublished
            | ReclaimPhase::ResidueEffect
            | ReclaimPhase::ResiduePublished => false,
        }
    }

    pub(super) fn advance(
        &mut self,
        mutation: PhysicalMutationIdentity,
        root: RootPublicationCell,
    ) {
        self.expected_root = root;
        self.phase = if self.manifest_mutation == Some(mutation) {
            ReclaimPhase::ManifestPublished
        } else if self.reservation_mutation == Some(mutation) {
            ReclaimPhase::ReservePublished
        } else {
            ReclaimPhase::DropPublished
        };
    }
}

impl PhysicalCurrentRootOwner {
    /// Called only after selected-session exclusivity was authenticated under
    /// the inspecting claim. It makes the reader check and capture fence atomic.
    pub(in crate::physical_runtime) fn admit_blob_reclaim(
        &self,
        claim: &mut PhysicalBlobSessionClaim,
        inspector: PhysicalProtectedRootObservation,
        displaced: &[crate::physical_runtime::durability::DisplacedArtifact],
        occupied_attempts: &[[u8; 16]],
        recovered_reservations: Vec<PhysicalRecoveredOriginalDropNoDurableEffect>,
    ) -> Result<PhysicalReclaimAttempt, PhysicalBlobReclaimAdmissionDenial> {
        let state = self.lock_publication_state();
        let mut fence = self.lock_reclaim();
        if fence.is_some() {
            return Err(PhysicalBlobReclaimAdmissionDenial::AlreadyFenced);
        }
        if state.current_root.root_cell() != inspector.root() {
            return Err(PhysicalBlobReclaimAdmissionDenial::SourceRootChanged);
        }
        if recovered_reservations.iter().any(|proof| {
            proof.selected_root() != inspector.root()
                || proof.registry_runtime() != self.runtime_identity
                || proof.reservation_record() == proof.reservation().manifest_record()
                || proof.reservation_sha256() == [0; 32]
        }) {
            return Err(PhysicalBlobReclaimAdmissionDenial::SelectedResidueInvalid);
        }
        if self
            .read_protection
            .external_root_at_or_below(inspector, state.current_root.generation())
        {
            return Err(PhysicalBlobReclaimAdmissionDenial::ExternalProtectedReader);
        }
        if self.publication.pending_len() != 0 {
            return Err(PhysicalBlobReclaimAdmissionDenial::PendingPublication);
        }
        if displaced.is_empty()
            || displaced.len() > 1024
            || displaced.iter().any(|entry| {
                entry.source_root != state.current_root.generation()
                    || entry.bytes == 0
                    || !matches!(
                        entry.artifact,
                        crate::physical_runtime::durability::RetiredArtifact::Extent { .. }
                    )
            })
            || displaced
                .windows(2)
                .any(|pair| pair[0].artifact >= pair[1].artifact)
        {
            return Err(PhysicalBlobReclaimAdmissionDenial::SelectedResidueInvalid);
        }
        let bytes = displaced
            .iter()
            .try_fold(0_u64, |sum, entry| sum.checked_add(entry.bytes))
            .ok_or(PhysicalBlobReclaimAdmissionDenial::Capacity)?;
        let capacity = self
            .publication
            .reserve_displaced_entries(displaced.len() as u32, bytes)
            .map_err(|_| PhysicalBlobReclaimAdmissionDenial::Capacity)?;
        let mut retained = Vec::new();
        retained
            .try_reserve_exact(displaced.len())
            .map_err(|_| PhysicalBlobReclaimAdmissionDenial::Capacity)?;
        retained.extend_from_slice(displaced);
        let mut id = [0_u8; 16];
        getrandom::fill(&mut id)
            .map_err(|_| PhysicalBlobReclaimAdmissionDenial::EntropyUnavailable)?;
        if id == [0; 16] {
            return Err(PhysicalBlobReclaimAdmissionDenial::EntropyUnavailable);
        }
        if occupied_attempts.contains(&id) {
            return Err(PhysicalBlobReclaimAdmissionDenial::AttemptCollision);
        }
        claim
            .promote_reclaim(&self.blob_claims)
            .map_err(PhysicalBlobReclaimAdmissionDenial::Claim)?;
        let id = PhysicalReclaimAttemptId(id);
        *fence = Some(ReclaimFenceState {
            id,
            expected_root: inspector.root(),
            manifest_mutation: None,
            reservation_mutation: None,
            drop_mutation: None,
            drop_records: Vec::new(),
            displaced: retained,
            phase: ReclaimPhase::BeforeEffect,
            purpose: ReclaimPurpose::PayloadDrop,
            _capacity: capacity,
            _recovered_reservations: recovered_reservations,
            release_certificate_pending: None,
        });
        Ok(PhysicalReclaimAttempt {
            fence: Arc::clone(&self.reclaim),
            state: Arc::downgrade(&self.state),
            id,
        })
    }

    pub(in crate::physical_runtime) fn begin_settled_group(
        &self,
        members: &[crate::physical_runtime::PhysicalRootPublicationMemberIdentity],
        identity: PhysicalRootPublicationIdentity,
        source_root: worth_store_physical_format::DurablePhysicalRootManifest,
    ) -> Result<PhysicalRootPublicationTransition, PhysicalRootPublicationTransitionDenial> {
        let state = self.lock_publication_state();
        let fence = self.lock_reclaim();
        if let Some(fence) = fence.as_ref() {
            let [member] = members else {
                return Err(PhysicalRootPublicationTransitionDenial::ReclaimFenced);
            };
            if fence.expected_root != source_root.root_cell()
                || !fence.accepts(member.mutation_identity())
            {
                return Err(PhysicalRootPublicationTransitionDenial::ReclaimFenced);
            }
        }
        self.transition
            .begin(identity, &state.current_root, source_root)
    }
}
