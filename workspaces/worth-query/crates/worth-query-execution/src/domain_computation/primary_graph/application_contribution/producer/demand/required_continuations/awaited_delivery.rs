//! Only caller-owned continuation custody can lend an awaited delivery.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::{
    OutputRowStage, WorthQueryOutputDemandKey,
};

/// This borrow is the actual successor, not a key naming someone else's row.
pub(in super::super) struct AwaitedDelivery<'a, Schema: ApplicationSchema> {
    entries: &'a mut [RequiredFreshProgress<Schema>],
}
/// Delivery, unlike compute, can thaw a frozen head. No other step mints this.
pub(in super::super) struct DeliveredAwaited(());
pub(super) fn delivered_registry() -> DeliveredAwaited {
    DeliveredAwaited(())
}
impl<Schema: ApplicationSchema + 'static> RequiredContinuations<Schema> {
    pub(super) fn awaited_delivery(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<AwaitedDelivery<'_, Schema>>, WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work(u64::try_from(self.entries.len()).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        for index in 0..self.entries.len() {
            if matches!(
                registry.row_stage_admitted(self.entries[index].interest(), admission)?,
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
        admission: &mut InvalidationEditAdmission,
        performed: &std::cell::RefCell<PerformedMembers>,
    ) -> Result<Option<DeliveredAwaited>, WorthQueryOutputDemandDenial> {
        let result = self.entries[0].resume(
            phase, runtime, principal, request, branch, admission, performed,
        );
        self.entries[0].report_caller_contacts(contacts);
        if matches!(result, Ok(Some(_))) || result.is_err() {
            let work = self
                .entries
                .len()
                .checked_mul(std::mem::size_of::<RequiredFreshProgress<Schema>>())
                .and_then(|work| u64::try_from(work).ok())
                .ok_or_else(work_denial)?;
            admission
                .charge_external_work(work)
                .map_err(|_| work_denial())?;
        }
        match result {
            Ok(Some(_)) => {
                runtime.output_demands.clear_required_stop(self.key());
                self.entries.rotate_left(1);
                Ok(Some(DeliveredAwaited(())))
            }
            Ok(None) => Ok(None),
            Err(stop) => Err(stop),
        }
    }
}
