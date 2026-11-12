//! Completed leases transfer to durable originals; actual finished receipts retire explicitly.
use super::*;
use crate::domain_computation::application_aftermath::dispatch_journal::{
    QueryDispatchRecordedObservation, WorthQueryDispatchJournal,
};
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
    WorthQueryOperationHomeRetirementProgress as Progress,
};

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct InstalledTransportCloseCustody {
    correlations: BTreeSet<ExternalEffectCorrelationIdentity>,
}
impl InstalledTransportCloseCustody {
    pub(in crate::domain_computation::primary_graph) fn covers(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> bool {
        self.correlations.contains(correlation)
    }
}

impl InstalledTransportCompletionCustody {
    pub(in crate::domain_computation::primary_graph::application_runtime) fn preflight_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        journal: &WorthQueryDispatchJournal,
    ) -> Result<InstalledTransportCloseCustody, Denial> {
        let mut coverage = InstalledTransportCloseCustody::default();
        for (correlation, entry) in &self.entries {
            request.consult().map_err(Denial::Execution)?;
            match &entry.state {
                RetainedCompletion::Ready(_)
                | RetainedCompletion::Unpublished(_, _, None)
                | RetainedCompletion::Recorded(_, _) => {}
                RetainedCompletion::Publishing
                | RetainedCompletion::Recovering
                | RetainedCompletion::PerformedPending(_, _)
                | RetainedCompletion::Unpublished(_, _, Some(_))
                | RetainedCompletion::StaleRelease(_, _) => {
                    return Err(Denial::Busy(Owner::PublicationReceipts))
                }
            }
            if !journal.is_durable()
                || journal
                    .custody(correlation.digest())
                    .and_then(|c| c.observation())
                    != Some(QueryDispatchRecordedObservation::Completed)
            {
                return Err(Denial::Busy(Owner::PublicationReceipts));
            }
            coverage.correlations.insert(*correlation);
        }
        for performed in self.finished.values_mut() {
            // finish() is reached only after retaining the actual canonical
            // World source guard. Release this duplicate history observation.
            drop(performed.publication_mut().take_history());
            performed
                .publication_mut()
                .preflight_home_retirement(request)?;
        }
        Ok(coverage)
    }

    pub(in crate::domain_computation::primary_graph::application_runtime) fn advance_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<Progress, Denial> {
        request.consult().map_err(Denial::Execution)?;
        if let Some((_, entry)) = self.entries.pop_first() {
            // Exclusive retirement has validated the recorded completion. The
            // same original remains protected in OutstandingDispatchOwner.
            self.retired.push(entry.state);
            drop(entry._in_flight);
            return Ok(Progress::Advanced);
        }
        if let Some(mut entry) = self.finished.first_entry() {
            if entry
                .get_mut()
                .publication_mut()
                .advance_home_retirement(request, native)?
                == Progress::Complete
            {
                self.retired_finished.push(entry.remove());
            }
            return Ok(Progress::Advanced);
        }
        Ok(Progress::Complete)
    }
}
