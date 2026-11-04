//! Actual Query principal, request and branch selected by the application owner.
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
    },
    declaration::application_operation::ApplicationMutationIntent,
    domain::ApplicationSchema,
    product::WorthQueryProductBranch,
};
pub struct WorthServerQueryMutationSelection<Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub(super) principal: WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(super) scope: WorthQueryRequestScope,
    pub(super) branch: WorthQueryProductBranch,
    pub(super) intent: Intent,
}
impl<Schema, Intent> WorthServerQueryMutationSelection<Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    /// Resolve the admitted Server caller and target through the application's
    /// actual authentication and branch selection. A scalar branch name grants
    /// no Query authority.
    pub fn new(
        principal: WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: WorthQueryRequestScope,
        branch: WorthQueryProductBranch,
        intent: Intent,
    ) -> Self {
        Self {
            principal,
            scope,
            branch,
            intent,
        }
    }
}
