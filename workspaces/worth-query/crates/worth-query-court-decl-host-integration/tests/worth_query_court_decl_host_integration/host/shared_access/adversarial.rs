//! Adversarial framing, authentication, budget, and single-owner boundary courts.

use std::time::Duration;

use worth_foundational::facade::HomeIdentity;
use worth_query_host::facade::{
    shared_access::{
        AuthenticationRefusalReason, BudgetRefusalReason, ProtocolRefusalReason, RemoteAuthConfig,
        RemoteAuthKey, RemoteClientLimits, RemoteServerLimits, RemoteTokenIssueRequest,
        RemoteWatchPollOutcome, ScopeAccess, SharedAccessDenial, WatchRefusalReason,
        WorthQueryRemoteClient, WorthQueryRemoteServer,
    },
    WorthQueryApplicationHomeLocation,
};
use worth_query_testkit_host::facade::host::checkpoint_capture;

use super::dst::{issue_principal_token, make_auth_config, tamper_token};
use super::mutation_handlers::SharedProjectProgram;
use super::mutations::{alice_identity, bob_identity, ProjectId};
use super::program::open_shared_home;
use super::query::ProjectQueryParams;
use super::schema::SharedProjectSchema;

/// Certifies pre-allocation frame bounds, bad headers, tampered/cross-home/unknown-key auth,
/// cross-principal watch isolation, watch capacity budget, MemoryOnly refusal, and single-owner lock.
#[test]
fn adversarial_framing_auth_budget_and_single_owner_boundaries() {
    let dir = worth_query_testkit_host::facade::host::application_home::location()
        .with_extension("shared-adversarial");
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    let home = open_exclusive_directory_home(&dir, execution);
    home.grant_scope_access(
        ProjectId::project1(),
        alice_identity(),
        ScopeAccess::read_and_watch(),
    )
    .unwrap();
    home.grant_scope_access(
        ProjectId::project1(),
        bob_identity(),
        ScopeAccess::read_and_watch(),
    )
    .unwrap();

    let limits = RemoteServerLimits::default()
        .with_max_frame_bytes(2048)
        .unwrap()
        .with_max_active_watches(1, 1)
        .unwrap();
    let auth_config = make_auth_config().with_limits(limits);
    let mut server = home
        .serve_remote("127.0.0.1:0".parse().unwrap(), auth_config.clone())
        .unwrap();

    assert_framing_rejections(&mut server, &auth_config, &home);
    assert_auth_rejections(&mut server, &auth_config, &home);
    assert_watch_budget_and_owner_isolation(&mut server, &auth_config, &home);

    server.close();
    home.close(&execution).unwrap().reclaim();
    let _ = std::fs::remove_dir_all(&dir);
}

fn open_exclusive_directory_home(
    dir: &std::path::Path,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> worth_query_host::facade::WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>
{
    let mem_home = open_shared_home(
        WorthQueryApplicationHomeLocation::MemoryOnly,
        execution,
        true,
    )
    .expect("memory home opens");
    assert_eq!(
        mem_home
            .serve_remote("127.0.0.1:0".parse().unwrap(), make_auth_config())
            .err(),
        Some(SharedAccessDenial::DirectoryHomeRequired)
    );
    drop(mem_home);

    let home = open_shared_home(
        WorthQueryApplicationHomeLocation::Directory(dir.to_path_buf()),
        execution,
        true,
    )
    .expect("directory home opens");
    let second_open = open_shared_home(
        WorthQueryApplicationHomeLocation::Directory(dir.to_path_buf()),
        execution,
        false,
    );
    assert!(
        second_open.is_err(),
        "concurrent second Directory Home open MUST be refused by Store exclusivity"
    );
    home
}

fn assert_framing_rejections(
    server: &mut WorthQueryRemoteServer<'_, SharedProjectSchema, SharedProjectProgram>,
    auth_config: &RemoteAuthConfig,
    home: &worth_query_host::facade::WorthQueryApplicationHome<
        SharedProjectSchema,
        SharedProjectProgram,
    >,
) {
    let mut raw_client = WorthQueryRemoteClient::connect_with_limits(
        server.local_addr(),
        issue_principal_token(auth_config, home, 0, 10),
        RemoteClientLimits::default(),
    )
    .unwrap();
    let mut huge_header = [0_u8; 12];
    huge_header[0..4].copy_from_slice(b"WQS1");
    huge_header[4] = 1;
    huge_header[5] = 0x01;
    huge_header[8..12].copy_from_slice(&(u32::MAX - 1).to_be_bytes());
    raw_client.write_raw_bytes(&huge_header).unwrap();
    assert!(server.serve_once(Duration::from_secs(5)).unwrap());
    assert_eq!(
        raw_client.read_refusal_frame(),
        Ok(SharedAccessDenial::ProtocolRefused(
            ProtocolRefusalReason::FrameTooLarge {
                requested: u32::MAX - 1,
                maximum: 2048,
            }
        ))
    );

    let bad_magic = server.handle_frame(b"NOPE\x01\x01\x00\x00\x00\x00\x00\x00");
    assert_eq!(&bad_magic[0..6], b"WQS1\x01\xFF");
    let bad_ver = server.handle_frame(b"WQS1\x02\x01\x00\x00\x00\x00\x00\x00");
    assert_eq!(&bad_ver[0..6], b"WQS1\x01\xFF");
    let bad_flags = server.handle_frame(b"WQS1\x01\x01\x00\x01\x00\x00\x00\x00");
    assert_eq!(&bad_flags[0..6], b"WQS1\x01\xFF");
}

fn assert_auth_rejections(
    server: &mut WorthQueryRemoteServer<'_, SharedProjectSchema, SharedProjectProgram>,
    auth_config: &RemoteAuthConfig,
    home: &worth_query_host::facade::WorthQueryApplicationHome<
        SharedProjectSchema,
        SharedProjectProgram,
    >,
) {
    let addr = server.local_addr();
    for (token, expected_reason) in invalid_auth_cases(auth_config, home) {
        let mut client = WorthQueryRemoteClient::connect(addr, token).unwrap();
        client
            .send_query::<SharedProjectSchema, SharedProjectProgram>(
                &ProjectId::project1(),
                &ProjectQueryParams,
            )
            .unwrap();
        assert!(server.serve_once(Duration::from_secs(5)).unwrap());
        assert_eq!(
            client.recv_query::<SharedProjectSchema, SharedProjectProgram>(),
            Err(SharedAccessDenial::AuthenticationRefused(expected_reason))
        );
    }
}

fn invalid_auth_cases(
    auth_config: &RemoteAuthConfig,
    home: &worth_query_host::facade::WorthQueryApplicationHome<
        SharedProjectSchema,
        SharedProjectProgram,
    >,
) -> [(
    worth_query_host::facade::shared_access::RemoteAuthToken,
    AuthenticationRefusalReason,
); 3] {
    let alice_tok = issue_principal_token(auth_config, home, 0, 11);
    let other_cfg = RemoteAuthConfig::new(
        auth_config.audience().clone(),
        auth_config.method().clone(),
        RemoteAuthKey::new("unknown-key", [0x99_u8; 32]).unwrap(),
    );
    let unknown_key_tok = other_cfg
        .issue_token(RemoteTokenIssueRequest {
            key_id: "unknown-key".to_owned(),
            home_identity: home.home_identity().unwrap(),
            pinned_incarnation: None,
            principal: alice_identity(),
            ttl: Duration::from_secs(60),
            nonce: [1_u8; 16],
        })
        .unwrap();
    let foreign_home_tok = auth_config
        .issue_token(RemoteTokenIssueRequest {
            key_id: "k1".to_owned(),
            home_identity: HomeIdentity::from_observed_bytes([0xAA_u8; 16]).unwrap(),
            pinned_incarnation: None,
            principal: alice_identity(),
            ttl: Duration::from_secs(60),
            nonce: [2_u8; 16],
        })
        .unwrap();
    [
        (
            tamper_token(&alice_tok),
            AuthenticationRefusalReason::InvalidSignature,
        ),
        (unknown_key_tok, AuthenticationRefusalReason::UnknownKey),
        (
            foreign_home_tok,
            AuthenticationRefusalReason::HomeIdentityMismatch,
        ),
    ]
}

fn assert_watch_budget_and_owner_isolation(
    server: &mut WorthQueryRemoteServer<'_, SharedProjectSchema, SharedProjectProgram>,
    auth_config: &RemoteAuthConfig,
    home: &worth_query_host::facade::WorthQueryApplicationHome<
        SharedProjectSchema,
        SharedProjectProgram,
    >,
) {
    let addr = server.local_addr();
    let alice_tok = issue_principal_token(auth_config, home, 0, 12);
    let bob_tok = issue_principal_token(auth_config, home, 1, 13);

    let mut c_alice = WorthQueryRemoteClient::connect(addr, alice_tok.clone()).unwrap();
    c_alice
        .send_watch_open::<SharedProjectSchema, SharedProjectProgram>(
            &ProjectId::project1(),
            &ProjectQueryParams,
        )
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(5)).unwrap());
    let mut alice_watch = c_alice
        .recv_watch_open::<SharedProjectSchema, SharedProjectProgram>()
        .unwrap();

    let mut c_bob = WorthQueryRemoteClient::connect(addr, bob_tok.clone()).unwrap();
    c_bob
        .send_watch_open::<SharedProjectSchema, SharedProjectProgram>(
            &ProjectId::project1(),
            &ProjectQueryParams,
        )
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(5)).unwrap());
    assert_eq!(
        c_bob
            .recv_watch_open::<SharedProjectSchema, SharedProjectProgram>()
            .err(),
        Some(SharedAccessDenial::BudgetRefused(
            BudgetRefusalReason::WatchCapacityExhausted {
                active: 1,
                maximum: 1,
            }
        ))
    );

    alice_watch.set_auth_token(bob_tok);
    alice_watch.send_poll().unwrap();
    assert!(server.serve_once(Duration::from_secs(5)).unwrap());
    assert_eq!(
        alice_watch.recv_poll(),
        Err(SharedAccessDenial::WatchRefused(
            WatchRefusalReason::WatchNotFound
        ))
    );
    alice_watch.set_auth_token(alice_tok);
    alice_watch.send_poll().unwrap();
    assert!(server.serve_once(Duration::from_secs(5)).unwrap());
    assert_eq!(alice_watch.recv_poll(), Ok(RemoteWatchPollOutcome::Pending));
}
