//! Clean certification for a required cue on the wave's one selected Product.

use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::demand::MatchedRequiredPredecessors;
use crate::domain_computation::primary_graph::application_contribution::producer::demand::ResolvedRequiredPredecessors;
use crate::domain_computation::primary_graph::application_query::FreshQueryPermissionStop;
use crate::domain_computation::primary_graph::application_query::PreparedApplicationQueryPermission;
use crate::domain_computation::primary_graph::product_operation::SelectedQueryBasisRetentionStop;
use crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

mod rebound_consumption;

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
    Outcome,
    Schema,
    Binding,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
    producer_contacts_in_this_demand: usize,
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
    ) -> Result<Outcome, ProducerExecutionStop>,
) -> Result<Outcome, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Outcome: From<super::super::required_cue::ReadyInspection<'basis>>,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Binding>>,
{
    use super::super::required_cue::ReadyInspection;
    // The installed executor owns this exact source Query binding from cold
    // contribution setup. Its schema affinity is rechecked here before the
    // prepared permission uses the caller's selected Product.

    let query = installed.query();
    if query.binding_identity() != &runtime.installed_schema.binding_identity() {
        return Err(denial(WorthQueryOutputDemandDenialKind::Superseded, "").into());
    }
    // The retained identity comparison reads two schema digests and one
    // Query digest. Static denial subjects need no initialized backing.

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
    let (product, basis) = runtime
        .retain_selected_query_basis_admitted(shared)
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
        request,
    )
    .map_err(|stop| ready::ready_resource_denial(Binding::IDENTITY, stop))?;
    runtime
        .with_fresh_retained_query_permission(execution, installed, Some(validated_principal), external_principal, retained, shared.selected(), controls, |permission| {
                let Some(candidate) = candidate else {
                    return on_disclosure(permission, None);
                };
                let Some(sealed) = permission
                    .seal_current_ready(execution)
                    .map_err(|error| query_admission_denied(Binding::IDENTITY, error))?
                else {
                    return on_disclosure(permission, None);
                };
                let handle = &runtime.primary_provider.graph;
                let current = handle
                    .with_runtime(execution, |relational| {
                        candidate.certify_current(execution,
                            &handle.source_owner.invalidation_owner,
                            relational,
                            sealed.product(),
                            sealed.snapshot_handle(),
                            positioned,
                        )
                    })
                    .map_err(|stop| ready::ready_currentness_denial(Binding::IDENTITY, stop))?;
                sealed
                    .validate_request()
                    .map_err(|error| query_admission_denied(Binding::IDENTITY, error))?;
                match current {
                    CurrentAcceptedResult::Current(proof) => {
                        let Some(bound) = proof
                            .bind_product()
                            .map_err(|stop| ready::ready_currentness_denial(Binding::IDENTITY, stop))?
                        else {
                            drop(sealed);
                            return on_disclosure(permission, None);
                        };
                        // Binding the pinned recorded row may wait on its mutex.
                        // Preserve the same request's terminal interruption
                        // boundary before constructing a reusable settlement.
                        sealed
                            .validate_request()
                            .map_err(|error| query_admission_denied(Binding::IDENTITY, error))?;
                        let settled = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement::from_current_accepted(
                            runtime,
                            bound,
                            Binding::IDENTITY,
                            <<Binding as WorthQueryApplicationProducerBinding<Schema>>::OutputFamily as super::super::super::WorthQueryProducerOutputFamily<Schema>>::IDENTITY,
                            producer_contacts_in_this_demand,
                        )?;
                        Ok(ReadyInspection::Current(settled).into())
                    }
                    CurrentAcceptedResult::PendingExact(mut pending) => {
                        if let Some(resolved) = resolved {
                            // Every pending consumed edge must have resolved on
                            // this wave before the consumer refreshes; the first
                            // unresolved one is the next upstream to schedule.
                            let consumer = candidate.retain_recorded_identity()
                                .map_err(|stop| ready::ready_resource_denial(Binding::IDENTITY, stop))?;
                            let mut matched = None;
                            loop {
                                let old = match resolved
                                    .match_pending(runtime, &pending, positioned)
                                    .map_err(ProducerExecutionStop::ExecutionStopped)?
                                {
                                    Some(old) => Some(old),
                                    None => resolved
                                        .match_current_upstream(runtime, &pending, positioned)
                                        .map_err(ProducerExecutionStop::ExecutionStopped)?,
                                };
                                let Some(old) = old else {
                                    break;
                                };
                                let (old, successor) = old.into_parts();
                                let evidence = rebound_consumption::retain(execution, runtime, &successor, &pending, sealed.product(),
                                    sealed.snapshot_handle(), positioned)?;

                                MatchedRequiredPredecessors::join(&mut matched, &runtime.output_demands, &consumer, old, evidence, positioned)
                                    .map_err(ProducerExecutionStop::ExecutionStopped)?;
                                let next = handle
                                    .with_runtime(execution, |relational| {
                                        pending.next_pending(execution,
                                            &handle.source_owner.invalidation_owner,
                                            relational,
                                            sealed.snapshot_handle(),
                                        )
                                    })
                                    .map_err(|stop| ready::ready_currentness_denial(
                                        Binding::IDENTITY,
                                        CurrentAcceptedStop::Closure(stop),
                                    ))?;
                                match next {
                                    Some(next) => pending = next,
                                    None => {
                                        drop(sealed);
                                        return on_disclosure(permission, matched);
                                    }
                                }
                            }
                        }
                        // The evidence is an owned, ticketed pin. Only its
                        // selected-root borrow crosses this scoped callback.
                        let rebound = pending
                            .rebind_selected(positioned)
                            .map_err(|stop| ready::ready_currentness_denial(
                                Binding::IDENTITY,
                                CurrentAcceptedStop::Closure(stop),
                            ))?;
                        Ok(match rebound {
                            Some(pending) => ReadyInspection::PendingExact(pending),
                            None => ReadyInspection::PendingUnresolved,
                        }.into())
                    }
                    CurrentAcceptedResult::PendingUnresolved =>
                        Ok(ReadyInspection::PendingUnresolved.into()),
                    CurrentAcceptedResult::NeedsDisclosure => {
                        let matched = if candidate.consumes_outputs() {
                            let consumer = candidate.retain_recorded_identity()
                                .map_err(|stop| ready::ready_resource_denial(Binding::IDENTITY, stop))?;
                            Some(MatchedRequiredPredecessors::retain_consumer(
                                &runtime.output_demands, &consumer, positioned,
                            ).map_err(ProducerExecutionStop::ExecutionStopped)?)
                        } else { None };
                        drop(sealed);
                        on_disclosure(permission, matched)
                    }
                }
            })
        .map_err(|stop| match stop {
            FreshQueryPermissionStop::Principal(error) => principal_rejected(Binding::IDENTITY, error),
            FreshQueryPermissionStop::Scope(error) => scope_rejected(Binding::IDENTITY, error),
            FreshQueryPermissionStop::Query(error) =>
                query_admission_denied(Binding::IDENTITY, error),
            FreshQueryPermissionStop::Inspect(error) => error,
        })
}
