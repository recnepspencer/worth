//! Durable handoff retains original source guards through every refused close.
use super::*;
use crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchCloseCustody;
use crate::domain_computation::primary_graph::{
    application_runtime::inbound_publication::InstalledTransportCloseCustody,
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
    WorthQueryOperationHomeRetirementProgress as Progress,
};

impl OutstandingDispatchOwner {
    pub(in crate::domain_computation::primary_graph::provider) fn protect_for_durable_close(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        inspection: &worth_runtime_world::facade::RuntimeWorldInspectionPort,
    ) -> Result<(), Denial> {
        let mut entries = self
            .entries
            .try_lock()
            .map_err(|_| Denial::Busy(Owner::GraphProvider))?;
        for entry in entries.by_correlation.values_mut() {
            request.consult().map_err(Denial::Execution)?;
            if let Some(OutstandingDispatchOriginal::Performed(publication)) = &entry.original {
                let protection = inspection
                    .protect_performed_publication(publication.composite_commit())
                    .map_err(|_| Denial::Busy(Owner::PublicationReceipts))?;
                // Capture the real source guard before releasing any duplicate receipt.
                entry.original = Some(OutstandingDispatchOriginal::Protected {
                    description: publication.descriptive(),
                    _protection: protection,
                });
            }
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::provider) fn preflight_home_retirement(
        &self,
        custody: &QueryDispatchCloseCustody,
        transport: &InstalledTransportCloseCustody,
    ) -> Result<(), Denial> {
        let entries = self
            .entries
            .try_lock()
            .map_err(|_| Denial::Busy(Owner::GraphProvider))?;
        if entries.by_correlation.is_empty() {
            return Ok(());
        }
        match custody {
            QueryDispatchCloseCustody::ProcessLocal => Err(Denial::Busy(Owner::GraphProvider)),
            QueryDispatchCloseCustody::Durable(_) => {
                for (correlation, entry) in &entries.by_correlation {
                    if entry.commit.is_none()
                        || entry.original.is_none()
                        || entry.in_flight > u64::from(transport.covers(correlation))
                    {
                        return Err(Denial::Busy(Owner::GraphProvider));
                    }
                }
                Ok(())
            }
        }
    }

    pub(in crate::domain_computation::primary_graph::provider) fn advance_source_retirement(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Progress, Denial> {
        request.consult().map_err(Denial::Execution)?;
        let mut entries = self
            .entries
            .try_lock()
            .map_err(|_| Denial::Busy(Owner::GraphProvider))?;
        let Some((_, entry)) = entries.by_correlation.pop_first() else {
            return Ok(Progress::Complete);
        };
        assert_eq!(
            entry.in_flight, 0,
            "transport leases transfer before source retirement"
        );
        decrement(&mut entries.counts, &entry.contract);
        decrement(&mut entries.by_branch, &entry.incarnation);
        Ok(Progress::Advanced)
    }
}

impl super::super::WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn protect_dispatch_originals_for_close(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        inspection: &worth_runtime_world::facade::RuntimeWorldInspectionPort,
    ) -> Result<(), Denial> {
        self.outstanding_dispatch
            .protect_for_durable_close(request, inspection)
    }
}
