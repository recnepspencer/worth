//! Only caller-owned continuation custody can lend an awaited delivery.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::OutputRowStage;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandKey;

/// This borrow is the actual successor, not a key naming someone else's row.
pub(in super::super) struct AwaitedDelivery<'a, Schema: ApplicationSchema> {
    entries: &'a mut [RequiredFreshProgress<Schema>],
}
/// Delivery, unlike compute, can thaw a frozen head. No other step mints this.
pub(in super::super) struct DeliveredAwaited {
    _delivery: (),
}
impl DeliveredAwaited {
    pub(super) fn from_delivery(ready: SelectedReadyReadmission) -> Self {
        drop(ready);
        Self { _delivery: () }
    }
}
impl<Schema: ApplicationSchema + 'static> RequiredContinuations<Schema> {
    pub(super) fn awaited_delivery(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
    ) -> Result<Option<AwaitedDelivery<'_, Schema>>, WorthQueryOutputDemandDenial> {
        for index in 0..self.entries.len() {
            if matches!(
                registry.row_stage_admitted(self.entries[index].interest())?,
                OutputRowStage::Published
            ) {
                return Ok(Some(AwaitedDelivery {
                    entries: &mut self.entries[index..],
                }));
            }
        }
        Ok(None)
    }
}
impl<Schema: ApplicationSchema + 'static> AwaitedDelivery<'_, Schema> {
    pub(in super::super) fn key(&self) -> &WorthQueryOutputDemandKey {
        self.entries[0].interest().key()
    }
    pub(in super::super) fn deliver(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
        contacts: Option<&mut usize>,

        performed: &std::cell::RefCell<PerformedMembers>,
    ) -> Result<Option<DeliveredAwaited>, WorthQueryOutputDemandDenial> {
        let result = self.entries[0].resume(phase, runtime, principal, request, branch, performed);
        self.entries[0].report_caller_contacts(contacts);

        match result {
            Ok(Some(ready)) => {
                runtime.output_demands.clear_required_stop(self.key());
                self.entries.rotate_left(1);
                Ok(Some(DeliveredAwaited::from_delivery(ready)))
            }
            Ok(None) => Ok(None),
            Err(stop) => Err(stop),
        }
    }
}
