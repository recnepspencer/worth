//! One root-only, WAL-bound activation of deterministic arena tier custody.
//! An uncertain effect quarantines the cached arena owner until reopen.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    tier_epoch_anchor, RecordArtifactFile, TierEpochActivationV1, TierEpochWalFrameWitnessV1,
};

use super::RecordPublicationDirector;
use crate::physical_runtime::{
    durability::{
        publish_tier_epoch_candidate, DurableMaintenanceReceipt, ScheduledMaintenanceDenial,
    },
    record_serving::{
        arena::ArenaTierEpochFence, planning::rebased_root::plan_tier_epoch_activation,
        publication::write_root_candidate_artifacts,
        residency::publication_artifacts::PublicationRecordArtifacts,
        PreparedPhysicalRootCandidate,
    },
    AdmittedRecordPlacementPolicy, PhysicalMutationIdentity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum TierEpochActivationFailure {
    Waiting,
    InvalidSelectedRoot,
    WalPlan,
    WalWrite,
    WalSync,
    WalFinish,
    Publication,
    Unresolved,
}

impl RecordPublicationDirector {
    /// Returns the durable arena frontier, not the successor root generation.
    pub(in crate::physical_runtime) fn activate_tier_epoch(
        &self,
        placement: AdmittedRecordPlacementPolicy,
    ) -> Result<u64, TierEpochActivationFailure> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(TierEpochActivationFailure::Unresolved)?;
        let reserved = self
            .mutation_identity
            .reserve_mutation_identity()
            .map_err(|_| TierEpochActivationFailure::Waiting)?;
        let operation = PhysicalMutationIdentity::from_reserved_operation(reserved.identity());
        let mut pending = Some(
            self.root_owner
                .register_tier_epoch_pending(operation)
                .map_err(|_| TierEpochActivationFailure::Waiting)?,
        );
        let (source, free) = self.root_owner.snapshot();
        if free.tier_epoch_start().is_some() || source.tier_epoch_anchor().is_some() {
            return Err(TierEpochActivationFailure::InvalidSelectedRoot);
        }
        let format = self.format.declaration();
        let page_bytes = u64::from(format.page_size().bytes());
        let bytes = std::num::NonZeroU64::new(page_bytes.saturating_mul(32))
            .ok_or(TierEpochActivationFailure::Waiting)?;
        let allocation = self
            .residency
            .begin_foreground_write_operation(bytes)
            .map_err(|_| TierEpochActivationFailure::Waiting)?;
        let candidate = RecordArtifactFile::CatalogCandidate {
            publication: super::super::append::next_nonzero_random()
                .map_err(|_| TierEpochActivationFailure::WalPlan)?,
        };
        let attempt = [
            operation.operation_identity().get().to_le_bytes(),
            operation.lifecycle_generation().to_le_bytes(),
        ]
        .concat()
        .try_into()
        .map_err(|_| TierEpochActivationFailure::WalPlan)?;
        let source_root_sha256 = Sha256::digest(source.encode(format)).into();
        let source_free_sha256 = Sha256::digest(free.encode(format)).into();
        let anchor = tier_epoch_anchor(
            runtime
                .executor
                .record_serving_media()
                .store_identity()
                .bytes(),
            attempt,
            source.generation(),
            source_root_sha256,
            source_free_sha256,
            free.next_arena(),
        )
        .ok_or(TierEpochActivationFailure::WalPlan)?;
        let (plan, successor_free) = plan_tier_epoch_activation(
            &source,
            &free,
            self.format,
            runtime.executor.record_serving_media(),
            candidate,
            free.next_arena(),
            anchor,
        )
        .map_err(|_| TierEpochActivationFailure::WalPlan)?;
        let growth_bytes = plan
            .retained_metadata_bytes()
            .ok_or(TierEpochActivationFailure::WalPlan)?;
        let intent = TierEpochActivationV1::intent(
            runtime
                .executor
                .record_serving_media()
                .store_identity()
                .bytes(),
            attempt,
            source.generation(),
            source_root_sha256,
            source_free_sha256,
            free.next_arena(),
            plan.generation,
            Sha256::digest(&plan.root_bytes).into(),
            growth_bytes,
            match candidate {
                RecordArtifactFile::CatalogCandidate { publication } => publication,
                _ => unreachable!(),
            },
        )
        .map_err(|_| TierEpochActivationFailure::WalPlan)?;
        let identity =
            crate::physical_runtime::durability::PhysicalRootPublicationIdentity::from_tier_epoch(
                self.durability.policy_identity(),
                operation,
                intent,
            )
            .ok_or(TierEpochActivationFailure::WalPlan)?;
        let mut transition = self
            .root_owner
            .begin_tier_epoch_root(identity, source.clone(), format)
            .map_err(|_| TierEpochActivationFailure::Waiting)?;
        let owner = self
            .arena_allocation_owner(&allocation, placement, &free)
            .map_err(|_| TierEpochActivationFailure::Waiting)?;
        let mut fence = ArenaTierEpochFence::begin(&owner, free.next_arena())
            .map_err(|_| TierEpochActivationFailure::Waiting)?;
        let growth = self
            .root_owner
            .publication_admission()
            .reserve_retained_bytes(growth_bytes)
            .map_err(|_| TierEpochActivationFailure::Waiting)?;
        fence.mark_effect_may_exist();
        let receipt = match self
            .wal
            .append_scheduled_maintenance_receipt(&intent.encode())
        {
            Ok(receipt) => receipt,
            Err(ScheduledMaintenanceDenial::NotStarted(_)) => {
                fence.prove_no_effect();
                return Err(TierEpochActivationFailure::Waiting);
            }
            Err(denial) => {
                growth.seal();
                transition.require_inspection();
                pending
                    .take()
                    .expect("exclusive pending")
                    .retain_unresolved();
                runtime.health.revoke();
                return Err(wal_denial(denial));
            }
        };
        let Some(intent_frame) = receipt_frame_witness(&receipt, &intent.encode()) else {
            growth.seal();
            transition.require_inspection();
            pending
                .take()
                .expect("exclusive pending")
                .retain_unresolved();
            runtime.health.revoke();
            return Err(TierEpochActivationFailure::WalFinish);
        };
        growth.seal();
        transition.mark_effect_started();
        let (segment, generation, ..) = receipt.interval();
        self.root_owner
            .publication_admission()
            .note_sealed_publication(segment, generation, growth_bytes);
        let result = self.publish_tier_epoch_root(
            source,
            successor_free,
            plan,
            transition,
            receipt,
            &allocation,
        );
        if let Err(denial) = result {
            pending
                .take()
                .expect("exclusive pending")
                .retain_unresolved();
            runtime.health.revoke();
            return Err(denial);
        }
        let completed_receipt = match self
            .wal
            .append_scheduled_maintenance_receipt(&intent.completed().encode())
        {
            Ok(receipt) => receipt,
            Err(denial) => {
                pending
                    .take()
                    .expect("exclusive pending")
                    .retain_unresolved();
                runtime.health.revoke();
                return Err(wal_denial(denial));
            }
        };
        let sealed = receipt_frame_witness(&completed_receipt, &intent.completed().encode())
            .ok_or(TierEpochActivationFailure::WalFinish)
            .and_then(|completed_frame| {
                self.root_owner
                    .seal_tier_epoch_checkpoint_basis(
                        intent,
                        intent_frame,
                        completed_frame,
                        self.format.declaration(),
                    )
                    .map_err(|_| TierEpochActivationFailure::Unresolved)
            });
        if let Err(denial) = sealed {
            pending
                .take()
                .expect("exclusive pending")
                .retain_unresolved();
            runtime.health.revoke();
            return Err(denial);
        }
        if fence.complete().is_err() {
            pending
                .take()
                .expect("exclusive pending")
                .retain_unresolved();
            runtime.health.revoke();
            return Err(TierEpochActivationFailure::Unresolved);
        }
        pending.take();
        Ok(intent.tier_epoch_start())
    }

    fn publish_tier_epoch_root(
        &self,
        source: worth_store_physical_format::DurablePhysicalRootManifest,
        free: worth_store_physical_format::DurableFreeSpaceManifestHeader,
        plan: crate::physical_runtime::record_serving::publication::PublicationPlan,
        mut transition: crate::physical_runtime::durability::PhysicalRootPublicationTransition,
        receipt: DurableMaintenanceReceipt,
        allocation: &worth_store_buffer_pool::ForegroundWriteAllocationGrant,
    ) -> Result<(), TierEpochActivationFailure> {
        let frames = plan
            .root_candidate_frame_set()
            .map_err(|_| TierEpochActivationFailure::WalPlan)?;
        let mut residency = self
            .residency
            .begin_candidate_publication(allocation, frames)
            .map_err(|_| TierEpochActivationFailure::Waiting)?;
        transition.mark_effect_started();
        let artifacts = PublicationRecordArtifacts::new(&self.mutation);
        let written = write_root_candidate_artifacts(&artifacts, plan, &mut residency)
            .map_err(|_| TierEpochActivationFailure::Publication)?;
        residency
            .require_complete()
            .map_err(|_| TierEpochActivationFailure::Publication)?;
        drop(residency);
        let candidate =
            PreparedPhysicalRootCandidate::new(source, free, written.plan, written.artifacts);
        let durable = publish_tier_epoch_candidate(candidate, transition, receipt, &self.root_work)
            .map_err(|_| TierEpochActivationFailure::Publication)?;
        self.root_owner
            .advance_tier_epoch_root(durable, self.format.declaration())
            .map_err(|_| TierEpochActivationFailure::Publication)
    }
}

fn receipt_frame_witness(
    receipt: &DurableMaintenanceReceipt,
    payload: &[u8],
) -> Option<TierEpochWalFrameWitnessV1> {
    let payload_digest: [u8; 32] = Sha256::digest(payload).into();
    let (identity_digest, frame_payload_digest) = receipt.frame_digests()?;
    if receipt.payload_digest() != payload_digest || frame_payload_digest != payload_digest {
        return None;
    }
    let (_, _, start, end, _, _) = receipt.interval();
    TierEpochWalFrameWitnessV1::new(start, end, identity_digest, frame_payload_digest)
}

fn wal_denial(denial: ScheduledMaintenanceDenial) -> TierEpochActivationFailure {
    match denial {
        ScheduledMaintenanceDenial::NotStarted(_)
        | ScheduledMaintenanceDenial::WrittenAwaitingBarrier { .. } => {
            TierEpochActivationFailure::Waiting
        }
        ScheduledMaintenanceDenial::Write => TierEpochActivationFailure::WalWrite,
        ScheduledMaintenanceDenial::Sync => TierEpochActivationFailure::WalSync,
        ScheduledMaintenanceDenial::Finish => TierEpochActivationFailure::WalFinish,
    }
}
