use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;

pub(super) fn external_identity(subject: &str) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new("https://issuer.test.invalid", subject)
        .expect("test external identity should admit")
}
