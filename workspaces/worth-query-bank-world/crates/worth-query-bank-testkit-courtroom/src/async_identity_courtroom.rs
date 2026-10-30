#[path = "async_identity_courtroom/mod.rs"]
pub mod support;

pub use support::authorization_flow::{acquire_browser_credential, acquire_browser_credential_for};
pub use support::callback::CallbackReceiver;
pub use support::credential_denials::{
    prove_real_hostile_credential_denials, prove_real_wrong_audience_denial,
};
pub use support::credential_lifecycle::prove_rotation_revocation_and_expiration;
pub use support::docker_world::DockerIdentityWorld;
pub use support::fixture::CourtroomIdentityRole;
pub use support::fixture::IdentityFixture;
pub use support::installed_identity::InstalledIdentityWorld;
pub use support::protocol_denials::prove_real_state_and_interruption_denials;

pub async fn prove_real_teardown_after_failed_body() {
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

pub fn assert_complete_actor_inventory(world: &InstalledIdentityWorld) {
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
