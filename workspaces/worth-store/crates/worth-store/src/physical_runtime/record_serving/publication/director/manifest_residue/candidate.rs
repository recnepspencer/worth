use super::*;

impl RecordPublicationDirector {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn publish_manifest_residue_candidate(
        &self,
        admitted: &AdmittedManifestResidueRetirement,
        source: worth_store_physical_format::DurablePhysicalRootManifest,
        free: worth_store_physical_format::DurableFreeSpaceManifestHeader,
        plan: crate::physical_runtime::record_serving::publication::PublicationPlan,
        mut transition: crate::physical_runtime::durability::PhysicalRootPublicationTransition,
        receipt: DurableMaintenanceReceipt,
        allocation: &worth_store_buffer_pool::ForegroundWriteAllocationGrant,
    ) -> Result<(), PhysicalRetirementDenial> {
        let frames = plan
            .root_candidate_frame_set()
            .map_err(|_| PhysicalRetirementDenial::WalPlan)?;
        let mut residency = self
            .residency
            .begin_candidate_publication(allocation, frames)
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        transition.mark_effect_started();
        let artifacts = PublicationRecordArtifacts::new(&self.mutation);
        let written = write_root_candidate_artifacts(&artifacts, plan, &mut residency)
            .map_err(|_| PhysicalRetirementDenial::Delete)?;
        residency
            .require_complete()
            .map_err(|_| PhysicalRetirementDenial::Delete)?;
        drop(residency);
        let candidate =
            PreparedPhysicalRootCandidate::new(source, free, written.plan, written.artifacts);
        let durable =
            publish_manifest_residue_candidate(candidate, transition, receipt, &self.root_work)?;
        self.root_owner
            .advance_manifest_residue_root(admitted, durable, self.format.declaration())
    }
}
