//! Clean certification for a required cue on the wave's one selected Product.

use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::producer::demand::{
        MatchedRequiredPredecessors, ResolvedRequiredPredecessors,
    },
    application_query::{FreshQueryPermissionStop, PreparedApplicationQueryPermission},
    product_operation::{SelectedQueryBasisRetentionStop, SharedSelectedProductOperation},
};
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

type BoundParameters<Schema, Binding> = <<SourceBinding<Schema, Binding> as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value;
type BoundResult<Schema, Binding> = <<SourceBinding<Schema, Binding> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

/// The caller's Fresh continuation consumes this exact prepared Query
/// permission only when the current-output proof needs disclosure. Current
/// and pending paths keep the same short authority path and do not invoke it.
#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn certify_ready_on_selected_with_disclosure<
    'basis,
    Schema,
    Binding,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    installed: &WorthQueryInstalledApplicationQueryBinding<Schema, SourceBinding<Schema, Binding>>,
    validated_principal: &worth_query_installation::facade::WorthQueryValidatedPrincipalBinding,
    external_principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    shared: &'basis SharedSelectedProductOperation<'_, Schema>,
    positioned: &'basis PositionedRelationalSnapshot,
    retained: &WorthQueryObservedSource<SourceQuery<Schema, Binding>>,
    candidate: Option<&'basis AcceptedCurrentCandidate>,
    resolved: Option<&ResolvedRequiredPredecessors<'_, Schema>>,
    limits: WorthQueryOutputDemandLimits,
    producer_contacts_in_this_demand: usize,
    admission: &mut InvalidationEditAdmission,
    on_disclosure: impl for<'prepared> FnOnce(
        PreparedApplicationQueryPermission<
            'prepared,
            Schema,
            SourceQuery<Schema, Binding>,
            BoundParameters<Schema, Binding>,
            BoundResult<Schema, Binding>,
            BoundPrincipal<Schema, Binding>,
            BoundPrincipalIdentity<Schema, Binding>,
            BoundScope<Schema, Binding>,
        >,
        Option<MatchedRequiredPredecessors<'_>>,
        &mut InvalidationEditAdmission,
    ) -> Result<
        super::super::required_cue::RequiredCueProgress<'basis, Schema>,
        ProducerExecutionStop,
    >,
) -> Result<super::super::required_cue::RequiredCueProgress<'basis, Schema>, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Binding>>,
{
    use super::super::required_cue::RequiredCueProgress;
    // The installed executor owns this exact source Query binding from cold
    // contribution setup. Its schema affinity is rechecked here before the
    // prepared permission uses the caller's selected Product.
    let schema_bytes = std::mem::size_of::<
        worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity,
    >();
    let installed_work = schema_bytes
        .checked_mul(3)
        .and_then(|work| work.checked_add(4))
        .and_then(|work| u64::try_from(work).ok())
        .ok_or_else(|| ready::ready_work_denial(""))?;
    admission
        .charge_external_work(installed_work)
        .map_err(|_| ready::ready_work_denial(""))?;
    let query = installed.query();
    if query.binding_identity() != &runtime.installed_schema.binding_identity() {
        return Err(denial(WorthQueryOutputDemandDenialKind::Superseded, "").into());
    }
    // The retained identity comparison reads two schema digests and one
    // Query digest. Static denial subjects need no initialized backing.
    let identity_work = 8u64
        .checked_add(64)
        .and_then(|work| work.checked_add(32))
        .and_then(|work| work.checked_add(3))
        .ok_or_else(|| ready::ready_work_denial(""))?;
    admission
        .charge_external_work(identity_work)
        .map_err(|_| ready::ready_work_denial(""))?;
    if retained.runtime_authority != runtime.runtime.authority_identity().as_u64()
        || retained.schema_binding != *query.binding_identity()
        || retained.query_identity != *query.identity()
    {
        return Err(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::Superseded,
            Binding::IDENTITY,
        )
        .into());
    }
    let installed_limits = runtime.resolve_application_query_limits(installed.limits());
    let maximum_work = NonZeroUsize::new(
        installed_limits
            .maximum_work()
            .get()
            .min(limits.producer_work())
            .min(admission.remaining_work()),
    )
    .ok_or_else(|| ready::ready_work_denial(Binding::IDENTITY))?;
    let (product, basis) = runtime
        .retain_selected_query_basis_admitted(shared, admission)
        .map_err(|stop| match stop {
            SelectedQueryBasisRetentionStop::Basis => denial(
                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                Binding::IDENTITY,
            )
            .into(),
            SelectedQueryBasisRetentionStop::Admission(stop) => {
                ready::ready_resource_denial(Binding::IDENTITY, stop)
            }
        })?;
    let controls = WorthQueryApplicationQueryControls::selected_read_one_shot(
        product,
        basis,
        installed_limits.maximum_results(),
        maximum_work,
        request,
        admission,
    )
    .map_err(|stop| ready::ready_resource_denial(Binding::IDENTITY, stop))?;
    runtime
        .with_fresh_retained_query_permission(
            installed,
            Some(validated_principal),
            external_principal,
            retained,
            shared.selected(),
            controls,
            admission,
            |permission, admission| {
                let Some(candidate) = candidate else {
                    return on_disclosure(permission, None, admission);
                };
                let Some(sealed) = permission
                    .seal_current_ready(admission)
                    .map_err(|error| query_admission_denied(Binding::IDENTITY, error))?
                else {
                    return on_disclosure(permission, None, admission);
                };
                let handle = &runtime.primary_provider.graph;
                let current = handle
                    .with_runtime(|relational| {
                        candidate.certify_current(
                            &handle.source_owner.invalidation_owner,
                            relational,
                            sealed.product(),
                            sealed.snapshot_handle(),
                            positioned,
                            admission,
                        )
                    })?
                    .map_err(|stop| ready::ready_currentness_denial(Binding::IDENTITY, stop))?;
                sealed
                    .validate_request(admission)
                    .map_err(|error| query_admission_denied(Binding::IDENTITY, error))?;
                match current {
                    CurrentAcceptedResult::Current(proof) => {
                        let Some(bound) = proof
                            .bind_product(admission)
                            .map_err(|stop| ready::ready_currentness_denial(Binding::IDENTITY, stop))?
                        else {
                            drop(sealed);
                            return on_disclosure(permission, None, admission);
                        };
                        // Binding the pinned recorded row may wait on its mutex.
                        // Preserve the same request's terminal interruption
                        // boundary before constructing a reusable settlement.
                        sealed
                            .validate_request(admission)
                            .map_err(|error| query_admission_denied(Binding::IDENTITY, error))?;
                        let settled = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement::from_current_accepted(
                            runtime,
                            bound,
                            Binding::IDENTITY,
                            <<Binding as WorthQueryApplicationProducerBinding<Schema>>::OutputFamily as super::super::super::WorthQueryProducerOutputFamily<Schema>>::IDENTITY,
                            producer_contacts_in_this_demand,
                            admission,
                        )?;
                        Ok(RequiredCueProgress::Current(settled))
                    }
                    CurrentAcceptedResult::PendingExact(mut pending) => {
                        if let Some(resolved) = resolved {
                            // Every pending consumed edge must have resolved on
                            // this wave before the consumer refreshes; the first
                            // unresolved one is the next upstream to schedule.
                            let mut matched = None;
                            loop {
                                let old = match resolved
                                    .match_pending(runtime, &pending, positioned, admission)
                                    .map_err(ProducerExecutionStop::ExecutionStopped)?
                                {
                                    Some(old) => Some(old),
                                    None => resolved
                                        .match_current_upstream(runtime, &pending, positioned, admission)
                                        .map_err(ProducerExecutionStop::ExecutionStopped)?,
                                };
                                let Some(old) = old else {
                                    break;
                                };
                                MatchedRequiredPredecessors::join(&mut matched, old, positioned, admission)
                                    .map_err(ProducerExecutionStop::ExecutionStopped)?;
                                let next = handle
                                    .with_runtime(|relational| {
                                        pending.next_pending(
                                            &handle.source_owner.invalidation_owner,
                                            relational,
                                            sealed.snapshot_handle(),
                                            admission,
                                        )
                                    })?
                                    .map_err(|stop| ready::ready_currentness_denial(
                                        Binding::IDENTITY,
                                        CurrentAcceptedStop::Closure(stop),
                                    ))?;
                                match next {
                                    Some(next) => pending = next,
                                    None => {
                                        drop(sealed);
                                        return on_disclosure(permission, matched, admission);
                                    }
                                }
                            }
                        }
                        // The evidence is an owned, ticketed pin. Only its
                        // selected-root borrow crosses this scoped callback.
                        let rebound = pending
                            .rebind_selected(positioned, admission)
                            .map_err(|stop| ready::ready_currentness_denial(
                                Binding::IDENTITY,
                                CurrentAcceptedStop::Closure(stop),
                            ))?;
                        Ok(match rebound {
                            Some(pending) => RequiredCueProgress::PendingExact(pending),
                            None => RequiredCueProgress::PendingUnresolved,
                        })
                    }
                    CurrentAcceptedResult::PendingUnresolved =>
                        Ok(RequiredCueProgress::PendingUnresolved),
                    CurrentAcceptedResult::NeedsDisclosure => {
                        drop(sealed);
                        on_disclosure(permission, None, admission)
                    }
                }
            },
        )
        .map_err(|stop| match stop {
            FreshQueryPermissionStop::Principal(error) => principal_rejected(Binding::IDENTITY, error),
            FreshQueryPermissionStop::Scope(error) => scope_rejected(Binding::IDENTITY, error),
            FreshQueryPermissionStop::Query(error) =>
                query_admission_denied(Binding::IDENTITY, error),
            FreshQueryPermissionStop::Inspect(error) => error,
        })
}
