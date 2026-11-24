//! Read owner-resident original meaning without claiming durable source custody.
use super::observed_memory_commit::ObservedRelationalMemoryCommit;
use super::RelationalAssociatedEffectRefusal as Refusal;
use crate::durability::authority::runtime_access::DurabilityAuthority;
use crate::history::data::RelationalCommitReceipt;

impl DurabilityAuthority<'_> {
    /// Observe a resident artifact explicitly. A source-backed artifact must
    /// use its source-readmission door; this is never a durability fallback.
    pub fn observe_memory_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &RelationalCommitReceipt,
    ) -> Result<ObservedRelationalMemoryCommit, Refusal> {
        crate::execution::check_request(execution).map_err(|cause| {
            Refusal::Recovery(crate::durability::data::RelationalRecoveryStop::Request(
                cause,
            ))
        })?;
        let _operation = self
            .runtime
            .owner_binding()
            .admit()
            .ok_or(Refusal::OwnerUnavailable)?;
        let artifact = self
            .runtime
            .history
            .commit_artifact(receipt.commit_id)
            .ok_or(Refusal::SubjectMismatch)?;
        if artifact.receipt() != *receipt {
            return Err(Refusal::SubjectMismatch);
        }
        let residency = artifact.envelope_residency();
        if let Some((entry, source)) = residency.source_placement() {
            let row = self
                .runtime
                .durability
                .checkpoint_selection()
                .admitted_commit(&entry)
                .map_err(|cause| {
                    Refusal::Recovery(
                        crate::durability::data::RelationalRecoveryStop::AdmissionImage(cause),
                    )
                })?
                .ok_or(Refusal::SubjectMismatch)?;
            let interim = match row.scope() {
                worth_foundational::facade::CheckpointAdmissionScope::Interim { .. } => true,
                worth_foundational::facade::CheckpointAdmissionScope::Home { .. } => false,
            };
            if !interim
                || !row.links_artifact(&artifact)
                || !self
                    .runtime
                    .durability
                    .checkpoint_selection()
                    .matches_admitted_source(&row, &source)
            {
                return Err(Refusal::SubjectMismatch);
            }
            let data = row.effect_data(execution).map_err(|cause| {
                Refusal::Recovery(
                    crate::durability::data::RelationalRecoveryStop::AdmissionImage(cause),
                )
            })?;
            return Ok(ObservedRelationalMemoryCommit::from_interim(
                self.runtime.services.origin_identity(),
                row,
                data,
                entry.payload_digest,
            ));
        }
        let positioned = residency
            .demand_positioned(execution, 1 << 30)
            .map_err(Refusal::Recovery)?;
        if &positioned.envelope().commit != receipt {
            return Err(Refusal::SubjectMismatch);
        }
        let digest = crate::history::commit::payload_commitment::digest_envelope_for_request(
            positioned.position(),
            positioned.canonical_arc(),
            execution,
        )
        .map_err(Refusal::Recovery)?;
        Ok(ObservedRelationalMemoryCommit::from_live(
            positioned, digest,
        ))
    }
}
