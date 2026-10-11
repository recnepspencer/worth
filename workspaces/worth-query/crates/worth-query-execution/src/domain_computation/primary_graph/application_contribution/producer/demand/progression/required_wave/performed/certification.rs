//! Native certification issues the sole consuming key for a Current result.
use super::super::*;
use super::PerformedMembers;
mod current_basis;
pub(in crate::domain_computation::primary_graph) use current_basis::CurrentBasis;

impl PerformedMembers {
    fn current<T>(
        &self,
        value: T,
        basis: CurrentBasis,
        record: Arc<crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership>,
    ) -> CurrentProof<T> {
        CurrentProof {
            value,
            basis,
            record,
        }
    }
}

/// A move-only key to acknowledgement, caller acceptance or requested return.
pub(in crate::domain_computation::primary_graph) struct CurrentProof<T> {
    value: T,
    basis: CurrentBasis,
    record: Arc<
        crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership,
    >,
}
impl<T> CurrentProof<T> {
    pub(in crate::domain_computation::primary_graph) fn certifies_record(
        &self,
        record: &Arc<crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership>,
    ) -> bool {
        Arc::ptr_eq(&self.record, record) && record.certified_at(&self.basis)
    }
    pub(in super::super) fn value(&self) -> &T {
        &self.value
    }
    pub(in super::super) fn into_value(self) -> T {
        self.value
    }
}

/// Certify one exact Ready cell while the wave retains one selected Product.
/// The registry's exact consumed-ID join converts PendingExact to owned
/// scheduling custody before the candidate's borrowed proof leaves scope.
pub(in super::super) fn certify_required_ready<'owner, Schema>(
    phase: &'owner WorthQueryAdvancementPhase<'owner>,

    runtime: &'owner WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    wave: &'owner RequiredWaveSelection<'_, Schema>,
    selected: &SelectedReadyReadmission,
    caller_installed: Option<&InstalledProducerProvider<Schema>>,
    resolved: Option<&ResolvedRequiredPredecessors<'_, Schema>>,
    producer_contacts_in_this_demand: usize,
    refresh_permission: Result<(), WorthQueryOutputDemandDenial>,
    performed: &'owner std::cell::RefCell<performed::PerformedMembers>,
    admission: &mut InvalidationEditAdmission,
) -> Result<RequiredWaveStep<'owner, Schema>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    preclaim_required_settlement_arguments(admission)?;
    let candidate = runtime
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .resolve_required_settlement(
            runtime.runtime.authority_identity().as_u64(),
            &runtime.installed_schema.binding_identity(),
            selected.completion(),
            admission,
        );
    // Without an exact candidate, Ready takes Fresh; this advance's record
    // refuses a published member unless there is evidence of change.
    let candidate = candidate.map_err(required_settlement_denial)?;
    admission
        .charge_external_work(1)
        .map_err(admission_denial)?;
    // Only the caller's exact InterestÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢Ready join may reuse the installed
    // entry retained when that same demand was admitted. Other cues still
    // resolve their own producer through the installed table.
    performed
        .borrow_mut()
        .capture(selected.key(), &candidate, admission)?;
    let candidate = candidate.ok().flatten();
    let installed = match caller_installed {
        Some(entry) => entry,
        None => installed_for_selected_cue(runtime, selected.producer_identity(), admission)?,
    };
    let result = installed
        .executor
        .advance_ready_on_selected(
            phase,
            runtime,
            principal,
            request_scope,
            &wave.shared,
            &wave.positioned,
            selected,
            candidate.as_ref(),
            resolved,
            installed,
            wave.branch,
            producer_contacts_in_this_demand,
            refresh_permission,
            performed,
            admission,
        )
        .map_err(|stop| match stop {
            ProducerExecutionStop::RequestAdmissionDenied(rejection) => rejection.into_denial(),
            ProducerExecutionStop::ExecutionStopped(denial) => denial,
            ProducerExecutionStop::LiveOutputNotReused { producer, reason } => {
                ProducerExecutionStop::live_output_not_reused(producer, reason)
            }
        })?;
    match result {
        RequiredCueProgress::Current(settlement) => {
            Ok(RequiredWaveStep::Current(performed.borrow().current(
                settlement,
                CurrentBasis::of(wave.shared.selected().product().observation()),
                selected.certify_current_record(CurrentBasis::of(
                    wave.shared.selected().product().observation(),
                )),
            )))
        }
        RequiredCueProgress::PendingExact(pending) => {
            let upstream = runtime.output_demands.pending_exact_ready_readmission(
                &pending,
                &wave.positioned,
                admission,
            )?;
            Ok(match upstream {
                PendingUpstream::Ready(ready) => RequiredWaveStep::Upstream(ready),
                PendingUpstream::Held(head) => RequiredWaveStep::Held(head),
                PendingUpstream::Unavailable(_) => RequiredWaveStep::Pending,
            })
        }
        RequiredCueProgress::PendingUnresolved => Ok(RequiredWaveStep::Pending),
        RequiredCueProgress::Fresh(plan) => Ok(RequiredWaveStep::Fresh(plan)),
    }
}

#[cfg(test)]
mod acknowledgement_basis;
/// Frozen permits own-row truth and the native required-upstream resume.
/// Thaw follows the successor's custody: its caller's continuation, or a
/// registry frame moved to the advance that requires it. No global sweep runs.
/// A caller needing work is Pending and is retried by its driving owner.
pub(in super::super) fn certify_frozen_ready<Schema: ApplicationSchema + 'static>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    wave: &RequiredWaveSelection<'_, Schema>,
    caller: FrozenCaller<'_, Schema>,
    performed: &std::cell::RefCell<PerformedMembers>,
    admission: &mut InvalidationEditAdmission,
) -> Result<FrozenCallerInspection<Schema>, WorthQueryOutputDemandDenial> {
    let FrozenCaller {
        selected,
        installed,
        contacts,
    } = caller;
    preclaim_required_settlement_arguments(admission)?;
    let candidate = runtime
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .resolve_required_settlement(
            runtime.runtime.authority_identity().as_u64(),
            &runtime.installed_schema.binding_identity(),
            selected.completion(),
            admission,
        )
        .map_err(required_settlement_denial)?;
    admission
        .charge_external_work(1)
        .map_err(admission_denial)?;
    performed
        .borrow_mut()
        .capture(selected.key(), &candidate, admission)?;
    let candidate = candidate.ok().flatten();
    let result = installed.executor.inspect_frozen_ready(
        crate::domain_computation::primary_graph::application_contribution::producer::execution::FrozenInspection {
            runtime, principal, request, shared: &wave.shared, positioned: &wave.positioned,
            selected, candidate: candidate.as_ref(), contacts,
        }, admission)
        .map_err(|stop| match stop {
            ProducerExecutionStop::RequestAdmissionDenied(rejection) => rejection.into_denial(),
            ProducerExecutionStop::ExecutionStopped(denial) => denial,
            ProducerExecutionStop::LiveOutputNotReused { producer, reason } =>
                ProducerExecutionStop::live_output_not_reused(producer, reason),
        })?;
    Ok(match result {
        crate::domain_computation::primary_graph::application_contribution::producer::execution::FrozenCurrentInspection::Current(value) =>
            FrozenCallerInspection::Current(performed.borrow().current(value, CurrentBasis::of(wave.shared.selected().product().observation()), selected.certify_current_record(CurrentBasis::of(wave.shared.selected().product().observation())))),
        crate::domain_computation::primary_graph::application_contribution::producer::execution::FrozenCurrentInspection::PendingExact(pending) => {
            match super::super::super::super::required_continuations::RegistryDelivery::from_pending(
                &pending, &wave.positioned, &runtime.output_demands, admission)? {
                super::super::super::super::required_continuations::RequiredDelivery::Delivery(delivery) => FrozenCallerInspection::RegistryDelivery(delivery),
                super::super::super::super::required_continuations::RequiredDelivery::Ready(ready) => FrozenCallerInspection::Upstream(ready),
                super::super::super::super::required_continuations::RequiredDelivery::Unavailable => FrozenCallerInspection::NotCurrent,
            }
        }
        crate::domain_computation::primary_graph::application_contribution::producer::execution::FrozenCurrentInspection::NotCurrent => {
            match super::super::super::super::required_continuations::RegistryDelivery::from_required_claim(
                selected, &runtime.output_demands, admission)? {
                Some(delivery) => FrozenCallerInspection::RegistryDelivery(delivery),
                None => FrozenCallerInspection::NotCurrent,
            }
        },
    })
}
pub(in super::super) enum FrozenCallerInspection<Schema: ApplicationSchema> {
    Upstream(SelectedReadyReadmission),
    RegistryDelivery(super::super::super::super::required_continuations::RegistryDelivery<Schema>),
    Current(CurrentProof<Arc<WorthQueryOutputDemandSettlement>>),
    NotCurrent,
}

pub(in super::super) struct FrozenCaller<'a, Schema: ApplicationSchema> {
    pub selected: &'a SelectedReadyReadmission,
    pub installed: &'a InstalledProducerProvider<Schema>,
    pub contacts: usize,
}
