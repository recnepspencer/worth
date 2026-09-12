use std::sync::Mutex;

#[cfg(feature = "certification-test-authority")]
mod capture_pause;
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
    PhysicalDurabilityGroupBasis, PhysicalRootPublicationMemberIdentity,
    RootNamespaceDurablePhysicalMutationMembers, RootPublicationPhysicalMutationMember,
};

pub(in crate::physical_runtime) struct PhysicalCurrentRootOwner {
    #[cfg(feature = "certification-test-authority")]
    capture_pause: Mutex<Option<std::sync::Arc<capture_pause::ReadRootCapturePause>>>,
    read_protection: std::sync::Arc<crate::physical_runtime::stability::RootProtectionRegistry>,
    state: Mutex<PhysicalCurrentRootState>,
    transition: PhysicalRootPublicationTransitionOwner,
}

struct PhysicalCurrentRootState {
    current_root: DurablePhysicalRootManifest,
    previous_root: Option<RetainedPhysicalRoot>,
    namespace_evidence: crate::physical_runtime::PhysicalRootNamespaceDurabilityEvidence,
    free_space: DurableFreeSpaceManifestHeader,
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
}

impl PhysicalCurrentRootOwner {
    pub(in crate::physical_runtime) fn new(
        runtime: &std::sync::Arc<crate::physical_runtime::instance::PhysicalStoreWorkRuntime>,
        current_root: DurablePhysicalRootManifest,
        previous_root: Option<DurablePhysicalRootManifest>,
        free_space: DurableFreeSpaceManifestHeader,
        read_protection: std::sync::Arc<crate::physical_runtime::stability::RootProtectionRegistry>,
    ) -> Self {
        Self {
            #[cfg(feature = "certification-test-authority")]
            capture_pause: Mutex::new(None),
            read_protection,
            state: Mutex::new(PhysicalCurrentRootState {
                namespace_evidence:
                    crate::physical_runtime::PhysicalRootNamespaceDurabilityEvidence::ReopenedCurrentRoot {
                        root: current_root.root_cell(),
                    },
                current_root,
                previous_root: previous_root.map(RetainedPhysicalRoot::from_manifest),
                free_space,
            }),
            transition: PhysicalRootPublicationTransitionOwner::new(runtime),
        }
    }

    pub(in crate::physical_runtime) fn snapshot(
        &self,
    ) -> (DurablePhysicalRootManifest, DurableFreeSpaceManifestHeader) {
        let state = self.lock_publication_state();
        (state.current_root.clone(), state.free_space.clone())
    }

    pub(in crate::physical_runtime) fn capture_read_root(
        &self,
    ) -> Result<
        (
            DurablePhysicalRootManifest,
            crate::physical_runtime::stability::PhysicalRootReadLease,
        ),
        crate::physical_runtime::PhysicalReadProtectionDenial,
    > {
        #[cfg(feature = "certification-test-authority")]
        self.pause_capture_at(CertificationReadRootCaptureStage::BeforeRootLock);
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let root = state.current_root.clone();
        #[cfg(feature = "certification-test-authority")]
        self.pause_capture_at(
            CertificationReadRootCaptureStage::AfterObservationBeforeRegistration,
        );
        let lease = self.read_protection.capture(&root)?;
        drop(state);
        Ok((root, lease))
    }

    pub(in crate::physical_runtime) fn begin(
        &self,
        identity: PhysicalRootPublicationIdentity,
        source_root: DurablePhysicalRootManifest,
    ) -> Result<PhysicalRootPublicationTransition, PhysicalRootPublicationTransitionDenial> {
        let state = self.lock_publication_state();
        self.transition
            .begin(identity, &state.current_root, source_root)
    }

    pub(in crate::physical_runtime) fn advance(
        &self,
        durable: RootNamespaceDurablePhysicalMutationMembers,
    ) -> PhysicalCurrentRootAdvanceOutcome {
        let mut state = self.lock_publication_state();
        let cause = validate_advance(&state.current_root, &durable);
        if let Some(cause) = cause {
            return PhysicalCurrentRootAdvanceOutcome::InspectionRequired(
                IndeterminatePhysicalCurrentRootAdvance::new(durable, cause),
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
        let (candidate, members) = core.release_transition();
        let (
            source_root,
            successor_free_space,
            current_root,
            current_artifacts,
            root_planning_observation,
        ) = candidate.into_root_parts();
        let retained_root = RetainedPhysicalRoot::from_manifest(source_root);
        state.current_root = current_root.clone();
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

    pub(in crate::physical_runtime) fn into_recovery_root_basis(
        self,
    ) -> crate::physical_runtime::PhysicalRecoveryRootBasis {
        let state = self
            .state
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        crate::physical_runtime::PhysicalRecoveryRootBasis::new(
            state.current_root,
            state.previous_root,
            state.namespace_evidence,
        )
    }

    fn lock_publication_state(&self) -> std::sync::MutexGuard<'_, PhysicalCurrentRootState> {
        #[cfg(feature = "certification-test-authority")]
        self.observe_publication_lock_wait();
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn validate_advance(
    current_root: &DurablePhysicalRootManifest,
    durable: &RootNamespaceDurablePhysicalMutationMembers,
) -> Option<PhysicalCurrentRootAdvanceFailureCause> {
    if current_root != durable.source_root() {
        return Some(PhysicalCurrentRootAdvanceFailureCause::CurrentRootMismatch);
    }
    if !durable.transition_matches() {
        return Some(PhysicalCurrentRootAdvanceFailureCause::TransitionIdentityMismatch);
    }
    let identity = durable.identity();
    let group = durable.group_basis();
    if identity.group() != group.identity()
        || identity.membership() != group.membership_digest()
        || identity.member_count() != group.member_count().get()
        || usize::try_from(identity.member_count()).ok() != Some(durable.members().len())
    {
        return Some(PhysicalCurrentRootAdvanceFailureCause::TransitionIdentityMismatch);
    }
    if identity.source_generation() != current_root.generation()
        || identity.candidate_generation() != durable.current_root_generation()
    {
        return Some(PhysicalCurrentRootAdvanceFailureCause::TransitionIdentityMismatch);
    }
    if current_root.generation().checked_add(1) != Some(durable.current_root_generation()) {
        return Some(PhysicalCurrentRootAdvanceFailureCause::CandidateGenerationMismatch);
    }
    None
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
