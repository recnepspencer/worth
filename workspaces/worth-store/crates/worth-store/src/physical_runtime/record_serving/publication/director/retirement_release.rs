use super::RecordPublicationDirector;
use crate::physical_runtime::durability::{
    encode_retirement, DisplacedArtifact, DurableMaintenanceReceipt, PendingPublicationLease,
    PhysicalRetirementDenial, PhysicalRootPublicationTransition, RetainedByteLease,
    RetirementReleaseProjection, ScheduledMaintenanceDenial,
};
use crate::physical_runtime::record_serving::arena::ArenaAllocationDenial;
use crate::physical_runtime::record_serving::publication::PublicationPlan;
use sha2::{Digest, Sha256};
use std::num::NonZeroU64;
use worth_store_physical_format::{DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest};

mod preparation;
mod publication;
mod resume;

pub(super) struct PendingRetirementRelease {
    displaced: DisplacedArtifact,
    release: RetirementReleaseProjection,
    pending: Option<PendingPublicationLease>,
    intent: Vec<u8>,
    growth: Option<RetainedByteLease>,
    growth_bytes: u64,
    progress: ReleaseProgress,
}

enum ReleaseProgress {
    Recovered(crate::physical_runtime::PhysicalMutationIdentity),
    Prepared(ReleaseCandidate),
    IntentDurable(ReleaseCandidate, DurableMaintenanceReceipt),
    RootPublished { allocator_update: bool },
    CompletionDurable { allocator_update: bool },
    InspectionRequired,
}

struct ReleaseCandidate {
    source: DurablePhysicalRootManifest,
    free: DurableFreeSpaceManifestHeader,
    plan: PublicationPlan,
    transition: PhysicalRootPublicationTransition,
    allocation: worth_store_buffer_pool::ForegroundWriteAllocationGrant,
    retry: Option<crate::physical_runtime::record_serving::RetirementCandidateRetryScope>,
}

impl RecordPublicationDirector {
    pub(super) fn has_pending_extent_release(&self) -> bool {
        self.retirement_release
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some()
    }

    pub(super) fn commit_extent_release_intent(
        &self,
    ) -> Result<DisplacedArtifact, PhysicalRetirementDenial> {
        let mut pending = self
            .retirement_release
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let state = pending.as_mut().ok_or(PhysicalRetirementDenial::Absent)?;
        if let ReleaseProgress::Recovered(operation) = state.progress {
            if let Err(denial) = self.resume_extent_release(state, operation) {
                if denial != PhysicalRetirementDenial::Waiting {
                    state.progress = ReleaseProgress::InspectionRequired;
                    if let Some(runtime) = self.runtime.upgrade() {
                        runtime.health.revoke();
                    }
                }
                return Err(denial);
            }
        }
        if matches!(state.progress, ReleaseProgress::InspectionRequired) {
            return Err(PhysicalRetirementDenial::WalWrite);
        }
        if matches!(state.progress, ReleaseProgress::Prepared(_)) {
            let result = if matches!(&state.progress, ReleaseProgress::Prepared(candidate) if candidate.retry.is_some())
            {
                self.wal
                    .synchronize_retained_maintenance_intent(&state.intent)
            } else {
                self.wal.append_scheduled_maintenance_receipt(&state.intent)
            };
            let receipt = match result {
                Ok(receipt) => receipt,
                Err(denial @ ScheduledMaintenanceDenial::NotStarted(_)) => {
                    return Err(maintenance_denial(denial))
                }
                Err(ScheduledMaintenanceDenial::WrittenAwaitingBarrier { interval }) => {
                    if let Some(growth) = state.growth.take() {
                        growth.seal();
                        self.root_owner
                            .publication_admission()
                            .note_sealed_publication(interval.0, interval.1, state.growth_bytes);
                    }
                    return Err(PhysicalRetirementDenial::Waiting);
                }
                Err(denial) => {
                    if let Some(growth) = state.growth.take() {
                        growth.seal();
                    }
                    state.progress = ReleaseProgress::InspectionRequired;
                    if let Some(runtime) = self.runtime.upgrade() {
                        runtime.health.revoke();
                    }
                    return Err(maintenance_denial(denial));
                }
            };
            if let Some(growth) = state.growth.take() {
                growth.seal();
                let (segment, generation, ..) = receipt.interval();
                self.root_owner
                    .publication_admission()
                    .note_sealed_publication(segment, generation, state.growth_bytes);
            }
            let ReleaseProgress::Prepared(candidate) =
                std::mem::replace(&mut state.progress, ReleaseProgress::InspectionRequired)
            else {
                unreachable!("the retirement mutex owns the exact preparation");
            };
            state.progress = ReleaseProgress::IntentDurable(candidate, receipt);
        }
        Ok(state.displaced)
    }

    pub(super) fn finish_extent_release(
        &self,
        displaced: DisplacedArtifact,
    ) -> Result<(), PhysicalRetirementDenial> {
        let mut pending = self
            .retirement_release
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let state = pending.as_mut().ok_or(PhysicalRetirementDenial::Absent)?;
        if state.displaced != displaced {
            return Err(PhysicalRetirementDenial::Retained);
        }
        let lease = state
            .pending
            .as_ref()
            .ok_or(PhysicalRetirementDenial::Unresolved)?;
        if let Some(denial) = self
            .root_owner
            .blocked_retirement_release(&displaced, lease)
        {
            return Err(denial);
        }
        match std::mem::replace(&mut state.progress, ReleaseProgress::InspectionRequired) {
            ReleaseProgress::IntentDurable(candidate, receipt) => {
                if let Err(denial) = self.publish_extent_release(candidate, receipt) {
                    if let Some(runtime) = self.runtime.upgrade() {
                        runtime.health.revoke();
                    }
                    return Err(denial);
                }
                state.progress = ReleaseProgress::RootPublished {
                    allocator_update: true,
                };
            }
            ReleaseProgress::RootPublished { allocator_update } => {
                state.progress = ReleaseProgress::RootPublished { allocator_update };
            }
            ReleaseProgress::CompletionDurable { allocator_update } => {
                state.progress = ReleaseProgress::CompletionDurable { allocator_update };
            }
            other => {
                state.progress = other;
                return Err(PhysicalRetirementDenial::Unresolved);
            }
        }
        if let ReleaseProgress::RootPublished { allocator_update } = state.progress {
            if let crate::physical_runtime::durability::RetiredArtifact::Arena { arena, .. } =
                displaced.artifact
            {
                self.forget_published_arena(arena)?;
                self.delete_displaced(displaced.artifact)?;
            }
            let completion = encode_retirement(
                displaced.artifact,
                true,
                displaced.source_root,
                displaced.bytes,
                Some(state.release),
            );
            self.wal
                .append_scheduled_maintenance_receipt(&completion)
                .map_err(maintenance_denial)?;
            state.progress = ReleaseProgress::CompletionDurable { allocator_update };
        }
        if matches!(
            state.progress,
            ReleaseProgress::CompletionDurable {
                allocator_update: true
            }
        ) && matches!(
            displaced.artifact,
            crate::physical_runtime::durability::RetiredArtifact::Extent { .. }
        ) {
            self.admit_completed_extent_release(displaced)?;
        }
        self.root_owner.complete_displaced(displaced.artifact);
        // Dropping this lease admits ordinary publication only after completion
        // and the allocator's corresponding durable release have both settled.
        state.pending.take();
        pending.take();
        Ok(())
    }

    fn admit_completed_extent_release(
        &self,
        displaced: DisplacedArtifact,
    ) -> Result<(), PhysicalRetirementDenial> {
        let crate::physical_runtime::durability::RetiredArtifact::Extent { range, .. } =
            displaced.artifact
        else {
            return Err(PhysicalRetirementDenial::Retained);
        };
        self.residency
            .invalidate_released_arena_range(range)
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        let preparation = self.preparation.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(owner) = &preparation.arenas {
            owner
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .admit_durable_release(range)
                .map_err(arena_release_denial)?;
        }
        Ok(())
    }
}

impl Drop for PendingRetirementRelease {
    fn drop(&mut self) {
        if let Some(lease) = self.pending.take() {
            lease.retain_unresolved();
        }
    }
}

fn maintenance_denial(denial: ScheduledMaintenanceDenial) -> PhysicalRetirementDenial {
    match denial {
        ScheduledMaintenanceDenial::NotStarted(_) => PhysicalRetirementDenial::Waiting,
        ScheduledMaintenanceDenial::WrittenAwaitingBarrier { .. } => {
            PhysicalRetirementDenial::Waiting
        }
        ScheduledMaintenanceDenial::Write => PhysicalRetirementDenial::WalWrite,
        ScheduledMaintenanceDenial::Sync => PhysicalRetirementDenial::WalSync,
        ScheduledMaintenanceDenial::Finish => PhysicalRetirementDenial::WalFinish,
    }
}

fn arena_release_denial(denial: ArenaAllocationDenial) -> PhysicalRetirementDenial {
    match denial {
        ArenaAllocationDenial::RangeBudget { .. } => PhysicalRetirementDenial::ArenaIndexCapacity,
        ArenaAllocationDenial::Capacity => PhysicalRetirementDenial::ArenaCapacity,
        ArenaAllocationDenial::InvalidGeometry
        | ArenaAllocationDenial::Overlap
        | ArenaAllocationDenial::StaleReservation => PhysicalRetirementDenial::ArenaReleaseInvalid,
        ArenaAllocationDenial::EvacuationBusy => PhysicalRetirementDenial::Waiting,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocator_index_pressure_is_not_a_scheduler_wait_or_invalid_range() {
        assert_eq!(
            arena_release_denial(ArenaAllocationDenial::RangeBudget {
                required: 4,
                maximum: 3,
            }),
            PhysicalRetirementDenial::ArenaIndexCapacity
        );
        assert_eq!(
            arena_release_denial(ArenaAllocationDenial::Capacity),
            PhysicalRetirementDenial::ArenaCapacity
        );
        for invalid in [
            ArenaAllocationDenial::InvalidGeometry,
            ArenaAllocationDenial::Overlap,
            ArenaAllocationDenial::StaleReservation,
        ] {
            assert_eq!(
                arena_release_denial(invalid),
                PhysicalRetirementDenial::ArenaReleaseInvalid
            );
        }
        assert_eq!(
            arena_release_denial(ArenaAllocationDenial::EvacuationBusy),
            PhysicalRetirementDenial::Waiting
        );
    }
}
