//! Actual admitted authentication for the Host identity fixture.
use std::time::Duration;
use worth_query_decl::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_host::facade::{
    admission::authenticated_principal::*, domain::WorthQueryInstalledApplicationSchema,
};

worth_query_decl::facade::worth_query_authentication_audience!(HostAudience);
worth_query_decl::facade::worth_query_principal_issuer!(IssuerExampleIssuer);
worth_query_decl::facade::worth_query_authentication_method!(TestMethod);

pub fn authenticate<Schema: worth_query_decl::facade::application_schema::ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<Schema> {
    let principal = WorthQueryLocalProcessPrincipal::new(
        WorthQueryExternalPrincipalIdentity::new(IssuerExampleIssuer, "subject").unwrap(),
        HostAudience,
        TestMethod,
        Duration::from_secs(3600),
    )
    .unwrap();
    admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::of(HostAudience),
            WorthQueryAuthenticationMethod::of(TestMethod),
            WorthQueryAdmissionClock::system(),
        ),
        principal,
    )
    .unwrap()
    .authenticate_local(scope)
    .unwrap()
}
