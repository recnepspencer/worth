use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
    },
    application_entry::WorthQueryApplicationRequestExt,
    product::WorthQueryProductBranch,
};

use super::adapters::block_on;
use super::application::{admit_identity_adapter, ExampleApplication};
use super::application_entry::TemporalIntentRead;
use super::schema::{IntentQueryResult, TemporalHostSchema};

pub fn principal(
    application: &ExampleApplication,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<TemporalHostSchema> {
    block_on(admit_identity_adapter(application.runtime.installed_schema()).authenticate((), scope))
        .expect("the example identity must authenticate")
}

pub fn read_input(
    application: &ExampleApplication,
    branch: WorthQueryProductBranch,
    principal: &WorthQueryAuthenticatedExternalPrincipal<TemporalHostSchema>,
    scope: &WorthQueryRequestScope,
) -> String {
    read_row(application, branch, principal, scope).input
}

pub fn read_row(
    application: &ExampleApplication,
    branch: WorthQueryProductBranch,
    principal: &WorthQueryAuthenticatedExternalPrincipal<TemporalHostSchema>,
    scope: &WorthQueryRequestScope,
) -> IntentQueryResult {
    application
        .runtime
        .request(principal, scope)
        .on_branch(branch)
        .query(TemporalIntentRead {
            identity: "intent-1".to_owned(),
        })
        .execute()
        .expect("the application entry must execute the exact product read")
        .rows()[0]
        .clone()
}
