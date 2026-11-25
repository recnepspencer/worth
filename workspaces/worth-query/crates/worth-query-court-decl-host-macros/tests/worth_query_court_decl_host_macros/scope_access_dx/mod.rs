//! Stock scope access end to end: typed grants gate a remote governed read and
//! live watch, revocation fails closed, the read-only level refuses watches,
//! and a restarted owner issues fresh grants without idempotency-key reuse.
mod program;
mod schema;
mod shared;

use std::time::Duration;

use worth_query_decl::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticationAudience, WorthQueryAuthenticationMethod,
    },
    shared_access::{
        RemoteAuthConfig, RemoteAuthKey, RemoteAuthToken, RemoteTokenIssueRequest,
        RemoteWatchPollOutcome, ScopeAccess, SharedAccessDenial, WorthQueryRemoteClient,
        WorthQueryRemoteServer,
    },
};

use program::{identity, open, owner, BoardAudience, BoardHome, BoardProgram, LocalProcess};
use program::{ALICE, PROJECT};
use schema::BoardSchema;
use shared::{RenameProject, SharedParams, SharedProject};

fn auth() -> RemoteAuthConfig {
    RemoteAuthConfig::new(
        WorthQueryAuthenticationAudience::of(BoardAudience),
        WorthQueryAuthenticationMethod::of(LocalProcess),
        RemoteAuthKey::new("k1", [0x5c; 32]).unwrap(),
    )
}

fn token(
    home: &BoardHome,
    auth: &RemoteAuthConfig,
    who: WorthQueryExternalPrincipalIdentity,
) -> RemoteAuthToken {
    auth.issue_token(RemoteTokenIssueRequest {
        key_id: "k1".to_owned(),
        home_identity: home.home_identity().unwrap(),
        pinned_incarnation: None,
        principal: who,
        ttl: Duration::from_secs(600),
        nonce: [7; 16],
    })
    .unwrap()
}

type Server<'home> = WorthQueryRemoteServer<'home, BoardSchema, BoardProgram>;

fn read(
    server: &mut Server<'_>,
    client: &mut WorthQueryRemoteClient,
) -> Result<Vec<SharedProject>, SharedAccessDenial> {
    client.send_query::<BoardSchema, BoardProgram>(&PROJECT.to_owned(), &SharedParams)?;
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    client.recv_query::<BoardSchema, BoardProgram>()
}

fn project(name: &str) -> Vec<SharedProject> {
    vec![SharedProject {
        id: PROJECT.to_owned(),
        name: name.to_owned(),
    }]
}

fn rename(home: &BoardHome, name: &str) {
    home.session(owner(home))
        .run(RenameProject {
            project: PROJECT.to_owned(),
            name: name.to_owned(),
        })
        .unwrap_or_else(|denial| panic!("rename: {denial:?}"));
}

#[test]
fn stock_scope_grants_gate_remote_read_watch_and_revoke_across_restart() {
    let mut path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("scope-access-dx");
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    let auth = auth();

    let home = open(&path);
    let watch_grant = home
        .grant_scope_access(
            PROJECT.to_owned(),
            identity(ALICE),
            ScopeAccess::ReadAndWatch,
        )
        .expect("owner grants read and watch");
    let mut server = home
        .serve_remote("127.0.0.1:0".parse().unwrap(), auth.clone())
        .unwrap();
    let mut client =
        WorthQueryRemoteClient::connect(server.local_addr(), token(&home, &auth, identity(ALICE)))
            .unwrap();
    assert_eq!(read(&mut server, &mut client), Ok(project("Launch")));

    client
        .send_watch_open::<BoardSchema, BoardProgram>(&PROJECT.to_owned(), &SharedParams)
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    let mut watch = client
        .recv_watch_open::<BoardSchema, BoardProgram>()
        .unwrap();
    assert_eq!(watch.initial_rows(), project("Launch").as_slice());
    rename(&home, "Liftoff");
    watch.send_poll().unwrap();
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    assert_eq!(
        watch.recv_poll(),
        Ok(RemoteWatchPollOutcome::Delivered(project("Liftoff")))
    );
    drop(watch);

    home.revoke_scope_access(watch_grant.id())
        .expect("owner revokes");
    assert!(matches!(
        read(&mut server, &mut client),
        Err(SharedAccessDenial::AuthorizationRefused(_))
    ));
    server.close();
    home.close_clean().unwrap();

    // A restarted owner reissues grants: keys derive from grant identity, not
    // the reset process-local sequence, so no durable key is reused.
    let home = open(&path);
    home.grant_scope_access(PROJECT.to_owned(), identity(ALICE), ScopeAccess::ReadOnly)
        .expect("restarted owner grants read only");
    let mut server = home
        .serve_remote("127.0.0.1:0".parse().unwrap(), auth.clone())
        .unwrap();
    let mut client =
        WorthQueryRemoteClient::connect(server.local_addr(), token(&home, &auth, identity(ALICE)))
            .unwrap();
    assert_eq!(read(&mut server, &mut client), Ok(project("Liftoff")));
    client
        .send_watch_open::<BoardSchema, BoardProgram>(&PROJECT.to_owned(), &SharedParams)
        .unwrap();
    assert!(server.serve_once(Duration::from_secs(2)).unwrap());
    assert!(matches!(
        client
            .recv_watch_open::<BoardSchema, BoardProgram>()
            .map(|_| ()),
        Err(SharedAccessDenial::AuthorizationRefused(_))
    ));
    server.close();
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}
