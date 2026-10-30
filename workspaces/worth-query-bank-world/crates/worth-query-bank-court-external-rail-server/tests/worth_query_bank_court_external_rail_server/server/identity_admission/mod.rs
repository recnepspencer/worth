pub(crate) use worth_query_bank_testkit_server::facade::identity_admission::*;

#[test]
fn dynamically_provisioned_external_identity_resolves_to_a_sealed_bank_principal() {
    let identity = DynamicIdentity::new("enabled-principal");
    let world = runtime([BankPrincipalSeed::enabled(
        BankPrincipalId::new(41).expect("nonzero principal id should admit"),
        identity.external(),
    )]);
    let scope = request_scope();

    let principal = block_on(world.runtime.authenticate_with(
        &world.authentication,
        CausalCredential::for_identity(&identity),
        &scope,
    ))
    .expect("enabled dynamic identity should resolve");

    assert_eq!(world.runtime.mapped_principal_count(), 1);
    assert_eq!(
        principal.principal_id(),
        BankPrincipalId::new(41).unwrap(),
        "the bank identity must come from the typed primary-graph principal field"
    );
    assert_eq!(
        principal.external_identity().subject(),
        identity.external().subject()
    );
    assert_eq!(
        format!("{principal:?}"),
        "BankAuthenticatedPrincipal { .. }",
        "authority diagnostics must not disclose principal identity"
    );
    assert_eq!(principal.examined_candidate_count(), 1);
    world
        .runtime
        .validate(&principal, &scope)
        .expect("fresh bank principal should remain usable");
}
