use super::*;
use crate::physical_runtime::record_serving::{
    publication::write_root_candidate_artifacts,
    residency::publication_artifacts::PublicationRecordArtifacts, PreparedPhysicalRootCandidate,
};

impl RecordPublicationDirector {
    pub(super) fn publish_extent_release(
        &self,
        mut candidate: ReleaseCandidate,
        receipt: DurableMaintenanceReceipt,
    ) -> Result<(), PhysicalRetirementDenial> {
        let (_, release, wal_digest) = candidate
            .transition
            .identity()
            .retirement_basis()
            .ok_or(PhysicalRetirementDenial::WalPlan)?;
        let candidate_digest: [u8; 32] = Sha256::digest(&candidate.plan.root_bytes).into();
        if wal_digest != receipt.payload_digest()
            || release.candidate_digest() != candidate_digest
            || release.source_generation() != candidate.source.generation()
            || release.candidate_generation() != candidate.plan.generation
            || candidate.plan.retained_metadata_bytes() != Some(release.metadata_bytes())
        {
            return Err(PhysicalRetirementDenial::WalPlan);
        }
        let frames = candidate
            .plan
            .root_candidate_frame_set()
            .map_err(|_| PhysicalRetirementDenial::WalPlan)?;
        let mut residency = self
            .residency
            .begin_candidate_publication(&candidate.allocation, frames)
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        candidate.transition.mark_effect_started();
        let artifacts = match &candidate.retry {
            Some(scope) => PublicationRecordArtifacts::retirement_retry(&self.mutation, scope),
            None => PublicationRecordArtifacts::new(&self.mutation),
        };
        let written = write_root_candidate_artifacts(&artifacts, candidate.plan, &mut residency)
            .map_err(|_| PhysicalRetirementDenial::Delete)?;
        residency
            .require_complete()
            .map_err(|_| PhysicalRetirementDenial::Delete)?;
        drop(residency);
        #[cfg(feature = "certification-test-authority")]
        self.pause_retirement_kill(3);
        let root_candidate = PreparedPhysicalRootCandidate::new(
            candidate.source,
            candidate.free,
            written.plan,
            written.artifacts,
        );
        let durable = crate::physical_runtime::durability::publish_retirement_candidate(
            root_candidate,
            candidate.transition,
            receipt,
            &self.root_work,
        )?;
        self.root_owner
            .advance_retirement_root(durable, self.format.declaration())?;
        #[cfg(feature = "certification-test-authority")]
        self.pause_retirement_kill(4);
        Ok(())
    }
}
