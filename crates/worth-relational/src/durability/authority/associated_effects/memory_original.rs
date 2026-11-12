//! Read owner-resident original meaning without claiming durable source custody.
use super::{DurabilityAuthority, RelationalAssociatedEffectRefusal as Refusal};
use crate::history::data::{PositionedCanonicalCommit, RelationalCommitReceipt};
use crate::transactions::data::RecoveredRelationalSemanticResult;

/// The actual resident Native owner artifact. This cannot qualify a directory
/// Home, mint a performed effect, or reconstruct a live CommitResult.
#[derive(Debug)]
pub struct ObservedRelationalMemoryCommit {
    origin: MemoryCommitOrigin,
    semantic: RecoveredRelationalSemanticResult,
    digest: [u8; 32],
}
#[derive(Debug)]
enum MemoryCommitOrigin {
    Live(PositionedCanonicalCommit),
    Interim {
        owner: std::sync::Weak<crate::runtime::RelationalRuntimeState>,
        admitted: crate::durability::checkpoint_admission::CheckpointAdmittedCommit,
        data: crate::durability::canonical_commit_artifact::CheckpointEffectData,
    },
}

impl ObservedRelationalMemoryCommit {
    pub fn receipt(&self) -> &RelationalCommitReceipt {
        self.semantic.semantic().receipt()
    }
    pub fn semantic_result(&self) -> &RecoveredRelationalSemanticResult {
        &self.semantic
    }
    pub fn position(&self) -> crate::publication::patch::data::PatchStreamPosition {
        match &self.origin {
            MemoryCommitOrigin::Live(positioned) => positioned.position(),
            MemoryCommitOrigin::Interim { admitted, .. } => admitted.index().position,
        }
    }
    pub fn artifact_digest(&self) -> &[u8; 32] {
        &self.digest
    }

    pub(crate) fn matches_original_envelope(
        &self,
        expected: &std::sync::Weak<crate::history::data::CanonicalCommitEnvelope>,
    ) -> bool {
        match &self.origin {
            MemoryCommitOrigin::Live(positioned) => std::sync::Weak::ptr_eq(
                expected,
                &std::sync::Arc::downgrade(positioned.canonical_arc()),
            ),
            MemoryCommitOrigin::Interim { .. } => false,
        }
    }

    pub(crate) fn matches_interim(
        &self,
        expected: &std::sync::Weak<crate::runtime::RelationalRuntimeState>,
        row: &crate::durability::checkpoint_admission::CheckpointAdmittedCommit,
    ) -> bool {
        match &self.origin {
            MemoryCommitOrigin::Live(_) => false,
            MemoryCommitOrigin::Interim {
                owner, admitted, ..
            } => std::sync::Weak::ptr_eq(owner, expected) && admitted.same_owner(row),
        }
    }

    pub fn created_entity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: crate::identity::data::EntityId,
    ) -> Result<Option<super::ReadmittedRelationalCreatedEntity<'_>>, Refusal> {
        super::created_row::read_created_entity(
            execution,
            self.semantic.semantic(),
            match &self.origin {
                MemoryCommitOrigin::Live(positioned) => {
                    &positioned.envelope().patch.authoritative_record_patches
                }
                MemoryCommitOrigin::Interim { data, .. } => &data.creations,
            },
            entity,
        )
    }
}

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
            return Ok(ObservedRelationalMemoryCommit {
                semantic: RecoveredRelationalSemanticResult {
                    semantic: data.semantic.clone(),
                },
                digest: entry.payload_digest,
                origin: MemoryCommitOrigin::Interim {
                    owner: self.runtime.state_binding(),
                    admitted: row,
                    data,
                },
            });
        }
        let positioned = residency
            .demand_positioned(execution, 1 << 30)
            .map_err(Refusal::Recovery)?;
        if &positioned.envelope().commit != receipt {
            return Err(Refusal::SubjectMismatch);
        }
        let digest = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::envelope_payload_digest_for_request(
            positioned.position(), positioned.canonical_arc(), execution,
        ).map_err(Refusal::Recovery)?;
        let semantic = RecoveredRelationalSemanticResult {
            semantic: positioned.envelope().semantic_result_arc(),
        };
        Ok(ObservedRelationalMemoryCommit {
            origin: MemoryCommitOrigin::Live(positioned),
            semantic,
            digest,
        })
    }
}
