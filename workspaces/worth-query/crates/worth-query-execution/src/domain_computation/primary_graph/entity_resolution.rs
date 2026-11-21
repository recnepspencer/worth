pub use worth_query_execution_lineage::facade::primary_graph::entity_resolution::*;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
pub use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, ApplicationSchema,
};
pub use super::{
    WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind,
    WorthQueryPrincipalResolutionMode,
};
pub(crate) mod product;
mod admitted_scope;
pub(in crate::domain_computation) use product::WorthQueryIssuedSelectedScope;

pub(super) fn admit_request(
    scope: &WorthQueryRequestScope,
    subject: &str,
) -> Result<(), WorthQueryEntityResolutionDenial> {
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeReadStop;
    match scope.interruption() {
        Some(WorthQueryRequestInterruption::Cancelled) => Err(
            WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::Cancelled),
                subject,
            ),
        ),
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => Err(
            WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::DeadlineElapsed),
                subject,
            ),
        ),
        None => Ok(()),
    }
}
