//! Read-only original effect facts; none of these descriptions grants adoption.
use super::DurabilityAuthority;
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::medium::RelationalDurableColdSource;
use crate::history::data::RelationalCallerAssociation;
use crate::history::RelationalCommitEnvelopeResidency;
use crate::transactions::data::RecoveredRelationalSemanticResult;
use std::sync::{Arc, Weak};

mod readmission;

/// Exact discovered index and weak owner/source identities. The actual Native
/// owner retains source custody; an observer cannot keep a closed Home serving.
/// This is not serializable or a publication capability.
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
    entry: crate::history::RetainedCanonicalIndex,
    source: Weak<dyn RelationalDurableColdSource>,
    residency: Weak<RelationalCommitEnvelopeResidency>,
}

impl RelationalAssociatedEffectSubject {
    pub(super) fn source_entry(&self) -> &crate::history::RelationalCommitIndexEntry {
        &self.entry
    }

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
    positioned: crate::history::data::PositionedCanonicalCommit,
    settlement: RelationalAssociatedEffectSettlement,
}

impl ReadmittedRelationalAssociatedEffect {
    pub(super) fn source_envelope(&self) -> &crate::history::data::CanonicalCommitEnvelope {
        self.positioned.envelope()
    }
    pub fn subject(&self) -> &RelationalAssociatedEffectSubject {
        &self.subject
    }
    pub fn semantic_result(&self) -> &crate::transactions::data::RelationalCommitSemanticResult {
        self.semantic.semantic()
    }
    pub fn recovered_semantic_result(&self) -> &RecoveredRelationalSemanticResult {
        &self.semantic
    }
    pub fn settlement(&self) -> RelationalAssociatedEffectSettlement {
        self.settlement
    }
}

impl RelationalAssociatedEffectSubject {
    pub(super) fn belongs_to_runtime(&self, runtime: &crate::runtime::RelationalRuntime) -> bool {
        self.runtime_instance_id == runtime.runtime_instance_id()
    }
    pub(super) fn current_settlement(
        &self,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Option<RelationalAssociatedEffectSettlement> {
        if !self.belongs_to_runtime(runtime) || !runtime.owner_binding().accepts_operations() {
            return None;
        }
        let residency = self.residency.upgrade()?;
        let source = self.source.upgrade()?;
        let (entry, actual_source) = residency.source_placement()?;
        if !entry.same_owner(&self.entry) || !Arc::ptr_eq(&actual_source, &source) {
            return None;
        }
        runtime
            .history
            .associated_effect_settlement(&self.entry, &residency)
    }
}

#[derive(Debug)]
pub enum RelationalAssociatedEffectRefusal {
    OwnerUnavailable,
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
        self.discover_matching_associated_effects(execution, Some(association))
    }

    /// Enumerate associated source subjects without reading cold artifact bodies.
    pub fn discover_all_associated_effects(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<RelationalAssociatedEffectSubject>, RelationalAssociatedEffectRefusal> {
        self.discover_matching_associated_effects(execution, None)
    }

    fn discover_matching_associated_effects(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        association: Option<&RelationalCallerAssociation>,
    ) -> Result<Vec<RelationalAssociatedEffectSubject>, RelationalAssociatedEffectRefusal> {
        use RelationalAssociatedEffectRefusal as Refusal;
        let matches = |actual: Option<&RelationalCallerAssociation>| {
            actual.is_some_and(|actual| association.is_none_or(|wanted| wanted == actual))
        };
        crate::execution::check_request(execution)
            .map_err(|cause| Refusal::Recovery(RelationalRecoveryStop::Request(cause)))?;
        let _operation = self
            .runtime
            .owner_binding()
            .admit()
            .ok_or(Refusal::OwnerUnavailable)?;
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
            let Some((entry, source)) = residency.source_placement() else {
                let envelope = residency
                    .get()
                    .ok_or(Refusal::MissingDiscovery { commit_id })?;
                if matches(envelope.caller_association()) {
                    return Err(Refusal::AssociatedMemory { commit_id });
                }
                continue;
            };
            if !matches(entry.association_discovery.association()) {
                continue;
            }
            subjects.push(RelationalAssociatedEffectSubject {
                runtime_instance_id: self.runtime.runtime_instance_id(),
                entry,
                source: Arc::downgrade(&source),
                residency: Arc::downgrade(&residency),
            });
        }
        subjects.sort_by_key(|subject| subject.position());
        Ok(subjects)
    }
}
