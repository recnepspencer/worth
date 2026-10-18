use super::super::WorthQueryApplicationRequestQueryDenial as ItemDenial;
use super::*;
use std::num::NonZeroUsize;
use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedDisclosedApplicationResult, WorthQueryApplicationBatchReadDenial,
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryBatchMemory,
    WorthQueryPrincipalResolutionMode, WorthQueryProductQueryControls,
    WorthQueryRetainedBatchQueryAdmissionDenial,
};

pub(super) enum BatchExecutionPolicy {
    Explicit(WorthQueryApplicationQueryBatchLimits),
    Installed(Option<WorthQueryApplicationQueryBatchLimits>),
}

pub(super) fn execute<Schema: ApplicationSchema, Intent: ApplicationQueryIntent<Schema>>(
phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    request: WorthQueryApplicationQueryBatchRequest<'_, '_, '_, Schema, Intent>,
    policy: BatchExecutionPolicy,
    query_limits: Option<(NonZeroUsize, NonZeroUsize)>,
) -> Result<crate::domain_computation::WorthQueryPublishedApplicationQueryBatch<
    <Intent::Binding as ApplicationQueryBinding<Schema>>::Query,
    <<Intent::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
>, WorthQueryApplicationQueryBatchDenial>
where
    <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    <<Intent::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<Schema, <Intent::Binding as ApplicationQueryBinding<Schema>>::Query>,
{
    checkpoint(request.scope)?;
    let binding = request
        .application
        .installed_schema()
        .installed_query_binding::<Intent::Binding>()
        .map_err(|cause| item(0, ItemDenial::BindingInstallation(cause)))?;
    let admission = match policy {
        BatchExecutionPolicy::Explicit(limits) => {
            WorthQueryApplicationQueryBatchAdmission::new(limits)
        }
        BatchExecutionPolicy::Installed(limits) => {
            let plan = request
                .application
                .plan_application_query_batch_reads(&binding, request.intents.len())
                .map_err(|denial| resource(None, denial))?;
            let admission = request
                .application
                .application_query_batch_from_installed_reads(&[plan])
                .map_err(|denial| resource(None, denial))?;
            match limits {
                Some(limits) => admission.narrow_and_freeze_planned_reads(limits),
                None => admission.freeze_planned_reads(),
            }
            .map_err(|denial| resource(None, denial))?;
            admission
        }
    };
    admission
        .admit_items(request.intents.len())
        .map_err(|denial| resource(None, denial))?;
    let item_limits = request
        .application
        .resolve_application_query_limits(binding.limits());
    let item_limits = match query_limits {
        Some((results, work)) => item_limits
            .narrow(results, work)
            .map_err(|cause| item(0, ItemDenial::Limit(cause)))?,
        None => item_limits,
    };
    let selected = request
        .application
        .on_branch(request.branch)
        .select(phase)
        .map_err(|cause| item(0, ItemDenial::ProductSelection(cause)))?;
    let mut retained = request
        .application
        .select_application_read_observation(phase, &request.observation)
        .map_err(|cause| item(0, ItemDenial::ProductSelection(cause)))?;
    let vector_bytes = request.intents.len().checked_mul(
        std::mem::size_of::<WorthQueryAdmittedDisclosedApplicationResult<
            <Intent::Binding as ApplicationQueryBinding<Schema>>::Query,
            <<Intent::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
        >>() + std::mem::size_of::<WorthQueryApplicationQueryBatchMemory>()
    ).ok_or_else(|| resource(None, WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow))?;
    let staging_claim = admission
        .claim_publication_memory(vector_bytes)
        .map_err(|denial| resource(None, denial))?;
    let mut staged = Vec::with_capacity(request.intents.len());
    let mut claims = Vec::with_capacity(request.intents.len());
    for (index, intent) in request.intents.into_iter().enumerate() {
        checkpoint(request.scope)?;
        // Application-authored parameter/scope callbacks precede framework
        // admission; their internals are not part of the read-work contract.
        let parameters = intent.parameters();
        let scope_binding = intent.into_scope();
        let principal = selected
            .resolve_authenticated_principal(
                phase,
                binding.principal_binding(),
                request.principal,
                request.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(|cause| item(index, ItemDenial::PrincipalResolution(cause)))?;
        let (field, value) = scope_binding.into_field_parts(principal.principal_identity());
        let scope = selected
            .resolve_entity(
                phase,
                field,
                value,
                request.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(|cause| item(index, ItemDenial::ScopeResolution(cause)))?;
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        let query_maximum = NonZeroUsize::new(binding.query().read_graph().maximum_result_count())
            .unwrap_or(NonZeroUsize::MAX);
        let controls = WorthQueryProductQueryControls::new(
            item_limits.maximum_results().min(query_maximum),
            item_limits.maximum_work(),
            request.scope,
        );
        let plan = selected
            .admit_retained_application_query_from_selection(
                phase,
                &mut retained,
                binding.query(),
                &access,
                parameters,
                controls,
                &admission,
            )
            .map_err(|cause| match cause {
                WorthQueryRetainedBatchQueryAdmissionDenial::Admission(cause) => {
                    item(index, ItemDenial::Admission(cause))
                }
                WorthQueryRetainedBatchQueryAdmissionDenial::Resource(denial) => {
                    resource(Some(index), denial)
                }
            })?;
        let result = request
            .application
            .execute_application_query_one_shot_in_batch(phase, plan, &admission)
            .map_err(|cause| match cause {
                WorthQueryApplicationBatchReadDenial::Resource(denial) => {
                    resource(Some(index), denial)
                }
                WorthQueryApplicationBatchReadDenial::Execution(cause) => {
                    item(index, ItemDenial::Execution(cause))
                }
            })?;
        let (result, claim) = result.into_parts();
        staged.push(result.into_admitted_disclosed());
        claims.push(claim);
    }
    checkpoint(request.scope)?;
    crate::domain_computation::application_batch_result::publish_batch(
        staged,
        claims,
        staging_claim,
        &admission,
    )
    .map_err(|denial| resource(None, denial))
}

fn item(index: usize, cause: ItemDenial) -> WorthQueryApplicationQueryBatchDenial {
    WorthQueryApplicationQueryBatchDenial::Item { index, cause }
}
fn resource(
    item: Option<usize>,
    denial: WorthQueryApplicationQueryBatchResourceDenial,
) -> WorthQueryApplicationQueryBatchDenial {
    WorthQueryApplicationQueryBatchDenial::Resource { item, denial }
}
fn checkpoint(scope: &WorthQueryRequestScope) -> Result<(), WorthQueryApplicationQueryBatchDenial> {
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
    match scope.interruption() {
        Some(WorthQueryRequestInterruption::Cancelled) => {
            Err(WorthQueryApplicationQueryBatchDenial::Cancelled)
        }
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => {
            Err(WorthQueryApplicationQueryBatchDenial::DeadlineExceeded)
        }
        None => Ok(()),
    }
}
