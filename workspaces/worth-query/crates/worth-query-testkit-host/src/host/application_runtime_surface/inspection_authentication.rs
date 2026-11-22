//! Actual admitted authentication for the Host identity fixture.
use std::time::Duration;
use worth_query_decl::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_host::facade::{
    admission::authenticated_principal::*, domain::WorthQueryInstalledApplicationSchema,
};

pub fn authenticate<Schema: worth_query_decl::facade::application_schema::ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<Schema> {
    let principal = WorthQueryLocalProcessPrincipal::new(
        "worth.query.host.inspection.authentication.v1",
        WorthQueryExternalPrincipalIdentity::new("https://issuer.example", "subject").unwrap(),
        WorthQueryAuthenticationAudience::new("host").unwrap(),
        WorthQueryAuthenticationMethod::new("test").unwrap(),
        Duration::from_secs(3600),
    )
    .unwrap();
    admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("host").unwrap(),
            WorthQueryAuthenticationMethod::new("test").unwrap(),
            WorthQueryAdmissionClock::system(),
        ),
        principal,
    )
    .unwrap()
    .authenticate_local(scope)
    .unwrap()
}
