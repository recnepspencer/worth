//! admit identity adapter.
use super::*;

pub(in super::super) fn admit_identity_adapter(
    schema: &domain::WorthQueryInstalledApplicationSchema<TemporalHostSchema>,
) -> admission::authenticated_principal::WorthQueryAdmittedAuthenticationAdapter<
    TemporalHostSchema,
    IdentityAdapter,
> {
    admission::authenticated_principal::admit_authentication_adapter(
        schema,
        admission::authenticated_principal::WorthQueryAuthenticationAdapterAdmission::new(
            admission::authenticated_principal::WorthQueryAuthenticationAudience::new("host")
                .unwrap(),
            admission::authenticated_principal::WorthQueryAuthenticationMethod::new("test")
                .unwrap(),
        ),
        IdentityAdapter,
    )
    .unwrap()
}
