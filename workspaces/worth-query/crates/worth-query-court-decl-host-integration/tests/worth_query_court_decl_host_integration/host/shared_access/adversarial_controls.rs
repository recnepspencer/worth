//! Round 2 security, transport, nonblocking deadline, and watch-reclamation boundary courts.

use std::io::Write;
use std::net::TcpStream;
use std::time::{Duration, Instant};

use worth_query_decl::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_host::facade::{
    shared_access::{
        RemoteAuthConfig, RemoteClientLimits, RemoteServerLimits, RemoteTokenIssueRequest,
        ScopeAccess, SharedAccessDenial, WorthQueryRemoteClient, WorthQueryRemoteServer,
        MIN_MAX_FRAME_BYTES,
    },
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation,
};
use worth_query_testkit_host::facade::host::checkpoint_capture;

use super::dst::{issue_principal_token, make_auth_config};
use super::mutation_handlers::SharedProjectProgram;
use super::mutations::{alice_identity, bob_identity, ProjectId};
use super::program::open_shared_home;
use super::query::ProjectQueryParams;
use super::schema::SharedProjectSchema;

#[test]
fn adversarial_round2_identity_loopback_nonblocking_bounds_and_watch_reclaim() {
    let dir = worth_query_testkit_host::facade::host::application_home::location()
        .with_extension("shared-r2-controls");
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    assert_min_frame_size_bounds();
    let home = open_shared_home(
        WorthQueryApplicationHomeLocation::Directory(dir.clone()),
        execution,
        true,
    )
    .expect("directory home opens");

    assert_wrong_issuer_and_nonloopback_boundaries(&home);
    assert_nonblocking_trickle_and_reclamation(&home);
    home.close(&execution).unwrap().reclaim();
    let _ = std::fs::remove_dir_all(&dir);
}

fn assert_min_frame_size_bounds() {
    assert!(RemoteServerLimits::default()
        .with_max_frame_bytes(0)
        .is_none());
    assert!(RemoteServerLimits::default()
        .with_max_frame_bytes(10)
        .is_none());
    assert!(RemoteServerLimits::default()
        .with_max_frame_bytes(MIN_MAX_FRAME_BYTES - 1)
        .is_none());
    assert!(RemoteServerLimits::default()
        .with_max_frame_bytes(MIN_MAX_FRAME_BYTES)
        .is_some());
    assert!(RemoteClientLimits::default()
        .with_max_frame_bytes(MIN_MAX_FRAME_BYTES - 1)
        .is_none());
}

fn assert_wrong_issuer_and_nonloopback_boundaries(
    home: &WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>,
) {
    let evil_alice =
        WorthQueryExternalPrincipalIdentity::new("https://evil.example", "alice").unwrap();
    let denied =
        home.grant_scope_access(ProjectId::project1(), evil_alice, ScopeAccess::read_only());
    assert!(
        matches!(denied, Err(SharedAccessDenial::GrantMutationRefused(_))),
        "wrong-issuer external identity MUST fail closed: {denied:?}"
    );

    let grant = home
        .grant_scope_access(
            ProjectId::project1(),
            alice_identity(),
            ScopeAccess::read_and_watch(),
        )
        .expect("valid issuer+subject grant commits");
    assert!(grant.committed_record_changes() > 0);
    home.grant_scope_access(
        ProjectId::project1(),
        bob_identity(),
        ScopeAccess::read_and_watch(),
    )
    .expect("bob grant commits");

    let cfg = make_auth_config();
    assert_eq!(
        home.serve_remote("0.0.0.0:0".parse().unwrap(), cfg.clone())
            .err(),
        Some(SharedAccessDenial::NonLoopbackAddressRefused)
    );
    assert_eq!(
        home.serve_remote("192.0.2.1:9000".parse().unwrap(), cfg.clone())
            .err(),
        Some(SharedAccessDenial::NonLoopbackAddressRefused)
    );
    let tok = issue_principal_token(&cfg, home, 0, 77);
    assert_eq!(
        WorthQueryRemoteClient::connect("192.0.2.1:9000".parse().unwrap(), tok).err(),
        Some(SharedAccessDenial::NonLoopbackAddressRefused)
    );

    let min_limits = RemoteServerLimits::default()
        .with_max_frame_bytes(MIN_MAX_FRAME_BYTES)
        .unwrap();
    let mut min_server = home
        .serve_remote("127.0.0.1:0".parse().unwrap(), cfg.with_limits(min_limits))
        .unwrap();
    let refusal = min_server.handle_frame(b"NOPE\x01\x01\x00\x00\x00\x00\x00\x00");
    assert_eq!(&refusal[0..6], b"WQS1\x01\xFF");
    min_server.close();
}

fn assert_nonblocking_trickle_and_reclamation(
    home: &WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>,
) {
    let limits = RemoteServerLimits::default()
        .with_max_active_watches(1, 1)
        .unwrap()
        .with_request_timeout(Duration::from_millis(240))
        .unwrap();
    let cfg = make_auth_config().with_limits(limits);
    let mut server = home
        .serve_remote("127.0.0.1:0".parse().unwrap(), cfg.clone())
        .unwrap();
    assert_trickle_does_not_stall(&mut server, &cfg, home);
    assert_disconnect_reclaims_watch(&mut server, &cfg, home);
    assert_token_expiry_reclaims_watch(&mut server, &cfg, home);
    server.close();
}

fn assert_trickle_does_not_stall(
    server: &mut WorthQueryRemoteServer<'_, SharedProjectSchema, SharedProjectProgram>,
    cfg: &RemoteAuthConfig,
    home: &WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>,
) {
    assert_multi_chunk_trickle_expires(server);
    assert_concurrent_trickle_allows_healthy_client(server, cfg, home);
}

fn assert_multi_chunk_trickle_expires(
    server: &mut WorthQueryRemoteServer<'_, SharedProjectSchema, SharedProjectProgram>,
) {
    let addr = server.local_addr();
    // Drive the server after EVERY intermediate chunk while the original 240ms
    // frame_deadline is still live, proving chunks 2 & 3 are actively consumed
    // without resetting the absolute deadline.
    let mut trickle = TcpStream::connect(addr).unwrap();
    trickle.write_all(b"WQS1").unwrap();
    trickle.flush().unwrap();
    let start = Instant::now();
    assert!(!server.serve_once(Duration::from_millis(15)).unwrap());
    assert!(start.elapsed() < Duration::from_millis(180));
    assert_eq!(server.active_connection_count(), 1);

    std::thread::sleep(Duration::from_millis(60));
    trickle.write_all(b"\x01\x01").unwrap();
    trickle.flush().unwrap();
    assert!(!server.serve_once(Duration::from_millis(10)).unwrap());
    assert_eq!(server.active_connection_count(), 1);

    std::thread::sleep(Duration::from_millis(60));
    trickle.write_all(b"\x00\x00").unwrap();
    trickle.flush().unwrap();
    let last_chunk_at = Instant::now();
    assert!(!server.serve_once(Duration::from_millis(10)).unwrap());
    assert_eq!(
        server.active_connection_count(),
        1,
        "chunk 3 must be read while original 240ms deadline is still live"
    );

    // Wait just past the original 240ms deadline, strictly BEFORE a renewed 240ms
    // deadline from chunk 3 could elapse.
    std::thread::sleep(Duration::from_millis(115));
    let _ = server.serve_pending().unwrap();
    assert!(
        last_chunk_at.elapsed() < Duration::from_millis(230),
        "expiry check must happen before a renewed 240ms deadline from chunk 3"
    );
    assert_eq!(
        server.active_connection_count(),
        0,
        "multi-chunk trickle connection MUST expire at original frame_deadline without reset"
    );
    drop(trickle);
}

fn assert_concurrent_trickle_allows_healthy_client(
    server: &mut WorthQueryRemoteServer<'_, SharedProjectSchema, SharedProjectProgram>,
    cfg: &RemoteAuthConfig,
    home: &WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>,
) {
    let addr = server.local_addr();
    let mut trickle = TcpStream::connect(addr).unwrap();
    trickle.write_all(b"WQS1\x01").unwrap();
    trickle.flush().unwrap();
    let _ = server.serve_pending().unwrap();

    let alice_tok = issue_principal_token(cfg, home, 0, 81);
    let mut healthy = WorthQueryRemoteClient::connect(addr, alice_tok).unwrap();
    healthy
        .send_query::<SharedProjectSchema, SharedProjectProgram>(
            &ProjectId::project1(),
            &ProjectQueryParams,
        )
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    assert_eq!(
        healthy
            .recv_query::<SharedProjectSchema, SharedProjectProgram>()
            .unwrap()[0]
            .project_key,
        "Project1"
    );
    drop(trickle);
    drop(healthy);
}

fn assert_disconnect_reclaims_watch(
    server: &mut WorthQueryRemoteServer<'_, SharedProjectSchema, SharedProjectProgram>,
    cfg: &RemoteAuthConfig,
    home: &WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>,
) {
    let addr = server.local_addr();
    let alice_tok = issue_principal_token(cfg, home, 0, 82);
    let mut c_alice = WorthQueryRemoteClient::connect(addr, alice_tok).unwrap();
    c_alice
        .send_watch_open::<SharedProjectSchema, SharedProjectProgram>(
            &ProjectId::project1(),
            &ProjectQueryParams,
        )
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    let alice_watch = c_alice
        .recv_watch_open::<SharedProjectSchema, SharedProjectProgram>()
        .unwrap();
    assert_eq!(server.active_watch_count(), 1);
    drop(alice_watch);
    std::thread::sleep(Duration::from_millis(20));
    let _ = server.serve_pending().unwrap();
    assert_eq!(
        server.active_watch_count(),
        0,
        "dropping remote watch MUST reclaim server-side watch slot"
    );
}

fn assert_token_expiry_reclaims_watch(
    server: &mut WorthQueryRemoteServer<'_, SharedProjectSchema, SharedProjectProgram>,
    cfg: &RemoteAuthConfig,
    home: &WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>,
) {
    let addr = server.local_addr();
    let short_tok = cfg
        .issue_token(RemoteTokenIssueRequest {
            key_id: "k1".to_owned(),
            home_identity: home.home_identity().unwrap(),
            pinned_incarnation: None,
            principal: alice_identity(),
            ttl: Duration::from_secs(1),
            nonce: [0x55_u8; 16],
        })
        .unwrap();
    let mut c_short = WorthQueryRemoteClient::connect(addr, short_tok).unwrap();
    c_short
        .send_watch_open::<SharedProjectSchema, SharedProjectProgram>(
            &ProjectId::project1(),
            &ProjectQueryParams,
        )
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    let _short_watch = c_short
        .recv_watch_open::<SharedProjectSchema, SharedProjectProgram>()
        .unwrap();
    assert_eq!(server.active_watch_count(), 1);
    std::thread::sleep(Duration::from_millis(1150));
    // Read-only observers must NOT mutate or reap watches.
    assert_eq!(server.active_watch_count(), 1);
    assert_eq!(server.active_watches().len(), 1);
    assert_eq!(server.reap_expired_watches(), 1);
    assert_eq!(server.active_watch_count(), 0);
    assert_bob_acquires_reclaimed_watch(server, cfg, home);
}

fn assert_bob_acquires_reclaimed_watch(
    server: &mut WorthQueryRemoteServer<'_, SharedProjectSchema, SharedProjectProgram>,
    cfg: &RemoteAuthConfig,
    home: &WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>,
) {
    let bob_tok = issue_principal_token(cfg, home, 1, 83);
    let mut c_bob = WorthQueryRemoteClient::connect(server.local_addr(), bob_tok).unwrap();
    c_bob
        .send_watch_open::<SharedProjectSchema, SharedProjectProgram>(
            &ProjectId::project1(),
            &ProjectQueryParams,
        )
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    let _bob_watch = c_bob
        .recv_watch_open::<SharedProjectSchema, SharedProjectProgram>()
        .expect("Bob acquires watch slot reclaimed after Alice expiry");
}
