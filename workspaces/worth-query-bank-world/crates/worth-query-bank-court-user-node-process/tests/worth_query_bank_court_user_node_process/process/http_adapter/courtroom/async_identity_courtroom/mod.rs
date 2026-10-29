pub(crate) use worth_query_bank_worlds::facade::courtroom::async_identity_courtroom::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "needs Docker: cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml -p worth-query-bank-court-user-node-process --test worth_query_bank_court_user_node_process async_identity_courtroom:: -- --ignored"]
async fn real_browser_identity_admits_and_real_revocation_fails_closed() {
    prove_real_teardown_after_failed_body().await;
    let world = InstalledIdentityWorld::install()
        .await
        .expect("real Docker identity world must install");
    let credential = acquire_browser_credential(
        &world.identity,
        &world.endpoints.webdriver_url(),
        &world.fixture,
        &world.callback,
        &world.scope,
    )
    .await
    .expect("real browser authorization should exchange");
    assert_eq!(format!("{credential:?}"), "AuthentikOidcCredential { .. }");
    let principal = world
        .identity
        .authenticate_credential(credential.clone(), &world.scope)
        .await
        .expect("real active Authentik token should admit");
    assert_eq!(
        principal.external_identity().subject(),
        world.fixture.primary_participant().username()
    );
    assert!(
        principal
            .attributes()
            .iter()
            .any(|attribute| attribute.key() == "email"),
        "real display claims must be retained as non-authoritative attributes"
    );
    assert_complete_actor_inventory(&world);

    let peer_credential = acquire_browser_credential_for(
        &world.identity,
        &world.endpoints.webdriver_url(),
        &world.fixture.participants()[1],
        &world.callback,
        &world.scope,
    )
    .await
    .expect("a second real user authorization should exchange");
    prove_real_hostile_credential_denials(
        &world.identity,
        &credential,
        &peer_credential,
        &world.scope,
    )
    .await;
    prove_real_wrong_audience_denial(
        credential,
        &world.endpoints,
        &world.fixture,
        &world.callback,
        &world.scope,
    )
    .await;
    prove_real_state_and_interruption_denials(
        &world.identity,
        &world.endpoints.webdriver_url(),
        &world.fixture,
        &world.callback,
    )
    .await;
    prove_rotation_revocation_and_expiration(&world).await;
    world
        .shutdown()
        .expect("courtroom drop must remove its containers and database volume");
}
