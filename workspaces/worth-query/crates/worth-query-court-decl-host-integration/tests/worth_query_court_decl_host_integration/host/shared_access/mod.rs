//! Shared Access courts: Locked Target DX #10 end-to-end loopback TCP journey,
//! adversarial framing/auth/budget boundaries, 64-seed DST, and TLC trace probes.

mod adversarial;
mod adversarial_controls;
mod adversarial_envelope;
mod adversarial_mismatch;
mod adversarial_reopen_watch;
mod adversarial_watch_shift;
mod alias_shift_contract;
mod capabilities;
mod dst;
mod dst_open_session;
mod dst_oracle;
mod dst_session;
mod dst_trace;
mod mutation_handlers;
mod mutation_registrations;
mod mutations;
mod owner_observe;
mod program;
mod query;
mod relations;
mod schema;
mod tlc_probes;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use worth_query_host::facade::{
    shared_access::{
        AuthenticationRefusalReason, AuthorizationRefusalReason, RemoteAuthToken,
        RemoteTokenIssueRequest, RemoteWatchPollOutcome, ScopeAccess, SharedAccessDenial,
        WorthQueryRemoteClient, WorthQueryRemoteServer,
    },
    WorthQueryApplicationHomeLocation,
};
use worth_query_testkit_host::facade::host::checkpoint_capture;

use dst::{issue_principal_token, make_auth_config};
use mutation_handlers::SharedProjectProgram;
use mutations::{alice_identity, bob_identity, ProjectId};
use program::{open_shared_home, owner_write_project};
use query::{ProjectQueryParams, ProjectStateRow};
use schema::SharedProjectSchema;

/// Exercises Locked Target DX #10 over real multi-threaded loopback TCP:
/// grant -> query -> cross-scope refusal -> watch -> live delivery ->
/// revoke -> immediate fail-closed watch/read refusal -> close & reopen durability.
#[test]
fn dx10_shared_project_grant_query_watch_revoke_and_reopen_over_loopback() {
    let dir = worth_query_testkit_host::facade::host::application_home::location()
        .with_extension("shared-dx10");
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let auth_config = make_auth_config();
    let home = open_shared_home(
        WorthQueryApplicationHomeLocation::Directory(dir.clone()),
        execution,
        true,
    )
    .expect("fresh Directory Home opens");
    let alice_grant = home
        .grant_scope_access(
            ProjectId::project1(),
            alice_identity(),
            ScopeAccess::read_and_watch(),
        )
        .expect("owner grants Alice read+watch on Project1");
    let _bob_grant = home
        .grant_scope_access(
            ProjectId::project1(),
            bob_identity(),
            ScopeAccess::read_only(),
        )
        .expect("owner grants Bob read-only on Project1");
    owner_write_project(&home, "Project1", 10);
    owner_write_project(&home, "Project2", 99);

    let alice_token = issue_principal_token(&auth_config, &home, 0, 1);
    let bob_token = issue_principal_token(&auth_config, &home, 1, 2);
    let pinned_token = issue_pinned_bob_token(&auth_config, &home);
    let mut server = home
        .serve_remote("127.0.0.1:0".parse().unwrap(), auth_config.clone())
        .expect("single-owner remote server binds loopback");
    run_live_collaborator_phases(
        &home,
        &mut server,
        alice_grant.id(),
        (alice_token.clone(), bob_token.clone()),
    );
    server.close();
    home.close(&execution)
        .expect("Directory Home closes cleanly")
        .reclaim();

    let reopened = open_shared_home(
        WorthQueryApplicationHomeLocation::Directory(dir.clone()),
        execution,
        false,
    )
    .expect("Directory Home reopens cleanly");
    let mut server2 = reopened
        .serve_remote("127.0.0.1:0".parse().unwrap(), auth_config)
        .expect("reopened Home serves remote");
    assert_post_reopen_state(&mut server2, alice_token, bob_token, pinned_token);
    server2.close();
    reopened.close(&execution).unwrap().reclaim();
    let _ = std::fs::remove_dir_all(&dir);
}

fn issue_pinned_bob_token(
    auth_config: &worth_query_host::facade::shared_access::RemoteAuthConfig,
    home: &worth_query_host::facade::WorthQueryApplicationHome<
        SharedProjectSchema,
        SharedProjectProgram,
    >,
) -> RemoteAuthToken {
    auth_config
        .issue_token(RemoteTokenIssueRequest {
            key_id: "k1".to_owned(),
            home_identity: home.home_identity().unwrap(),
            pinned_incarnation: home.home_incarnation(),
            principal: bob_identity(),
            ttl: Duration::from_secs(3600),
            nonce: [7_u8; 16],
        })
        .unwrap()
}

#[derive(Clone, Default)]
struct PhaseBarrier {
    phase1_done: Arc<AtomicBool>,
    phase2_ready: Arc<AtomicBool>,
    phase2_done: Arc<AtomicBool>,
    phase3_ready: Arc<AtomicBool>,
    client_done: Arc<AtomicBool>,
}

struct BarrierUnwindGuard(PhaseBarrier);
impl Drop for BarrierUnwindGuard {
    fn drop(&mut self) {
        self.0.phase1_done.store(true, Ordering::SeqCst);
        self.0.phase2_done.store(true, Ordering::SeqCst);
        self.0.client_done.store(true, Ordering::SeqCst);
    }
}

fn run_live_collaborator_phases(
    home: &worth_query_host::facade::WorthQueryApplicationHome<
        SharedProjectSchema,
        SharedProjectProgram,
    >,
    server: &mut WorthQueryRemoteServer<'_, SharedProjectSchema, SharedProjectProgram>,
    alice_grant_id: &worth_query_host::facade::shared_access::ScopeGrantId,
    tokens: (RemoteAuthToken, RemoteAuthToken),
) {
    let addr = server.local_addr();
    let barrier = PhaseBarrier::default();
    let client_barrier = barrier.clone();

    std::thread::scope(|threads| {
        let collaborator = threads.spawn(move || {
            let _guard = BarrierUnwindGuard(client_barrier.clone());
            run_collaborator_assertions(addr, tokens.0, tokens.1, &client_barrier);
        });
        while !barrier.phase1_done.load(Ordering::SeqCst) {
            let _ = server.serve_pending().unwrap();
            std::thread::yield_now();
        }
        owner_write_project(home, "Project1", 11);
        barrier.phase2_ready.store(true, Ordering::SeqCst);
        while !barrier.phase2_done.load(Ordering::SeqCst) {
            let _ = server.serve_pending().unwrap();
            std::thread::yield_now();
        }
        owner_write_project(home, "Project1", 12);
        home.revoke_scope_access(alice_grant_id)
            .expect("owner revokes Alice's grant");
        barrier.phase3_ready.store(true, Ordering::SeqCst);
        while !barrier.client_done.load(Ordering::SeqCst) {
            let _ = server.serve_pending().unwrap();
            std::thread::yield_now();
        }
        collaborator.join().unwrap();
    });
}

fn run_collaborator_assertions(
    addr: std::net::SocketAddr,
    alice_token: RemoteAuthToken,
    bob_token: RemoteAuthToken,
    barrier: &PhaseBarrier,
) {
    let (mut alice, mut watch) = assert_alice_initial_access(addr, alice_token);
    assert_bob_read_only(addr, bob_token);
    barrier.phase1_done.store(true, Ordering::SeqCst);
    while !barrier.phase2_ready.load(Ordering::SeqCst) {
        std::thread::yield_now();
    }
    assert_eq!(
        watch.poll(),
        Ok(RemoteWatchPollOutcome::Delivered(vec![ProjectStateRow {
            project_key: "Project1".to_owned(),
            version: 11,
        }]))
    );
    barrier.phase2_done.store(true, Ordering::SeqCst);
    while !barrier.phase3_ready.load(Ordering::SeqCst) {
        std::thread::yield_now();
    }
    assert_eq!(
        watch.poll(),
        Err(SharedAccessDenial::AuthorizationRefused(
            AuthorizationRefusalReason::WatchCapabilityRevoked
        ))
    );
    assert_eq!(
        alice.query::<SharedProjectSchema, SharedProjectProgram>(
            &ProjectId::project1(),
            &ProjectQueryParams
        ),
        Err(SharedAccessDenial::AuthorizationRefused(
            AuthorizationRefusalReason::CapabilityDenied
        ))
    );
    barrier.client_done.store(true, Ordering::SeqCst);
}

fn assert_alice_initial_access(
    addr: std::net::SocketAddr,
    alice_token: RemoteAuthToken,
) -> (
    WorthQueryRemoteClient,
    worth_query_host::facade::shared_access::WorthQueryRemoteWatch<
        SharedProjectSchema,
        SharedProjectProgram,
    >,
) {
    let mut alice = WorthQueryRemoteClient::connect(addr, alice_token).unwrap();
    let rows = alice
        .query::<SharedProjectSchema, SharedProjectProgram>(
            &ProjectId::project1(),
            &ProjectQueryParams,
        )
        .expect("Alice reads granted Project1");
    assert_eq!(
        rows,
        vec![ProjectStateRow {
            project_key: "Project1".to_owned(),
            version: 10
        }]
    );
    assert_eq!(
        alice.query::<SharedProjectSchema, SharedProjectProgram>(
            &ProjectId::project2(),
            &ProjectQueryParams
        ),
        Err(SharedAccessDenial::AuthorizationRefused(
            AuthorizationRefusalReason::CapabilityDenied
        ))
    );
    let mut watch = alice
        .watch::<SharedProjectSchema, SharedProjectProgram>(
            &ProjectId::project1(),
            &ProjectQueryParams,
        )
        .expect("Alice opens watch on Project1");
    assert_eq!(watch.initial_rows()[0].version, 10);
    assert_eq!(watch.poll(), Ok(RemoteWatchPollOutcome::Pending));
    (alice, watch)
}

fn assert_bob_read_only(addr: std::net::SocketAddr, bob_token: RemoteAuthToken) {
    let mut bob = WorthQueryRemoteClient::connect(addr, bob_token).unwrap();
    assert_eq!(
        bob.query::<SharedProjectSchema, SharedProjectProgram>(
            &ProjectId::project1(),
            &ProjectQueryParams,
        )
        .unwrap()[0]
            .version,
        10
    );
    assert_eq!(
        bob.watch::<SharedProjectSchema, SharedProjectProgram>(
            &ProjectId::project1(),
            &ProjectQueryParams,
        )
        .err(),
        Some(SharedAccessDenial::AuthorizationRefused(
            AuthorizationRefusalReason::CapabilityDenied
        ))
    );
}

fn assert_post_reopen_state(
    server: &mut WorthQueryRemoteServer<'_, SharedProjectSchema, SharedProjectProgram>,
    alice_token: RemoteAuthToken,
    bob_token: RemoteAuthToken,
    pinned_token: RemoteAuthToken,
) {
    let addr = server.local_addr();
    let mut alice = WorthQueryRemoteClient::connect(addr, alice_token).unwrap();
    alice
        .send_query::<SharedProjectSchema, SharedProjectProgram>(
            &ProjectId::project1(),
            &ProjectQueryParams,
        )
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(5)).unwrap());
    assert_eq!(
        alice.recv_query::<SharedProjectSchema, SharedProjectProgram>(),
        Err(SharedAccessDenial::AuthorizationRefused(
            AuthorizationRefusalReason::CapabilityDenied
        ))
    );

    let mut bob = WorthQueryRemoteClient::connect(addr, bob_token).unwrap();
    bob.send_query::<SharedProjectSchema, SharedProjectProgram>(
        &ProjectId::project1(),
        &ProjectQueryParams,
    )
    .unwrap();
    assert!(server.serve_once(Duration::from_secs(5)).unwrap());
    assert_eq!(
        bob.recv_query::<SharedProjectSchema, SharedProjectProgram>()
            .unwrap()[0]
            .version,
        12
    );

    let mut stale_bob = WorthQueryRemoteClient::connect(addr, pinned_token).unwrap();
    stale_bob
        .send_query::<SharedProjectSchema, SharedProjectProgram>(
            &ProjectId::project1(),
            &ProjectQueryParams,
        )
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(5)).unwrap());
    assert_eq!(
        stale_bob.recv_query::<SharedProjectSchema, SharedProjectProgram>(),
        Err(SharedAccessDenial::AuthenticationRefused(
            AuthenticationRefusalReason::StaleIncarnation
        ))
    );
}
