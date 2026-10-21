//! Retain principal currentness from the installed layout under one admission.

use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryAuthenticatedPrincipal;
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::authorization::WorthQueryPrincipalCurrentnessDependency;
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;

pub(in crate::domain_computation) enum PrincipalCurrentnessCaptureStop {
    MissingGraph,
    MissingBinding,
}

pub(in crate::domain_computation) fn capture_principal_currentness_admitted<
    Schema,
    Principal,
    PrincipalIdentity,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    session: WorthQueryGraphWorkSessionIdentity,
) -> Result<WorthQueryPrincipalCurrentnessDependency, PrincipalCurrentnessCaptureStop>
where
    Schema: ApplicationSchema,
{
    use PrincipalCurrentnessCaptureStop as Stop;
    // The selected binding header, runtime graph header and the base-six
    // lookup-bound loop all precede the variable comparison charge. Each
    // loop iteration grows the minimum key count, so usize::BITS is a finite
    // upper bound even on the widest installed catalog.

    let binding = principal.binding();

    // Stale graph/layout can return an owned binding subject. First refusal
    // remains allocation-free at the caller.

    let graph = runtime.runtime.primary_graph().ok_or(Stop::MissingGraph)?;

    let layout = graph
        .layout
        .principal_binding(binding)
        .ok_or(Stop::MissingBinding)?;

    Ok(WorthQueryPrincipalCurrentnessDependency::capture(
        session, principal, layout,
    ))
}
