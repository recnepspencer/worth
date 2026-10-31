//! Read-only original effect facts; none of these descriptions grants adoption.
use super::DurabilityAuthority;
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::medium::RelationalDurableColdSource;
use crate::history::data::RelationalCallerAssociation;
use crate::history::{RelationalCommitEnvelopeResidency, RelationalCommitIndexEntry};
use crate::transactions::data::RecoveredRelationalSemanticResult;
use std::sync::Arc;

mod readmission;

/// A subject discovered from the actual Native catalog, retaining its exact
/// source and index. It is not serializable or a publication capability.
///
/// ```compile_fail,E0277
/// use worth_relational::facade::durability::RelationalAssociatedEffectSubject;
/// fn cannot_readmit_a_source_subject_from_data(bytes: &[u8]) {
///     let _: RelationalAssociatedEffectSubject = rmp_serde::from_slice(bytes).unwrap();
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RelationalAssociatedEffectSubject {
    runtime_instance_id: u64,
    entry: Arc<RelationalCommitIndexEntry>,
    source: Arc<dyn RelationalDurableColdSource>,
    residency: Arc<RelationalCommitEnvelopeResidency>,
}

impl RelationalAssociatedEffectSubject {
    pub fn receipt(&self) -> &crate::history::data::RelationalCommitReceipt {
        &self.entry.commit
    }
    pub fn position(&self) -> crate::publication::patch::data::PatchStreamPosition {
        self.entry.position
    }
    pub fn artifact_digest(&self) -> &[u8; 32] {
        &self.entry.payload_digest
    }
    pub fn roots(&self) -> &crate::branch::RelationalBranchRootDescriptor {
        &self.entry.root
    }
    pub fn association(&self) -> Option<&RelationalCallerAssociation> {
        self.entry.association_discovery.association()
    }
}

/// These observations describe Native owner state, never permission to move a
/// product head. In particular checkpoint readmission does not mean settled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalAssociatedEffectSettlement {
    PerformedPending,
    PerformedSettled,
    CheckpointReadmitted,
}

#[derive(Debug)]
pub struct ReadmittedRelationalAssociatedEffect {
    subject: RelationalAssociatedEffectSubject,
    semantic: RecoveredRelationalSemanticResult,
    settlement: RelationalAssociatedEffectSettlement,
}

impl ReadmittedRelationalAssociatedEffect {
    pub fn subject(&self) -> &RelationalAssociatedEffectSubject {
        &self.subject
    }
    pub fn semantic_result(&self) -> &crate::transactions::data::RelationalCommitSemanticResult {
        self.semantic.semantic()
    }
    pub fn settlement(&self) -> RelationalAssociatedEffectSettlement {
        self.settlement
    }
}

#[derive(Debug)]
pub enum RelationalAssociatedEffectRefusal {
    Recovery(RelationalRecoveryStop),
    AssociatedMemory {
        commit_id: crate::history::data::CommitId,
    },
    MissingDiscovery {
        commit_id: crate::history::data::CommitId,
    },
    ForeignRuntime,
    CatalogSubjectChanged,
    MissingSettlementEvidence,
    SubjectMismatch,
}

impl DurabilityAuthority<'_> {
    /// Inspect only catalog metadata and already-resident memory facts. Cold
    /// bodies unrelated to this original opaque association remain cold.
    pub fn discover_associated_effects(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        association: &RelationalCallerAssociation,
    ) -> Result<Vec<RelationalAssociatedEffectSubject>, RelationalAssociatedEffectRefusal> {
        use RelationalAssociatedEffectRefusal as Refusal;
        crate::execution::check_request(execution)
            .map_err(|cause| Refusal::Recovery(RelationalRecoveryStop::Request(cause)))?;
        // A performed publication may still await catalog settlement. Include
        // that same route's custody; missing catalog projection is not a fate.
        let mut cells = self
            .runtime
            .history
            .commit_artifacts()
            .into_iter()
            .map(|artifact| (artifact.commit_id(), artifact.envelope_residency()))
            .collect::<std::collections::BTreeMap<_, _>>();
        for (id, cell) in self.runtime.history.performed_effect_residencies() {
            if let Some(existing) = cells.get(&id) {
                if !Arc::ptr_eq(existing, &cell) {
                    return Err(Refusal::CatalogSubjectChanged);
                }
            } else {
                cells.insert(id, cell);
            }
        }
        let mut subjects = Vec::new();
        for (commit_id, residency) in cells {
            crate::execution::check_request(execution)
                .map_err(|cause| Refusal::Recovery(RelationalRecoveryStop::Request(cause)))?;
            let Some((entry, source)) = residency.journal_placement() else {
                let envelope = residency
                    .get()
                    .ok_or(Refusal::MissingDiscovery { commit_id })?;
                if envelope.caller_association() == Some(association) {
                    return Err(Refusal::AssociatedMemory { commit_id });
                }
                continue;
            };
            if entry.association_discovery.association() != Some(association) {
                continue;
            }
            subjects.push(RelationalAssociatedEffectSubject {
                runtime_instance_id: self.runtime.runtime_instance_id(),
                entry,
                source,
                residency,
            });
        }
        subjects.sort_by_key(|subject| subject.position());
        Ok(subjects)
    }
}
