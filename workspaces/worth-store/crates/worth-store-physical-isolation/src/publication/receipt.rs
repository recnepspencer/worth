use super::{
    AtomicPhysicalRootSwap, CopyOnWritePublicationBinding, CrashStableFreeReusePosture,
    OldReachabilityPreservation, PhysicalPublicationFoundationalEvidence,
    PhysicalPublicationReadiness, PublicationEpochPair, ReleasedOldReachability,
    RootSwapOrderingContract,
};
use crate::{CurrentPhysicalRoot, PhysicalReadPlanReleaseReceipt};
use worth_store_physical_backend::StorageBoundaryExecutionIdentity;
use worth_store_physical_format::RootPublicationValidationWitness;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalPublicationReleasePosture {
    OldReachabilityRetainedUntilReadRelease,
    IdentityReuseProtectedByAllocatorFence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalPublicationCounterSnapshot {
    intent_validations: u64,
    old_reachability_checks: u64,
    epoch_checks: u64,
    ordering_checks: u64,
    readiness_joins: u64,
    root_swaps: u64,
    denied_in_place_overwrites: u64,
    denied_stale_epochs: u64,
    denied_weak_orderings: u64,
    denied_identity_reuse: u64,
    mixed_tree_denials: u64,
}

#[derive(Debug, Clone)]
pub struct PhysicalPublicationReceipt {
    old_root: CurrentPhysicalRoot,
    new_root: CurrentPhysicalRoot,
    old_root_validation: RootPublicationValidationWitness,
    new_root_validation: RootPublicationValidationWitness,
    epochs: PublicationEpochPair,
    old_reachability: OldReachabilityPreservation,
    ordering: RootSwapOrderingContract,
    release_posture: PhysicalPublicationReleasePosture,
    free_reuse: Option<CrashStableFreeReusePosture>,
    counters: PhysicalPublicationCounterSnapshot,
    storage_boundary_execution: Option<StorageBoundaryExecutionIdentity>,
}

impl PhysicalPublicationCounterSnapshot {
    pub(crate) const fn for_validated_lowering() -> Self {
        Self {
            intent_validations: 1,
            old_reachability_checks: 1,
            epoch_checks: 1,
            ordering_checks: 1,
            readiness_joins: 0,
            root_swaps: 0,
            denied_in_place_overwrites: 0,
            denied_stale_epochs: 0,
            denied_weak_orderings: 0,
            denied_identity_reuse: 0,
            mixed_tree_denials: 0,
        }
    }

    pub(crate) const fn for_completed_publication() -> Self {
        Self {
            intent_validations: 1,
            old_reachability_checks: 1,
            epoch_checks: 1,
            ordering_checks: 1,
            readiness_joins: 1,
            root_swaps: 1,
            denied_in_place_overwrites: 0,
            denied_stale_epochs: 0,
            denied_weak_orderings: 0,
            denied_identity_reuse: 0,
            mixed_tree_denials: 0,
        }
    }

    pub const fn intent_validations(self) -> u64 {
        self.intent_validations
    }

    pub const fn old_reachability_checks(self) -> u64 {
        self.old_reachability_checks
    }

    pub const fn epoch_checks(self) -> u64 {
        self.epoch_checks
    }

    pub const fn ordering_checks(self) -> u64 {
        self.ordering_checks
    }

    pub const fn readiness_joins(self) -> u64 {
        self.readiness_joins
    }

    pub const fn root_swaps(self) -> u64 {
        self.root_swaps
    }

    pub const fn denied_in_place_overwrites(self) -> u64 {
        self.denied_in_place_overwrites
    }

    pub const fn denied_stale_epochs(self) -> u64 {
        self.denied_stale_epochs
    }

    pub const fn denied_weak_orderings(self) -> u64 {
        self.denied_weak_orderings
    }

    pub const fn denied_identity_reuse(self) -> u64 {
        self.denied_identity_reuse
    }

    pub const fn mixed_tree_denials(self) -> u64 {
        self.mixed_tree_denials
    }
}

impl PhysicalPublicationReceipt {
    pub(crate) fn from_completed_plan(
        binding: CopyOnWritePublicationBinding,
        readiness: PhysicalPublicationReadiness,
        atomic_swap: AtomicPhysicalRootSwap,
    ) -> Self {
        let release_posture = if readiness.free_reuse().is_some() {
            PhysicalPublicationReleasePosture::IdentityReuseProtectedByAllocatorFence
        } else {
            PhysicalPublicationReleasePosture::OldReachabilityRetainedUntilReadRelease
        };
        Self {
            old_root: binding.old_root(),
            new_root: binding.new_root(),
            old_root_validation: binding.old_root_validation(),
            new_root_validation: binding.new_root_validation(),
            epochs: readiness.epochs().epochs(),
            old_reachability: readiness.old_reachability(),
            ordering: atomic_swap.ordering(),
            release_posture,
            free_reuse: readiness.free_reuse(),
            counters: PhysicalPublicationCounterSnapshot::for_completed_publication(),
            storage_boundary_execution: None,
        }
    }
}

impl PhysicalPublicationReceipt {
    pub const fn old_root(&self) -> CurrentPhysicalRoot {
        self.old_root
    }

    pub const fn new_root(&self) -> CurrentPhysicalRoot {
        self.new_root
    }

    pub const fn old_root_validation(&self) -> RootPublicationValidationWitness {
        self.old_root_validation
    }

    pub const fn new_root_validation(&self) -> RootPublicationValidationWitness {
        self.new_root_validation
    }

    pub const fn epochs(&self) -> PublicationEpochPair {
        self.epochs
    }

    pub const fn old_reachability(&self) -> OldReachabilityPreservation {
        self.old_reachability
    }

    pub const fn ordering(&self) -> RootSwapOrderingContract {
        self.ordering
    }

    pub const fn release_posture(&self) -> PhysicalPublicationReleasePosture {
        self.release_posture
    }

    pub const fn free_reuse(&self) -> Option<CrashStableFreeReusePosture> {
        self.free_reuse
    }

    pub const fn counters(&self) -> PhysicalPublicationCounterSnapshot {
        self.counters
    }

    pub const fn storage_boundary_execution_identity(
        &self,
    ) -> Option<StorageBoundaryExecutionIdentity> {
        self.storage_boundary_execution
    }

    pub fn lower_to_foundational_evidence(
        &self,
    ) -> Result<
        PhysicalPublicationFoundationalEvidence,
        worth_foundational::FoundationalBoundaryEvidenceProvenanceConstructionDenial,
    > {
        PhysicalPublicationFoundationalEvidence::lower(self)
    }

    pub fn admit_old_reachability_release(
        &self,
        release_receipt: PhysicalReadPlanReleaseReceipt,
    ) -> Result<ReleasedOldReachability, super::PhysicalPublicationDenial> {
        self.old_reachability.admit_release(release_receipt)
    }
}
