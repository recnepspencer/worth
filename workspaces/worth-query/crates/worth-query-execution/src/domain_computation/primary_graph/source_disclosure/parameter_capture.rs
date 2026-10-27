//! Opaque authored projection followed by admitted framework parameter preparation.
use super::OriginalSourceDisclosure;
mod stage;
use crate::domain_computation::primary_graph::application_contribution::execution_denial::{
    publication_execution_kind, scope_stop_kind,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use stage::{checkpoint, run_stage};
use worth_execution::{
    ChargedBytes, ExecutionRequest, LeaseDenial, MapKernelFailure, WorkCeilingDenial,
};
use worth_query_admission::facade::application_query::{
    admit_application_query_parameters_admitted, WorthQueryAdmittedApplicationQueryParameters,
    WorthQueryParameterPreparationStop,
};
use worth_query_declaration::facade::{
    application_query::{ApplicationQueryBinding, ApplicationQueryParameterSet},
    application_schema::ApplicationSchema,
};

struct Projected<Query>(ApplicationQueryParameterSet<Query>);
impl<Query> ChargedBytes for Projected<Query> {
    fn additional_charged_bytes(&self) -> u64 {
        self.0
            .owned_allocation_capacity_bytes()
            .and_then(|bytes| u64::try_from(bytes).ok())
            .unwrap_or(u64::MAX)
    }
}
struct Prepared {
    parameters: WorthQueryAdmittedApplicationQueryParameters,
    framework: u64,
}
impl ChargedBytes for Prepared {
    fn additional_charged_bytes(&self) -> u64 {
        self.parameters
            .retained_owned_capacity_bytes()
            .and_then(|bytes| u64::try_from(bytes).ok())
            .and_then(|bytes| bytes.checked_add(self.framework))
            .unwrap_or(u64::MAX)
    }
}

pub(in crate::domain_computation::primary_graph) fn capture_parameters<Schema, Binding>(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    framework_bytes: usize,
    project: impl FnOnce() -> Result<ApplicationQueryParameterSet<Binding::Query>, Denial>,
) -> Result<OriginalSourceDisclosure, Denial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
{
    let installed = runtime
        .installed_schema()
        .installed_query_binding::<Binding>()
        .map_err(|_| {
            Denial::new(
                Kind::SourceDisclosureCaptureDenied,
                "source disclosure query is unavailable",
            )
        })?;
    let query = installed.query();
    // The application mapping is opaque. Its returned heap is admitted after
    // materialization, before framework retention; it is not a preflight estimate.
    let (Projected(parameters), projection_memory) = run_stage(execution, 0, |context, _| {
        checkpoint(context, 1)?;
        project().map(Projected).map_err(MapKernelFailure::Domain)
    })?;
    let framework = framework_bytes
        .checked_add(std::mem::size_of::<
            WorthQueryAdmittedApplicationQueryParameters,
        >())
        .and_then(|bytes| bytes.checked_add(2 * std::mem::size_of::<usize>()))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(overflow)?;
    let (Prepared { parameters, .. }, memory) =
        run_stage(execution, framework, |context, mut retained| {
            let parameters =
                admit_application_query_parameters_admitted(query, parameters, |work, bytes| {
                    let work =
                        u64::try_from(work).map_err(|_| MapKernelFailure::Domain(overflow()))?;
                    checkpoint(context, work)?;
                    if let Some(retained) = retained.as_deref_mut() {
                        let bytes = u64::try_from(bytes)
                            .ok()
                            .and_then(|bytes| retained.bytes().checked_add(bytes))
                            .ok_or_else(|| MapKernelFailure::Domain(overflow()))?;
                        retained.resize(bytes).map_err(|cause| {
                            MapKernelFailure::Domain(request_stopped(WorkCeilingDenial::Admission(
                                LeaseDenial::MemoryExhausted(cause),
                            )))
                        })?;
                    }
                    Ok(())
                })
                .map_err(|stop| match stop {
                    WorthQueryParameterPreparationStop::Admission(cause)
                    | WorthQueryParameterPreparationStop::Preparation { cause, .. } => cause,
                    WorthQueryParameterPreparationStop::Parameters(cause) => {
                        MapKernelFailure::Domain(Denial::source_disclosure_parameters(cause))
                    }
                    WorthQueryParameterPreparationStop::AccountingOverflow => {
                        MapKernelFailure::Domain(overflow())
                    }
                })?;
            Ok(Prepared {
                parameters,
                framework,
            })
        })?;
    // The opaque mapping's temporary backing has now been released by admission.
    drop(projection_memory);
    Ok(OriginalSourceDisclosure::captured(
        query.identity().clone(),
        parameters,
        memory,
    ))
}

fn stopped(stop: worth_execution::ScopeStop<Denial>) -> Denial {
    match scope_stop_kind(stop) {
        Ok(denial) => denial,
        Err(kind) => Denial::source_disclosure_execution(kind),
    }
}
fn request_stopped(cause: WorkCeilingDenial) -> Denial {
    Denial::source_disclosure_execution(publication_execution_kind(cause))
}
fn overflow() -> Denial {
    request_stopped(WorkCeilingDenial::Admission(
        LeaseDenial::ChargedBytesOverflow,
    ))
}
