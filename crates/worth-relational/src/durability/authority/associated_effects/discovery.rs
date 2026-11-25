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

    /// Look up one exact associated source subject by receipt without scanning
    /// unrelated history or reading cold artifact bodies.
    pub fn associated_effect_for_receipt(
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
        let commit_id = receipt.commit_id;
        let catalog_cell = self
            .runtime
            .history
            .commit_artifact(commit_id)
            .map(|artifact| artifact.envelope_residency());
        let performed_cell = self.runtime.history.performed_effect_residency(commit_id);
        let residency = match (catalog_cell, performed_cell) {
            (Some(catalog), Some(performed)) => {
                if !Arc::ptr_eq(&catalog, &performed) {
                    return Err(Refusal::CatalogSubjectChanged);
                }
                catalog
            }
            (Some(cell), None) | (None, Some(cell)) => cell,
            (None, None) => return Ok(None),
        };
        let Some((entry, source)) = residency.source_placement() else {
            let envelope = residency
                .get()
                .ok_or(Refusal::MissingDiscovery { commit_id })?;
            if envelope.caller_association().is_some() {
                return Err(Refusal::AssociatedMemory { commit_id });
            }
            return Ok(None);
        };
        if &entry.commit != receipt || entry.association_discovery.association().is_none() {
            return Ok(None);
        }
        Ok(Some(RelationalAssociatedEffectSubject::capture_subject(
            self.runtime.runtime_instance_id(),
            entry,
            Arc::downgrade(&source),
            Arc::downgrade(&residency),
        )))
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
            subjects.push(RelationalAssociatedEffectSubject::capture_subject(
                self.runtime.runtime_instance_id(),
                entry,
                Arc::downgrade(&source),
                Arc::downgrade(&residency),
            ));
        }
        subjects.sort_by_key(|subject| subject.position());
        Ok(subjects)
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
