//! Incarnation-bound `RemoteWatchId` non-aliasing court across Directory Home reopen.

use std::time::Duration;

use worth_query_host::facade::{
    shared_access::{
        RemoteAuthConfig, RemoteAuthToken, RemoteWatchId, RemoteWatchPollOutcome, ScopeAccess,
        SharedAccessDenial, WatchRefusalReason, WorthQueryRemoteClient,
    },
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation,
};
use worth_query_testkit_host::facade::host::checkpoint_capture;

use super::dst::make_auth_config;
use super::mutation_handlers::SharedProjectProgram;
use super::mutations::{alice_identity, ProjectId};
use super::program::{issue_alice_tok, open_shared_home};
use super::query::ProjectQueryParams;
use super::schema::SharedProjectSchema;

/// Verifies incarnation-bound `RemoteWatchId`: after reopen, opening replacement watch #1
/// with an unpinned token never allows stale pre-reopen watch #1 to alias it.
#[test]
fn adversarial_reopen_watch_id_never_aliases_replacement_watch() {
    let dir = worth_query_testkit_host::facade::host::application_home::location()
        .with_extension("shared-r5-watch-incarnation");
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let cfg = make_auth_config();

    let home1 = open_shared_home(
        WorthQueryApplicationHomeLocation::Directory(dir.clone()),
        execution,
        true,
    )
    .unwrap();
    home1
        .grant_scope_access(
            ProjectId::project1(),
            alice_identity(),
            ScopeAccess::read_and_watch(),
        )
        .unwrap();
    let unpinned_tok = issue_alice_tok(&cfg, home1.home_identity().unwrap(), 0x7A);
    let mut s1 = home1
        .serve_remote("127.0.0.1:0".parse().unwrap(), cfg.clone())
        .unwrap();
    let mut c1 = WorthQueryRemoteClient::connect(s1.local_addr(), unpinned_tok.clone()).unwrap();
    c1.send_watch_open::<SharedProjectSchema, SharedProjectProgram>(
        &ProjectId::project1(),
        &ProjectQueryParams,
    )
    .unwrap();
    assert!(s1.serve_once(Duration::from_secs(2)).unwrap());
    let w1 = c1
        .recv_watch_open::<SharedProjectSchema, SharedProjectProgram>()
        .unwrap();
    let stale_id = w1.id();
    drop(w1);
    drop(c1);
    s1.close();
    home1.close(&execution).unwrap().reclaim();

    let home2 = open_shared_home(
        WorthQueryApplicationHomeLocation::Directory(dir.clone()),
        execution,
        false,
    )
    .unwrap();
    assert_replacement_watch_does_not_alias_stale(&home2, &cfg, &unpinned_tok, stale_id);
    home2.close(&execution).unwrap().reclaim();
    let _ = std::fs::remove_dir_all(&dir);
}

fn assert_replacement_watch_does_not_alias_stale(
    home2: &WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>,
    cfg: &RemoteAuthConfig,
    unpinned_tok: &RemoteAuthToken,
    stale_id: RemoteWatchId,
) {
    let mut s2 = home2
        .serve_remote("127.0.0.1:0".parse().unwrap(), cfg.clone())
        .unwrap();
    let mut c2 = WorthQueryRemoteClient::connect(s2.local_addr(), unpinned_tok.clone()).unwrap();
    c2.send_watch_open::<SharedProjectSchema, SharedProjectProgram>(
        &ProjectId::project1(),
        &ProjectQueryParams,
    )
    .unwrap();
    assert!(s2.serve_once(Duration::from_secs(2)).unwrap());
    let mut w2 = c2
        .recv_watch_open::<SharedProjectSchema, SharedProjectProgram>()
        .unwrap();
    assert_eq!(stale_id.sequence(), 1);
    assert_eq!(w2.id().sequence(), 1);
    assert_ne!(stale_id.incarnation(), w2.id().incarnation());
    assert_ne!(stale_id, w2.id());

    c2.send_watch_poll_id(stale_id).unwrap();
    assert!(s2.serve_once(Duration::from_secs(2)).unwrap());
    assert_eq!(
        c2.recv_watch_poll_id::<SharedProjectSchema, SharedProjectProgram>(),
        Err(SharedAccessDenial::WatchRefused(
            WatchRefusalReason::WatchNotFound
        ))
    );
    w2.send_poll().unwrap();
    assert!(s2.serve_once(Duration::from_secs(2)).unwrap());
    assert_eq!(w2.recv_poll(), Ok(RemoteWatchPollOutcome::Pending));
    assert_eq!(s2.active_watch_count(), 1);
    s2.close();
}
