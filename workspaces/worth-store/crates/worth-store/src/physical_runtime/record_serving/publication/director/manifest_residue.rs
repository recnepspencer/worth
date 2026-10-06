//! One effect-bearing maintenance operation for an already admitted orphan
//! DropSetManifest. The ordinary C5 mutation/group path is not entered.

use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobManifestResidueCleanup, BlobManifestResidueCleanupV1, BlobManifestResidueCleanupV2,
    BlobReclaimDescriptorV1, RecordArtifactFile,
};

use super::RecordPublicationDirector;
use crate::physical_runtime::{
    durability::{
        publish_manifest_residue_candidate, AdmittedManifestResidueRetirement,
        DurableMaintenanceReceipt, ManifestResidueDisplacement, ManifestResidueProof,
        PhysicalRootPublicationIdentity, ScheduledMaintenanceDenial, SelectedOriginalDropProof,
    },
    record_serving::{
        planning::rebased_root::{plan_manifest_residue_cleanup, RetirementRootPlanningContext},
        publication::write_root_candidate_artifacts,
        residency::publication_artifacts::PublicationRecordArtifacts,
        PreparedPhysicalRootCandidate,
    },
    AdmittedRecordPlacementPolicy, PhysicalMutationIdentity, PhysicalRetirementDenial,
};

mod admission;
mod candidate;

impl RecordPublicationDirector {
    /// Publishes only the root/routing change. The manifest's old extent is
    /// retained as displaced garbage after namespace durability, and the
    /// original payload identities never enter the removal plan.
    pub(in crate::physical_runtime) fn publish_manifest_residue_cleanup(
        &self,
        admitted: AdmittedManifestResidueRetirement,
    ) -> Result<(u64, ManifestResidueDisplacement), PhysicalRetirementDenial> {
        let displaced = admitted.displaced_manifest();
        let source_generation = admitted.source_root_generation();
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(PhysicalRetirementDenial::Unresolved)?;
        let reserved = self
            .mutation_identity
            .reserve_mutation_identity()
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        let operation = PhysicalMutationIdentity::from_reserved_operation(reserved.identity());
        let mut pending = Some(
            self.root_owner
                .register_manifest_residue_pending(&admitted, operation)
                .map_err(|_| PhysicalRetirementDenial::Waiting)?,
        );
        let (source, free) = self.root_owner.snapshot();
        let height = source
            .routing_root()
            .map_or(0, |root| u64::from(root.level()));
        let bytes = u64::from(self.format.declaration().page_size().bytes())
            .checked_mul(6 * (height + 1) + 8)
            .and_then(NonZeroU64::new)
            .ok_or(PhysicalRetirementDenial::Waiting)?;
        let allocation = self
            .residency
            .begin_foreground_write_operation(bytes)
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        let publication = super::super::append::next_nonzero_random()
            .map_err(|_| PhysicalRetirementDenial::WalPlan)?;
        let candidate = RecordArtifactFile::CatalogCandidate { publication };
        let context = RetirementRootPlanningContext {
            allocation: &allocation,
            residency: self.residency.clone(),
            format: self.format,
            access: self.access,
            media: runtime.executor.record_serving_media(),
            current_root: &source,
            current_free: &free,
        };
        let removed = admitted.removed_records();
        let records = [
            removed[0].ok_or(PhysicalRetirementDenial::WalPlan)?,
            removed[1].unwrap_or(removed[0].ok_or(PhysicalRetirementDenial::WalPlan)?),
        ];
        let removed = &records[..usize::from(removed[1].is_some()) + 1];
        let (plan, successor_free) = plan_manifest_residue_cleanup(context, removed, candidate)
            .map_err(|_| PhysicalRetirementDenial::WalPlan)?;
        let growth_bytes = plan
            .retained_metadata_bytes()
            .ok_or(PhysicalRetirementDenial::WalPlan)?;
        let proof = admitted.proof();
        let store = runtime
            .executor
            .record_serving_media()
            .store_identity()
            .bytes();
        let intent = match (proof.wire_proof(), proof.reserved()) {
            (
                worth_store_physical_format::OriginalDropProofV1::ProvenNoEffect {
                    idempotency,
                    fingerprint,
                    reserved: None,
                },
                None,
            ) => BlobManifestResidueCleanup::V1(
                BlobManifestResidueCleanupV1::intent(
                    store,
                    proof.attempt(),
                    admitted.manifest_record(),
                    admitted.manifest_sha256(),
                    admitted.source_basis_digest(store),
                    idempotency,
                    fingerprint,
                    source.generation(),
                    plan.generation,
                    Sha256::digest(&plan.root_bytes).into(),
                    growth_bytes,
                    publication,
                )
                .map_err(|_| PhysicalRetirementDenial::WalPlan)?,
            ),
            (wire, _) => BlobManifestResidueCleanup::V2(
                BlobManifestResidueCleanupV2::intent(
                    store,
                    proof.attempt(),
                    admitted.manifest_record(),
                    admitted.manifest_sha256(),
                    admitted.source_basis_digest(store),
                    wire,
                    source.generation(),
                    plan.generation,
                    Sha256::digest(&plan.root_bytes).into(),
                    growth_bytes,
                    publication,
                )
                .map_err(|_| PhysicalRetirementDenial::WalPlan)?,
            ),
        };
        let identity = PhysicalRootPublicationIdentity::from_manifest_residue(
            self.durability.policy_identity(),
            operation,
            intent,
        )
        .ok_or(PhysicalRetirementDenial::WalPlan)?;
        let mut transition = self
            .root_owner
            .begin_manifest_residue(&admitted, identity, source.clone())
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        let growth = self
            .root_owner
            .publication_admission()
            .reserve_retained_bytes(growth_bytes)
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        let receipt = match self
            .wal
            .append_scheduled_maintenance_receipt(&intent.encode())
        {
            Ok(receipt) => receipt,
            Err(ScheduledMaintenanceDenial::NotStarted(_)) => {
                return Err(PhysicalRetirementDenial::Waiting);
            }
            Err(denial) => {
                growth.seal();
                admitted.mark_effect_started();
                transition.require_inspection();
                pending
                    .take()
                    .expect("registered lease")
                    .retain_unresolved();
                runtime.health.revoke();
                return Err(maintenance_denial(denial));
            }
        };
        growth.seal();
        let (segment, generation, ..) = receipt.interval();
        self.root_owner
            .publication_admission()
            .note_sealed_publication(segment, generation, growth_bytes);
        // The Intent is WAL-durable and the selected root is unchanged.
        // Certification can kill here to prove C8 replays the typed Intent.
        self.mutations.reach_checkpoint(
            crate::physical_runtime::durability::PhysicalMutationCheckpoint::AfterWalDurability,
        );
        if !admitted.mark_effect_started() {
            transition.require_inspection();
            pending
                .take()
                .expect("registered lease")
                .retain_unresolved();
            runtime.health.revoke();
            return Err(PhysicalRetirementDenial::Unresolved);
        }
        let result = self.publish_manifest_residue_candidate(
            &admitted,
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
                .expect("registered lease")
                .retain_unresolved();
            runtime.health.revoke();
            return Err(denial);
        }
        // The selected root is now namespace-durable. Certification can kill
        // here to prove C8 closes the typed Intent without a Completed frame.
        self.mutations.reach_checkpoint(
            crate::physical_runtime::durability::PhysicalMutationCheckpoint::AfterRootReplacement,
        );
        if let Err(denial) = self
            .wal
            .append_scheduled_maintenance_receipt(&intent.completed().encode())
        {
            pending
                .take()
                .expect("registered lease")
                .retain_unresolved();
            runtime.health.revoke();
            return Err(maintenance_denial(denial));
        }
        if !admitted.complete() {
            pending
                .take()
                .expect("registered lease")
                .retain_unresolved();
            runtime.health.revoke();
            return Err(PhysicalRetirementDenial::Unresolved);
        }
        pending.take();
        Ok((source_generation, displaced))
    }
}

fn maintenance_denial(denial: ScheduledMaintenanceDenial) -> PhysicalRetirementDenial {
    match denial {
        ScheduledMaintenanceDenial::NotStarted(_)
        | ScheduledMaintenanceDenial::WrittenAwaitingBarrier { .. } => {
            PhysicalRetirementDenial::Waiting
        }
        ScheduledMaintenanceDenial::Write => PhysicalRetirementDenial::WalWrite,
        ScheduledMaintenanceDenial::Sync => PhysicalRetirementDenial::WalSync,
        ScheduledMaintenanceDenial::Finish => PhysicalRetirementDenial::WalFinish,
    }
}
