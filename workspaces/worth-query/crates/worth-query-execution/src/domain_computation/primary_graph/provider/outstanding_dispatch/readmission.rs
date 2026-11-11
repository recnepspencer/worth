//! Reconstruct finite outstanding membership from original outboxes minus canonical terminals.
use super::super::{
    WorthQueryCommittedDispatchOutboxReadDenial as Denial, WorthQueryPrimaryGraphProvider,
};
use super::*;
use worth_runtime_world::facade::{
    RuntimeWorldInspectionPort, RuntimeWorldObservationPort, WorldReadmittedPublication,
};

impl OutstandingDispatchOwner {
    fn readmit_original(
        &self,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
        world: WorldReadmittedPublication,
    ) -> Result<(), Denial> {
        let record = owner.record();
        let contract = record.inbound().ok_or(Denial::RecordMismatch)?;
        let source = OutstandingDispatchOriginal::Readmitted(world);
        if !source.matches(owner.committed_product_publication()) {
            return Err(Denial::CommitMismatch);
        }
        let mut entries = self.entries.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(existing) = entries.by_correlation.get(record.correlation()) {
            return if existing.contract == *contract
                && existing
                    .original
                    .as_ref()
                    .is_some_and(|source| source.matches(owner.committed_product_publication()))
            {
                Ok(())
            } else {
                Err(Denial::RecordMismatch)
            };
        }
        let count = entries.counts.get(contract).copied().unwrap_or(0);
        if count
            >= contract
                .limits()
                .maximum_outstanding_dispatch_provenance
                .get()
        {
            return Err(Denial::RecordMismatch);
        }
        let publication = owner.committed_product_publication();
        entries.counts.insert(contract.clone(), count + 1);
        *entries
            .by_branch
            .entry(publication.product_incarnation())
            .or_default() += 1;
        entries.by_correlation.insert(
            *record.correlation(),
            OutstandingDispatchEntry {
                contract: contract.clone(),
                branch: publication.product_branch().clone(),
                incarnation: publication.product_incarnation(),
                commit: Some(owner.commit_reference().commit_id),
                original: Some(source),
                in_flight: 0,
                terminal_world: None,
            },
        );
        Ok(())
    }
}

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn readmit_outstanding_dispatches(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &RuntimeWorldObservationPort,
        inspection: &RuntimeWorldInspectionPort,
    ) -> Result<Vec<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
        let mut after = None;
        let mut outstanding = Vec::new();
        loop {
            execution.consult().map_err(|_| Denial::NotAuthoritative)?;
            let next = self
                .completed_commit_evidence
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .next_recovered_dispatch(after.as_ref());
            let Some((correlation, source)) = next else {
                return Ok(outstanding);
            };
            after = Some(correlation);
            if self
                .lookup_completed_inbound(&correlation)
                .map_err(|_| Denial::CommittedIndexUnavailable)?
                .is_some()
            {
                continue;
            }
            let (owner, world) =
                self.read_recovered_dispatch_source(execution, &source, observation, inspection)?;
            if owner.record().inbound().is_some() {
                self.outstanding_dispatch.readmit_original(&owner, world)?;
                outstanding.push(owner);
            }
        }
    }
}
