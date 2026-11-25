//! admit identity adapter.
use super::*;

worth_query_decl::facade::worth_query_authentication_audience!(HostAudience);
worth_query_decl::facade::worth_query_authentication_method!(TestMethod);

pub fn admit_identity_adapter(
    schema: &domain::WorthQueryInstalledApplicationSchema<TemporalHostSchema>,
) -> admission::authenticated_principal::WorthQueryAdmittedAuthenticationAdapter<
    TemporalHostSchema,
    IdentityAdapter,
> {
    admission::authenticated_principal::admit_authentication_adapter(
        schema,
        admission::authenticated_principal::WorthQueryAuthenticationAdapterAdmission::new(
            admission::authenticated_principal::WorthQueryAuthenticationAudience::of(HostAudience),
            admission::authenticated_principal::WorthQueryAuthenticationMethod::of(TestMethod), worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()),
        IdentityAdapter,
    )
    .unwrap()
}
