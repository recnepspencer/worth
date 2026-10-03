//! Query permission consumes the principal owner's admitted retained capture.

use worth_query_declaration::facade::application_schema::ApplicationSchema;

use super::{denial, resource_denial, work_denial};
use crate::domain_computation::authorization::WorthQueryPrincipalCurrentnessDependency;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenialKind;
use crate::domain_computation::primary_graph::{
    capture_principal_currentness_admitted, InvalidationEditAdmission,
    PrincipalCurrentnessCaptureStop, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;

pub(super) fn capture<Schema, Principal, PrincipalIdentity>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    session: WorthQueryGraphWorkSessionIdentity,
    admission: &mut InvalidationEditAdmission,
) -> Result<
    WorthQueryPrincipalCurrentnessDependency,
    super::WorthQueryApplicationQueryAdmissionDenial,
>
where
    Schema: ApplicationSchema,
{
    capture_principal_currentness_admitted(runtime, principal, session, admission).map_err(|stop| {
        match stop {
            PrincipalCurrentnessCaptureStop::MissingGraph
            | PrincipalCurrentnessCaptureStop::MissingBinding => denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StalePrincipal,
                principal.binding(),
            ),
            PrincipalCurrentnessCaptureStop::Admission(stop) => resource_denial(stop),
            PrincipalCurrentnessCaptureStop::AccountingOverflow => work_denial(),
        }
    })
}
