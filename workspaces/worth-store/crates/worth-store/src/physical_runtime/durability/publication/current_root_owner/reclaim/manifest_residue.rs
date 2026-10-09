use std::sync::Arc;

use worth_store_physical_format::{
    DurablePhysicalRootManifest, FailedIngestReclaimBasisV1, PersistedRecordIdentity,
    RootPublicationCell,
};

use super::{
    PhysicalBlobReclaimAdmissionDenial, PhysicalCurrentRootOwner, PhysicalReclaimAttempt,
    PhysicalReclaimAttemptId, ReclaimFenceState, ReclaimPhase, ReclaimPurpose,
};
use crate::physical_runtime::{
    durability::{
        NamespaceDurableManifestResidueRoot, PhysicalBlobSessionClaim,
        PhysicalOriginalDropNoEffect, PhysicalRootPublicationIdentity,
        PhysicalRootPublicationTransition, PhysicalRootPublicationTransitionDenial,
        RetainedPhysicalRoot, RetiredArtifact,
    },
    PhysicalRecordReader,
};
use sha2::{Digest, Sha256};

mod proof;
pub(in crate::physical_runtime) use proof::{
    ManifestResidueDisplacement, ManifestResidueProof, SelectedOriginalDropProof,
};

/// Exact original descriptor no-effect evidence, sealed by the idempotency
/// owner and then joined with the selected manifest/root custody.
#[derive(Clone, Copy)]
pub(in crate::physical_runtime) struct PhysicalReconciledReclaimDescriptorFate {
    store: [u8; 16],
    original_attempt: [u8; 16],
    basis_digest: [u8; 32],
    manifest_record: PersistedRecordIdentity,
    manifest_sha256: [u8; 32],
    selected_root: RootPublicationCell,
    drop_idempotency: [u8; 32],
    drop_fingerprint: [u8; 32],
}

impl PhysicalReconciledReclaimDescriptorFate {
    pub(in crate::physical_runtime) fn from_original_drop_no_effect(
        original: PhysicalOriginalDropNoEffect,
        basis: FailedIngestReclaimBasisV1,
        manifest_record: PersistedRecordIdentity,
        manifest_sha256: [u8; 32],
        selected_root: RootPublicationCell,
    ) -> Option<Self> {
        (manifest_sha256 != [0; 32]).then_some(Self {
            store: original.store(),
            original_attempt: original.attempt(),
            basis_digest: basis.digest(original.store()),
            manifest_record,
            manifest_sha256,
            selected_root,
            drop_idempotency: original.idempotency_identity(),
            drop_fingerprint: original.request_fingerprint(),
        })
    }

    fn matches(
        self,
        store: [u8; 16],
        original_attempt: [u8; 16],
        basis: FailedIngestReclaimBasisV1,
        manifest_record: PersistedRecordIdentity,
        manifest_sha256: [u8; 32],
        selected_root: RootPublicationCell,
    ) -> bool {
        self.store == store
            && self.original_attempt == original_attempt
            && self.basis_digest == basis.digest(store)
            && self.manifest_record == manifest_record
            && self.manifest_sha256 == manifest_sha256
            && self.selected_root == selected_root
    }
}

/// Pre-effect custody only. No mutation can currently register against its
/// fence; the root-only WAL/C8 operation must be installed before submission.
pub(in crate::physical_runtime) struct AdmittedManifestResidueRetirement {
    _reader: PhysicalRecordReader,
    _attempt: PhysicalReclaimAttempt,
    _basis: FailedIngestReclaimBasisV1,
    _manifest_record: PersistedRecordIdentity,
    _manifest_sha256: [u8; 32],
    _displaced: ManifestResidueDisplacement,
    _proof: ManifestResidueProof,
}

impl AdmittedManifestResidueRetirement {
    pub(in crate::physical_runtime) const fn displaced_manifest(
        &self,
    ) -> ManifestResidueDisplacement {
        self._displaced
    }

    pub(in crate::physical_runtime) const fn source_root_generation(&self) -> u64 {
        self._displaced.manifest.source_root
    }

    pub(in crate::physical_runtime) fn proof(&self) -> ManifestResidueProof {
        self._proof
    }

    pub(in crate::physical_runtime) fn removed_records(
        &self,
    ) -> [Option<PersistedRecordIdentity>; 2] {
        [
            Some(self._manifest_record),
            self._proof.reserved().map(|value| value.record()),
        ]
    }

    pub(in crate::physical_runtime) const fn manifest_record(&self) -> PersistedRecordIdentity {
        self._manifest_record
    }

    pub(in crate::physical_runtime) const fn manifest_sha256(&self) -> [u8; 32] {
        self._manifest_sha256
    }

    pub(in crate::physical_runtime) fn source_basis_digest(&self, store: [u8; 16]) -> [u8; 32] {
        self._basis.digest(store)
    }

    pub(in crate::physical_runtime) fn mark_effect_started(&self) -> bool {
        let mut state = self._attempt.lock_fence();
        let Some(fence) = state.as_mut().filter(|fence| fence.id == self._attempt.id) else {
            return false;
        };
        if fence.purpose != ReclaimPurpose::ManifestResidue
            || fence.phase != ReclaimPhase::BeforeEffect
        {
            return false;
        }
        fence.phase = ReclaimPhase::ResidueEffect;
        true
    }

    pub(in crate::physical_runtime) fn complete(self) -> bool {
        let mut state = self._attempt.lock_fence();
        if state.as_ref().is_some_and(|fence| {
            fence.id == self._attempt.id
                && fence.purpose == ReclaimPurpose::ManifestResidue
                && fence.phase == ReclaimPhase::ResiduePublished
        }) {
            *state = None;
            true
        } else {
            false
        }
    }
}

impl PhysicalCurrentRootOwner {
    pub(in crate::physical_runtime) fn register_manifest_residue_pending(
        &self,
        admitted: &AdmittedManifestResidueRetirement,
        identity: crate::physical_runtime::PhysicalMutationIdentity,
    ) -> Result<
        crate::physical_runtime::durability::PendingPublicationLease,
        crate::physical_runtime::durability::PhysicalPublicationAdmissionDenial,
    > {
        let state = self.lock_publication_state();
        let fence = self.lock_reclaim();
        if !fence.as_ref().is_some_and(|fence| {
            fence.id == admitted._attempt.id
                && fence.purpose == ReclaimPurpose::ManifestResidue
                && fence.phase == ReclaimPhase::BeforeEffect
                && fence.expected_root == state.current_root.root_cell()
        }) {
            return Err(crate::physical_runtime::durability::PhysicalPublicationAdmissionDenial::ReclaimFenced);
        }
        self.publication.register_exclusive_pending(identity)
    }

    pub(in crate::physical_runtime) fn begin_manifest_residue(
        &self,
        admitted: &AdmittedManifestResidueRetirement,
        identity: PhysicalRootPublicationIdentity,
        source: DurablePhysicalRootManifest,
    ) -> Result<PhysicalRootPublicationTransition, PhysicalRootPublicationTransitionDenial> {
        let state = self.lock_publication_state();
        let fence = self.lock_reclaim();
        let Some((_, intent, _)) = identity.manifest_residue_basis() else {
            return Err(PhysicalRootPublicationTransitionDenial::ReclaimFenced);
        };
        if !fence.as_ref().is_some_and(|fence| {
            fence.id == admitted._attempt.id
                && fence.purpose == ReclaimPurpose::ManifestResidue
                && fence.phase == ReclaimPhase::BeforeEffect
                && fence.expected_root == state.current_root.root_cell()
                && intent.manifest_record() == admitted._manifest_record
                && intent.manifest_frame_sha256() == admitted._manifest_sha256
                && intent.source_basis_digest() == admitted._basis.digest(intent.store())
                && intent.proof() == admitted._proof.wire_proof()
                && intent.source_root_generation() == source.generation()
        }) {
            return Err(PhysicalRootPublicationTransitionDenial::ReclaimFenced);
        }
        self.transition.begin(identity, &state.current_root, source)
    }

    pub(in crate::physical_runtime) fn advance_manifest_residue_root(
        &self,
        admitted: &AdmittedManifestResidueRetirement,
        durable: NamespaceDurableManifestResidueRoot,
        format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    ) -> Result<(), crate::physical_runtime::durability::PhysicalRetirementDenial> {
        use crate::physical_runtime::durability::PhysicalRetirementDenial;
        let mut state = self.lock_publication_state();
        let mut fence = self.lock_reclaim();
        let Some((operation, intent, wal_digest)) =
            durable.transition.identity().manifest_residue_basis()
        else {
            return Err(PhysicalRetirementDenial::Delete);
        };
        let candidate = durable.candidate.successor_root();
        let digest: [u8; 32] = Sha256::digest(candidate.encode(format)).into();
        let valid_fence = fence.as_ref().is_some_and(|fence| {
            fence.id == admitted._attempt.id
                && fence.purpose == ReclaimPurpose::ManifestResidue
                && fence.phase == ReclaimPhase::ResidueEffect
                && fence.expected_root == state.current_root.root_cell()
        });
        if !valid_fence
            || durable.transition.source_root() != &state.current_root
            || durable.candidate.source_root() != &state.current_root
            || candidate.tier_epoch_anchor() != state.current_root.tier_epoch_anchor()
            || intent.source_root_generation() != state.current_root.generation()
            || intent.candidate_root_generation() != candidate.generation()
            || intent.candidate_root_sha256() != digest
            || intent.proof() != admitted._proof.wire_proof()
            || wal_digest != durable.receipt.payload_digest()
        {
            return Err(PhysicalRetirementDenial::Delete);
        }
        let (source, free, root, _, _) = durable.candidate.into_root_parts();
        state.namespace_evidence = crate::physical_runtime::PhysicalRootNamespaceDurabilityEvidence::ManifestResidueCurrentRoot {
            operation,
            manifest: intent.manifest_record(),
            source_generation: source.generation(),
            current_generation: root.generation(),
            replacement: durable.replacement,
            namespace_synchronization: durable.namespace,
        };
        state.previous_root = Some(RetainedPhysicalRoot::from_manifest(source));
        state.current_root = root;
        state.free_space = free;
        if let Some(fence) = fence.as_mut() {
            fence.expected_root = state.current_root.root_cell();
            fence.phase = ReclaimPhase::ResiduePublished;
        }
        self.publication
            .retain_displaced(admitted._displaced.manifest);
        if let Some(reserved) = admitted._displaced.reserved {
            self.publication.retain_displaced(reserved);
        }
        durable.transition.release();
        Ok(())
    }
}

impl PhysicalCurrentRootOwner {
    /// Atomically excludes external older/current readers and competing
    /// publications while reserving the one manifested extent's retirement.
    pub(in crate::physical_runtime) fn admit_manifest_residue_retirement(
        &self,
        claim: &mut PhysicalBlobSessionClaim,
        reader: PhysicalRecordReader,
        basis: FailedIngestReclaimBasisV1,
        manifest_record: PersistedRecordIdentity,
        manifest_sha256: [u8; 32],
        displaced: ManifestResidueDisplacement,
        proof: ManifestResidueProof,
    ) -> Result<AdmittedManifestResidueRetirement, PhysicalBlobReclaimAdmissionDenial> {
        let inspector = reader.protected_root();
        let state = self.lock_publication_state();
        let mut fence = self.lock_reclaim();
        if fence.is_some() {
            return Err(PhysicalBlobReclaimAdmissionDenial::AlreadyFenced);
        }
        let selected_root = state.current_root.root_cell();
        if selected_root != inspector.root() {
            return Err(PhysicalBlobReclaimAdmissionDenial::SourceRootChanged);
        }
        if !proof.matches(
            reader.store_identity().bytes(),
            proof.attempt(),
            basis,
            manifest_record,
            manifest_sha256,
            selected_root,
        ) || proof
            .registry_runtime()
            .is_some_and(|runtime| runtime != self.runtime_identity)
            || displaced.manifest.source_root != state.current_root.generation()
            || displaced.manifest.bytes == 0
            || !matches!(displaced.manifest.artifact, RetiredArtifact::Extent { .. })
            || displaced.reserved.is_some() != proof.reserved().is_some()
            || displaced.reserved.is_some_and(|reserved| {
                reserved.source_root != state.current_root.generation()
                    || reserved.bytes == 0
                    || !matches!(reserved.artifact, RetiredArtifact::Extent { .. })
                    || reserved.artifact == displaced.manifest.artifact
            })
        {
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
        let bytes = displaced
            .bytes()
            .ok_or(PhysicalBlobReclaimAdmissionDenial::Capacity)?;
        let capacity = self
            .publication
            .reserve_displaced_entries(displaced.count(), bytes)
            .map_err(|_| PhysicalBlobReclaimAdmissionDenial::Capacity)?;
        let mut retained = Vec::new();
        retained
            .try_reserve_exact(displaced.count() as usize)
            .map_err(|_| PhysicalBlobReclaimAdmissionDenial::Capacity)?;
        retained.push(displaced.manifest);
        if let Some(reserved) = displaced.reserved {
            retained.push(reserved);
        }
        let mut id = [0_u8; 16];
        getrandom::fill(&mut id)
            .map_err(|_| PhysicalBlobReclaimAdmissionDenial::EntropyUnavailable)?;
        if id == [0; 16] || id == proof.attempt() {
            return Err(PhysicalBlobReclaimAdmissionDenial::AttemptCollision);
        }
        claim
            .promote_reclaim(&self.blob_claims)
            .map_err(PhysicalBlobReclaimAdmissionDenial::Claim)?;
        let id = PhysicalReclaimAttemptId(id);
        *fence = Some(ReclaimFenceState {
            id,
            expected_root: selected_root,
            manifest_mutation: None,
            reservation_mutation: None,
            drop_mutation: None,
            retirement_mutation: None,
            drop_records: Vec::new(),
            displaced: retained,
            phase: ReclaimPhase::BeforeEffect,
            purpose: ReclaimPurpose::ManifestResidue,
            _capacity: Some(capacity),
            _recovered_reservations: Vec::new(),
            release_certificate_pending: None,
        });
        drop(state);
        Ok(AdmittedManifestResidueRetirement {
            _reader: reader,
            _attempt: PhysicalReclaimAttempt {
                fence: Arc::clone(&self.reclaim),
                state: Arc::downgrade(&self.state),
                id,
            },
            _basis: basis,
            _manifest_record: manifest_record,
            _manifest_sha256: manifest_sha256,
            _displaced: displaced,
            _proof: proof,
        })
    }
}

#[cfg(test)]
#[path = "manifest_residue/tests.rs"]
mod tests;
