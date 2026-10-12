#[path = "async_identity_courtroom/mod.rs"]
mod support;

use support::authorization_flow::{acquire_browser_credential, acquire_browser_credential_for};
use support::callback::CallbackReceiver;
use support::credential_denials::{
    prove_real_hostile_credential_denials, prove_real_wrong_audience_denial,
};
use support::credential_lifecycle::prove_rotation_revocation_and_expiration;
use support::docker_world::DockerIdentityWorld;
use support::fixture::CourtroomIdentityRole;
use support::fixture::IdentityFixture;
use support::installed_identity::InstalledIdentityWorld;
use support::protocol_denials::prove_real_state_and_interruption_denials;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "needs Docker: cargo test --manifest-path workspaces/worth-query-bank-world/Cargo.toml -p bank-courtroom --test async_identity_courtroom -- --ignored"]
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

async fn prove_real_teardown_after_failed_body() {
    let callback = CallbackReceiver::bind()
        .await
        .expect("failure-path callback listener should bind");
    let fixture = IdentityFixture::dynamic(callback.redirect_url());
    let docker = DockerIdentityWorld::start(&fixture)
        .await
        .expect("failure-path Docker world must start");
    let project = docker.project_name();
    let directory = docker.directory_path();
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _owned_world = docker;
        panic!("intentional courtroom body failure");
    }));

    assert!(failure.is_err(), "the hostile body must really unwind");
    DockerIdentityWorld::require_project_absent(&project)
        .expect("unwinding must remove containers, volumes, and networks");
    assert!(
        !directory.exists(),
        "unwinding must remove the secret-bearing fixture directory"
    );
}

fn assert_complete_actor_inventory(world: &InstalledIdentityWorld) {
    assert_eq!(
        world.identity.mapped_principal_count(),
        CourtroomIdentityRole::ALL.len()
    );
    assert_eq!(
        world
            .fixture
            .participants()
            .iter()
            .map(|participant| participant.role())
            .collect::<Vec<_>>(),
        CourtroomIdentityRole::ALL
    );
}
