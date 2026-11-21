pub use super::{
    WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind,
    WorthQueryPrincipalResolutionMode,
};
pub use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
pub use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, ApplicationSchema,
};
pub use worth_query_execution_lineage::facade::primary_graph::entity_resolution::*;
mod admitted_scope;
pub mod product;
pub use product::WorthQueryIssuedSelectedScope;

pub fn admit_request(
    scope: &WorthQueryRequestScope,
    subject: &str,
) -> Result<(), WorthQueryEntityResolutionDenial> {
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeReadStop;
    match scope.interruption() {
        Some(WorthQueryRequestInterruption::Cancelled) => {
            Err(WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::Cancelled),
                subject,
            ))
        }
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => {
            Err(WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::DeadlineElapsed),
                subject,
            ))
        }
        None => Ok(()),
    }
}
