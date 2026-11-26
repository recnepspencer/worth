//! Discover read-only original effect facts from the live owner's catalog.
use super::*;
use crate::durability::authority::runtime_access::DurabilityAuthority;
use worth_relational_durability::internal_port::durability::authority::associated_effects::RelationalAssociatedEffectSubjectOwnerPort as _;

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

    /// The one associated subject of exactly `receipt`, by key. Both custody
    /// cells the enumeration merges are read, and a disagreement refuses.
    pub fn discover_associated_effect(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &crate::history::data::RelationalCommitReceipt,
    ) -> Result<Option<RelationalAssociatedEffectSubject>, RelationalAssociatedEffectRefusal> {
        use RelationalAssociatedEffectRefusal as Refusal;
        crate::execution::check_request(execution)
            .map_err(|cause| Refusal::Recovery(RelationalRecoveryStop::Request(cause)))?;
        let _operation = self
            .runtime
            .owner_binding()
            .admit()
            .ok_or(Refusal::OwnerUnavailable)?;
        let id = receipt.commit_id;
        let catalog = self
            .runtime
            .history
            .try_commit_artifact(id)
            .map_err(|cause| {
                Refusal::Recovery(
                    worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailable {
                        subject: worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageSubject::Envelope(
                            crate::publication::patch::data::PatchStreamPosition(0),
                        ),
                        cause,
                    }
                    .into(),
                )
            })?
            .map(|artifact| artifact.envelope_residency());
        let performed = self.runtime.history.performed_effect_residency(id);
        let residency = match (catalog, performed) {
            (Some(existing), Some(cell)) if !Arc::ptr_eq(&existing, &cell) => {
                return Err(Refusal::CatalogSubjectChanged);
            }
            (Some(cell), _) | (None, Some(cell)) => cell,
            (None, None) => return Ok(None),
        };
        Ok(self
            .discover_cell(id, residency, None)?
            .filter(|subject| subject.receipt() == receipt))
    }

    fn discover_matching_associated_effects(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        association: Option<&RelationalCallerAssociation>,
    ) -> Result<Vec<RelationalAssociatedEffectSubject>, RelationalAssociatedEffectRefusal> {
        use RelationalAssociatedEffectRefusal as Refusal;
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
            .try_commit_artifacts()
            .map_err(|cause| {
                Refusal::Recovery(
                    worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailable {
                        subject: worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageSubject::Envelope(
                            crate::publication::patch::data::PatchStreamPosition(0),
                        ),
                        cause,
                    }
                    .into(),
                )
            })?
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
            subjects.extend(self.discover_cell(commit_id, residency, association)?);
        }
        subjects.sort_by_key(|subject| subject.position());
        Ok(subjects)
    }

    /// One custody cell: its subject when its association matches `wanted`
    /// (any association when `None`). Cold bodies stay cold.
    fn discover_cell(
        &self,
        commit_id: crate::history::data::CommitId,
        residency: Arc<
            worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency,
        >,
        wanted: Option<&RelationalCallerAssociation>,
    ) -> Result<Option<RelationalAssociatedEffectSubject>, RelationalAssociatedEffectRefusal> {
        use RelationalAssociatedEffectRefusal as Refusal;
        let matches = |actual: Option<&RelationalCallerAssociation>| {
            actual.is_some_and(|actual| wanted.is_none_or(|wanted| wanted == actual))
        };
        let Some((entry, source)) = residency.source_placement() else {
            let envelope = residency
                .get()
                .ok_or(Refusal::MissingDiscovery { commit_id })?;
            if matches(envelope.caller_association()) {
                return Err(Refusal::AssociatedMemory { commit_id });
            }
            return Ok(None);
        };
        if !matches(entry.association_discovery.association()) {
            return Ok(None);
        }
        Ok(Some(RelationalAssociatedEffectSubject::capture_subject(
            self.runtime.runtime_instance_id(),
            entry,
            Arc::downgrade(&source),
            Arc::downgrade(&residency),
        )))
    }
}

pub(crate) fn current_settlement(
    subject: &RelationalAssociatedEffectSubject,
    runtime: &crate::runtime::RelationalRuntime,
) -> Option<RelationalAssociatedEffectSettlement> {
    if !subject.belongs_to_runtime(runtime.runtime_instance_id())
        || !runtime.owner_binding().accepts_operations()
    {
        return None;
    }
    let residency = subject.bound_residency()?;
    runtime
        .history
        .associated_effect_settlement(subject.retained_entry(), &residency)
}
