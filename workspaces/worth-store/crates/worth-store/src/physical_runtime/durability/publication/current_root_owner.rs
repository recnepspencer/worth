use std::collections::HashMap;
use std::sync::Mutex;

mod advance_validation;
use advance_validation::validate_advance;
mod blob_claim;
#[cfg(feature = "certification-test-authority")]
mod capture_pause;
mod certificate_capacity;
mod displaced;
mod maintenance;
mod pending_publication;
mod reclaim;
mod recovered_copy;
mod recovered_custody;
pub(in crate::physical_runtime) use recovered_custody::PreparedRecoveredCheckpointCustody;
pub(in crate::physical_runtime) use release_capacity::RecoveredReleaseLedgerDenial;
mod release_capacity;
pub use release_capacity::SelectedReleaseHeadDenial;
mod root_basis;
mod root_capture;
pub(in crate::physical_runtime) use root_capture::ReleasedDropSourceCaptureDenial;
mod tier_epoch;
#[cfg(feature = "certification-test-authority")]
pub use capture_pause::{CertificationReadRootCapturePauseGate, CertificationReadRootCaptureStage};

use worth_proof::NonEmpty;
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, RecordArtifactFile,
};

use super::{
    PhysicalRootPublicationIdentity, PhysicalRootPublicationTransition,
    PhysicalRootPublicationTransitionDenial, PhysicalRootPublicationTransitionOwner,
    RetainedPhysicalRoot,
};
use crate::physical_runtime::{
    PhysicalDurabilityGroupBasis, PhysicalMutationIdentity, PhysicalRootPublicationMemberIdentity,
    RootNamespaceDurablePhysicalMutationMembers, RootPublicationPhysicalMutationMember,
};
pub(in crate::physical_runtime) use blob_claim::{
    PhysicalBlobSessionClaim, PhysicalBlobSessionClaimDenial, PhysicalBlobTerminalAdmissionDenial,
};
pub(in crate::physical_runtime) use certificate_capacity::{
    CheckpointCustodyDenial, CheckpointCustodyOrigin, SelectedCheckpointCertificate,
    SelectedCheckpointCustodySnapshot,
};
pub(in crate::physical_runtime) use reclaim::{
    AdmittedFailedIngestDrop, AdmittedManifestResidueRetirement, AdmittedReleasedGenerationDrop,
    ManifestResidueDisplacement, ManifestResidueProof, PhysicalBlobReclaimAdmissionDenial,
    PhysicalReclaimAttempt, PhysicalReconciledReclaimDescriptorFate, SelectedOriginalDropProof,
};
pub(in crate::physical_runtime) use release_capacity::{
    ReleaseCertificateCapacityDenial, ReleaseCertificateCapacityLease, ReleaseHeadCapacityCharge,
    SelectedReleaseCustodyLedger, SelectedReleaseHeadBasis,
};

pub(in crate::physical_runtime) struct PhysicalCurrentRootOwner {
    runtime_identity: crate::physical_runtime::RuntimeIdentity,
    recovery_allocation: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
    #[cfg(feature = "certification-test-authority")]
    capture_pause: Mutex<Option<std::sync::Arc<capture_pause::ReadRootCapturePause>>>,
    read_protection: std::sync::Arc<crate::physical_runtime::stability::RootProtectionRegistry>,
    blob_claims: std::sync::Arc<blob_claim::BlobClaimRegistry>,
    reclaim: std::sync::Arc<Mutex<Option<reclaim::ReclaimFenceState>>>,
    state: std::sync::Arc<Mutex<PhysicalCurrentRootState>>,
    transition: PhysicalRootPublicationTransitionOwner,
    publication: std::sync::Arc<
        crate::physical_runtime::durability::retention::PhysicalPublicationAdmission,
    >,
    rewrite_growth: Mutex<
        HashMap<
            PhysicalMutationIdentity,
            Vec<crate::physical_runtime::durability::retention::CandidateGrowthLease>,
        >,
    >,
    displaced: Mutex<
        HashMap<
            PhysicalMutationIdentity,
            Vec<crate::physical_runtime::durability::retention::DisplacedArtifact>,
        >,
    >,
}

pub(super) struct PhysicalCurrentRootState {
    current_root: DurablePhysicalRootManifest,
    previous_root: Option<RetainedPhysicalRoot>,
    namespace_evidence: crate::physical_runtime::PhysicalRootNamespaceDurabilityEvidence,
    free_space: DurableFreeSpaceManifestHeader,
    checkpoint_custody: certificate_capacity::CheckpointCustodyState,
    release_ledger: release_capacity::ReleaseLedgerState,
}

pub struct CompletedPhysicalRootPublication {
    group: PhysicalDurabilityGroupBasis,
    member_identities: Box<[PhysicalRootPublicationMemberIdentity]>,
    members: NonEmpty<RootPublicationPhysicalMutationMember>,
    current_root: DurablePhysicalRootManifest,
    current_artifacts: Box<[RecordArtifactFile]>,
    retained_root: RetainedPhysicalRoot,
    root_planning_observation: crate::physical_runtime::RecordRootPlanningObservation,
}

pub enum PhysicalCurrentRootAdvanceOutcome {
    Advanced(CompletedPhysicalRootPublication),
    InspectionRequired(IndeterminatePhysicalCurrentRootAdvance),
}

pub struct IndeterminatePhysicalCurrentRootAdvance {
    durable: RootNamespaceDurablePhysicalMutationMembers,
    cause: PhysicalCurrentRootAdvanceFailureCause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalCurrentRootAdvanceFailureCause {
    PublicationAuthorityReleased,
    CurrentRootMismatch,
    TransitionIdentityMismatch,
    CandidateGenerationMismatch,
    PublicationRetentionMismatch,
    ReclaimFenceMismatch,
}

impl PhysicalCurrentRootOwner {
    pub(in crate::physical_runtime) fn new(
        runtime: &std::sync::Arc<crate::physical_runtime::instance::PhysicalStoreWorkRuntime>,
        checkpoint_custody_origin: CheckpointCustodyOrigin,
        recovered_checkpoint_custody: Option<PreparedRecoveredCheckpointCustody>,
        current_root: DurablePhysicalRootManifest,
        previous_root: Option<DurablePhysicalRootManifest>,
        free_space: DurableFreeSpaceManifestHeader,
        read_protection: std::sync::Arc<crate::physical_runtime::stability::RootProtectionRegistry>,
        publication: std::sync::Arc<
            crate::physical_runtime::durability::PhysicalPublicationAdmission,
        >,
        recovery_allocation: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
    ) -> Self {
        let blob_claim_capacity = read_protection.acquisition_capacity();
        let checkpoint_custody = certificate_capacity::CheckpointCustodyState::from_origin(
            checkpoint_custody_origin,
            &current_root,
        );
        let owner = Self {
            runtime_identity: runtime.submission.runtime_identity(),
            recovery_allocation,
            #[cfg(feature = "certification-test-authority")]
            capture_pause: Mutex::new(None),
            blob_claims: std::sync::Arc::new(blob_claim::BlobClaimRegistry::new(
                blob_claim_capacity,
            )),
            reclaim: std::sync::Arc::new(Mutex::new(None)),
            read_protection,
            state: std::sync::Arc::new(Mutex::new(PhysicalCurrentRootState {
                namespace_evidence:
                    crate::physical_runtime::PhysicalRootNamespaceDurabilityEvidence::ReopenedCurrentRoot {
                        root: current_root.root_cell(),
                    },
                current_root,
                previous_root: previous_root.map(RetainedPhysicalRoot::from_manifest),
                free_space,
                checkpoint_custody,
                release_ledger: release_capacity::ReleaseLedgerState::from_origin(
                    checkpoint_custody_origin,
                ),
            })),
            transition: PhysicalRootPublicationTransitionOwner::new(runtime),
            publication,
            rewrite_growth: Mutex::new(HashMap::new()),
            displaced: Mutex::new(HashMap::new()),
        };
        if let Some(recovered) = recovered_checkpoint_custody {
            owner.install_recovered_checkpoint_custody(recovered);
        }
        owner
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn charged_growth_bytes(&self) -> u64 {
        self.publication.charged_growth_bytes()
    }

    pub(in crate::physical_runtime) fn publication_admission(
        &self,
    ) -> std::sync::Arc<crate::physical_runtime::durability::retention::PhysicalPublicationAdmission>
    {
        std::sync::Arc::clone(&self.publication)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn pending_publication_count(&self) -> usize {
        self.publication.pending_len()
    }

    pub(in crate::physical_runtime) fn hold_rewrite_candidate(
        &self,
        identity: PhysicalMutationIdentity,
        artifact: RecordArtifactFile,
        bytes: u64,
    ) -> Result<(), ()> {
        let lease = self
            .publication
            .reserve_candidate(artifact, bytes)
            .map_err(|_| ())?;
        self.rewrite_growth
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .entry(identity)
            .or_default()
            .push(lease);
        Ok(())
    }

    pub(in crate::physical_runtime) fn install_retention_profile(
        &self,
        profile: crate::physical_runtime::durability::PhysicalRetentionProfile,
    ) {
        self.publication.replace_profile(profile);
    }

    pub(in crate::physical_runtime) fn begin(
        &self,
        identity: PhysicalRootPublicationIdentity,
        source_root: DurablePhysicalRootManifest,
    ) -> Result<PhysicalRootPublicationTransition, PhysicalRootPublicationTransitionDenial> {
        let state = self.lock_publication_state();
        if self.lock_reclaim().is_some() {
            return Err(PhysicalRootPublicationTransitionDenial::ReclaimFenced);
        }
        self.transition
            .begin(identity, &state.current_root, source_root)
    }

    pub(in crate::physical_runtime) fn advance(
        &self,
        durable: RootNamespaceDurablePhysicalMutationMembers,
    ) -> PhysicalCurrentRootAdvanceOutcome {
        let mut state = self.lock_publication_state();
        let mut reclaim = self.lock_reclaim();
        let reclaim_member = if let Some(fence) = reclaim.as_ref() {
            match durable.members() {
                [member]
                    if fence.expected_root == state.current_root.root_cell()
                        && fence.accepts(member.mutation_identity()) =>
                {
                    Some(member.mutation_identity())
                }
                _ => {
                    return PhysicalCurrentRootAdvanceOutcome::InspectionRequired(
                        IndeterminatePhysicalCurrentRootAdvance::new(
                            durable,
                            PhysicalCurrentRootAdvanceFailureCause::ReclaimFenceMismatch,
                        ),
                    );
                }
            }
        } else {
            None
        };
        let cause = validate_advance(&state.current_root, &durable);
        if let Some(cause) = cause {
            return PhysicalCurrentRootAdvanceOutcome::InspectionRequired(
                IndeterminatePhysicalCurrentRootAdvance::new(durable, cause),
            );
        }
        if self.publication.settle_wal_publication(&durable).is_err() {
            return PhysicalCurrentRootAdvanceOutcome::InspectionRequired(
                IndeterminatePhysicalCurrentRootAdvance::new(
                    durable,
                    PhysicalCurrentRootAdvanceFailureCause::PublicationRetentionMismatch,
                ),
            );
        }
        let namespace_evidence =
            crate::physical_runtime::PhysicalRootNamespaceDurabilityEvidence::PublishedCurrentRoot {
                group: durable.group_basis(),
                source_generation: durable.source_root_generation(),
                current_generation: durable.current_root_generation(),
                replacement: durable
                    .replacement_effect_identity()
                    .expect("a namespace-durable root has a replacement effect"),
                namespace_synchronization: durable
                    .namespace_effect_identity()
                    .expect("a namespace-durable root has a namespace synchronization effect"),
            };
        let (core, _replacement, _namespace_synchronization) = durable.into_parts();
        let group = core.group();
        let member_identities = core.members().to_vec().into_boxed_slice();
        let (mut candidate, members) = core.release_transition();
        candidate.commit_arena_reservations();
        let (
            source_root,
            successor_free_space,
            current_root,
            current_artifacts,
            root_planning_observation,
        ) = candidate.into_root_parts();
        let retained_root = RetainedPhysicalRoot::from_manifest(source_root);
        state.current_root = current_root.clone();
        if let (Some(fence), Some(mutation)) = (reclaim.as_mut(), reclaim_member) {
            fence.advance(mutation, current_root.root_cell());
        }
        state.previous_root = Some(retained_root.clone());
        state.namespace_evidence = namespace_evidence;
        state.free_space = successor_free_space;
        PhysicalCurrentRootAdvanceOutcome::Advanced(CompletedPhysicalRootPublication {
            group,
            member_identities,
            members,
            current_root,
            current_artifacts,
            retained_root,
            root_planning_observation,
        })
    }

    fn lock_publication_state(&self) -> std::sync::MutexGuard<'_, PhysicalCurrentRootState> {
        #[cfg(feature = "certification-test-authority")]
        self.observe_publication_lock_wait();
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_reclaim(&self) -> std::sync::MutexGuard<'_, Option<reclaim::ReclaimFenceState>> {
        self.reclaim
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl IndeterminatePhysicalCurrentRootAdvance {
    fn new(
        mut durable: RootNamespaceDurablePhysicalMutationMembers,
        cause: PhysicalCurrentRootAdvanceFailureCause,
    ) -> Self {
        let (mut core, replacement, namespace_synchronization) = durable.into_parts();
        core.require_inspection();
        durable = RootNamespaceDurablePhysicalMutationMembers::new(
            core,
            replacement,
            namespace_synchronization,
        );
        Self { durable, cause }
    }

    #[cfg_attr(not(feature = "certification-test-authority"), allow(dead_code))]
    pub(in crate::physical_runtime) fn publication_authority_released(
        durable: RootNamespaceDurablePhysicalMutationMembers,
    ) -> Self {
        Self::new(
            durable,
            PhysicalCurrentRootAdvanceFailureCause::PublicationAuthorityReleased,
        )
    }

    pub const fn cause(&self) -> PhysicalCurrentRootAdvanceFailureCause {
        self.cause
    }

    pub fn namespace_durable(&self) -> &RootNamespaceDurablePhysicalMutationMembers {
        &self.durable
    }
}

impl CompletedPhysicalRootPublication {
    pub const fn group_basis(&self) -> PhysicalDurabilityGroupBasis {
        self.group
    }

    pub fn members(&self) -> &[PhysicalRootPublicationMemberIdentity] {
        &self.member_identities
    }

    pub fn settled_members(&self) -> &[RootPublicationPhysicalMutationMember] {
        self.members.as_slice()
    }

    pub const fn current_root(&self) -> &DurablePhysicalRootManifest {
        &self.current_root
    }

    pub fn current_artifacts(&self) -> &[RecordArtifactFile] {
        &self.current_artifacts
    }

    pub const fn retained_root(&self) -> &RetainedPhysicalRoot {
        &self.retained_root
    }

    pub const fn root_planning_observation(
        &self,
    ) -> crate::physical_runtime::RecordRootPlanningObservation {
        self.root_planning_observation
    }
}
