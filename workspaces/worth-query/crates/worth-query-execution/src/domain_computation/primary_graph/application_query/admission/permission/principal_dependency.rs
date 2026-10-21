//! Query permission consumes the principal owner's admitted retained capture.

use worth_query_declaration::facade::application_schema::ApplicationSchema;

use super::denial;
use crate::domain_computation::authorization::WorthQueryPrincipalCurrentnessDependency;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenialKind;
use crate::domain_computation::primary_graph::capture_principal_currentness_admitted;
use crate::domain_computation::primary_graph::PrincipalCurrentnessCaptureStop;
use crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;

pub(super) fn capture<Schema, Principal, PrincipalIdentity>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    session: WorthQueryGraphWorkSessionIdentity,
) -> Result<
    WorthQueryPrincipalCurrentnessDependency,
    super::WorthQueryApplicationQueryAdmissionDenial,
>
where
    Schema: ApplicationSchema,
{
    capture_principal_currentness_admitted(runtime, principal, session).map_err(|stop| match stop {
        PrincipalCurrentnessCaptureStop::MissingGraph
        | PrincipalCurrentnessCaptureStop::MissingBinding => denial(
            WorthQueryApplicationQueryAdmissionDenialKind::StalePrincipal,
            principal.binding(),
        ),
    })
}
