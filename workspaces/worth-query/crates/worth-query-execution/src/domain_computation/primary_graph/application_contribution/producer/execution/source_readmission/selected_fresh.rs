//! Source disclosure for an exact required cue on the wave's selected Product.

use super::*;
use crate::domain_computation::primary_graph::{
    application_query::{PreparedApplicationQueryPermission, WorthQueryAdmittedOneShotStop},
    product_operation::SharedSelectedProductOperation,
};
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;

type Parameters<Schema, Binding> = <<SourceBinding<Schema, Binding> as ApplicationQueryBinding<
    Schema,
>>::ParameterBinding as ApplicationStructuredValueBinding>::Value;
type ResultValue<Schema, Binding> = <<SourceBinding<Schema, Binding> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

/// Consume the same permission used by the Clean probe. Its access owners stay
/// alive until the one-shot read returns an owned, inseparable disclosure.
#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn disclose_prepared_on_selected<
    'prepared,
    Schema,
    Binding,
>(
    runtime: &'prepared WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    permission: PreparedApplicationQueryPermission<
        'prepared,
        Schema,
        SourceQuery<Schema, Binding>,
        Parameters<Schema, Binding>,
        ResultValue<Schema, Binding>,
        BoundPrincipal<Schema, Binding>,
        BoundPrincipalIdentity<Schema, Binding>,
        BoundScope<Schema, Binding>,
    >,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    retained: &WorthQueryObservedSource<SourceQuery<Schema, Binding>>,
    shared: &SharedSelectedProductOperation<'_, Schema>,
    edition: super::super::super::InstalledProducerEdition,
    admission: &mut InvalidationEditAdmission,
) -> Result<
    super::super::super::demand::disclosure::FreshOutputDisclosure<
        SourceQuery<Schema, Binding>,
        SourceValue<Schema, Binding>,
    >,
    ProducerExecutionStop,
>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Binding>>,
{
    let plan = runtime
        .finish_prepared_application_query_permission(permission, admission)
        .map_err(|stop| query_admission_denied(Binding::IDENTITY, stop))?;
    let read = runtime
        .execute_application_query_one_shot_admitted(plan, admission)
        .map_err(|stop| match stop {
            WorthQueryAdmittedOneShotStop::Admission(stop) => {
                ready::ready_resource_denial(Binding::IDENTITY, stop)
            }
            WorthQueryAdmittedOneShotStop::WorkUnavailable
            | WorthQueryAdmittedOneShotStop::WorkCounterOverflow
            | WorthQueryAdmittedOneShotStop::WorkAccountingMismatch => {
                ready::ready_work_denial(Binding::IDENTITY)
            }
            WorthQueryAdmittedOneShotStop::Execution(stop) => {
                query_execution_denied(Binding::IDENTITY, stop)
            }
        })?;
    super::super::super::demand::disclosure::validate_readmitted_on_selected(
        runtime,
        retained,
        principal,
        request,
        branch,
        shared,
        read.into_admitted_disclosed().into_output_demand_source(),
        edition,
        admission,
        Binding::IDENTITY,
    )
    .map_err(ProducerExecutionStop::ExecutionStopped)
}
