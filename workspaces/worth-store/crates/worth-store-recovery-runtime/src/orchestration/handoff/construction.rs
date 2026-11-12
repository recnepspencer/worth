//! Dispatch the selected recovery family before entering its owning construction frame.
use worth_store::physical_runtime::{
    AbsentCheckpointWitness, AdmittedRecoveryFilesystemMedia, ClosedPhysicalRecoveryCleanup,
    ConfiguredPhysicalDurabilityDeclaration, PhysicalRecoveryAllocationAdmission,
    PhysicalRecoveryCoordination, RecoveredPhysicalRuntimeConstructionDenial,
    RecoveredPhysicalRuntimeCore,
};
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedOrderedHistoricalReleaseCustody,
    VerifiedPendingWalReleaseCustody, VerifiedSelectedNoReleaseCustody,
    VerifiedSelectedReleaseHeadCustodyV2, VerifiedSelectedTierEpochCustody,
};
mod no_checkpoint;
mod no_release;
mod ordered_release;
mod pending_release;
mod source_heads;

pub(super) struct RejoinConstruction {
    pub(super) coordination: PhysicalRecoveryCoordination,
    pub(super) media: AdmittedRecoveryFilesystemMedia,
    pub(super) closed_cleanup: ClosedPhysicalRecoveryCleanup,
}
impl RejoinConstruction {
    pub(super) fn complete(
        self,
        custody: crate::progression::CustodyState,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        allocation: Option<PhysicalRecoveryAllocationAdmission>,
        absent: Option<AbsentCheckpointWitness>,
        durability: Option<ConfiguredPhysicalDurabilityDeclaration>,
        selection: &crate::orchestration::ResidentSourceSelection,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        match custody {
            crate::progression::CustodyState::SourceHeads(claim) => {
                self.source_heads(claim, tier, allocation, selection)
            }
            crate::progression::CustodyState::NoRelease(claim) => match tier {
                Some(tier) => self.tier_no_release(claim, tier),
                None => self.no_release(claim),
            },
            crate::progression::CustodyState::Pending {
                claim,
                replay,
                effective_heads,
            } => {
                drop(replay);
                match tier {
                    Some(tier) => self.tier_pending_release(
                        claim,
                        effective_heads,
                        tier,
                        allocation,
                        selection,
                    ),
                    None => self.pending_release(claim, effective_heads, allocation, selection),
                }
            }
            crate::progression::CustodyState::OrderedCompleted {
                claim,
                effective_heads,
            } => self.ordered_release(claim, effective_heads, tier, allocation),
            crate::progression::CustodyState::NoCheckpoint => match tier {
                None => self.no_checkpoint(absent, durability),
                Some(_tier) => {
                    Err(RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch)
                }
            },
        }
    }
}

/// Constructor-only facts; later publication metadata remains in admitted backing.
pub(super) struct RejoinConstructionInput {
    pub(super) construction: RejoinConstruction,
    pub(super) custody: crate::progression::CustodyState,
    pub(super) tier: Option<VerifiedSelectedTierEpochCustody>,
    pub(super) allocation: Option<PhysicalRecoveryAllocationAdmission>,
    pub(super) absent: Option<AbsentCheckpointWitness>,
    pub(super) durability: Option<ConfiguredPhysicalDurabilityDeclaration>,
}
