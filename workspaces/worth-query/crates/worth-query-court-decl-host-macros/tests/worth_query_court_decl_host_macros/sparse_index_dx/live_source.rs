//! One `live = whole_result` Source query backs both stock managed indexes
//! and an owned live subscription: edits wake the subscription while index
//! probes stay exact, and both agree again after a cold reopen.
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;

use super::program::{open, owner};
use super::schema::{ReadInbox, INBOX};
use super::{add, edit, ensure_ready, focus, ids, titled};

fn inbox() -> ReadInbox {
    ReadInbox {
        inbox: INBOX.to_owned(),
    }
}

#[test]
fn a_live_source_query_backs_managed_indexes_and_an_owned_subscription() {
    let mut path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("sparse-live-source");
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    ensure_ready(&home, &clock);
    let me = owner(&home, &clock);
    let mut live = home.session(&me).subscribe_owned(inbox()).unwrap();
    assert!(live.current().unwrap().is_empty());

    let session = home.session(&me);
    session
        .run(add(1, Some("Alpha"), (Some(10), Some(20))))
        .unwrap();
    let woke = live
        .refresh(&session)
        .unwrap()
        .expect("an add wakes the Source");
    assert_eq!(ids(woke), vec![1]);
    session.run(add(2, None, (None, None))).unwrap();
    let woke = live
        .refresh(&session)
        .unwrap()
        .expect("an add wakes the Source");
    assert_eq!(ids(woke), vec![1, 2]);
    assert_eq!(focus(&home, &clock, 0, 100), vec![1]);
    assert_eq!(titled(&home, &clock, "al"), vec![1]);

    // Keying Task 2 changes a Source row and moves it into both indexes.
    session
        .run(edit(2, Some("Alto"), (Some(50), Some(60))))
        .unwrap();
    let woke = live.refresh(&session).unwrap().expect("an edit wakes it");
    assert_eq!(woke[1].title.as_deref(), Some("Alto"));
    assert_eq!(focus(&home, &clock, 0, 100), vec![1, 2]);
    assert_eq!(titled(&home, &clock, "al"), vec![1, 2]);
    assert!(live.refresh(&session).unwrap().is_none());
    drop(session);
    live.close().unwrap();
    drop(me);
    home.close_clean().unwrap();

    let home = open(&path, &clock);
    ensure_ready(&home, &clock);
    let reopened = home
        .session(owner(&home, &clock))
        .subscribe_owned(inbox())
        .unwrap();
    assert_eq!(ids(reopened.current().unwrap()), vec![1, 2]);
    assert_eq!(focus(&home, &clock, 0, 100), vec![1, 2]);
    assert_eq!(titled(&home, &clock, "al"), vec![1, 2]);
    reopened.close().unwrap();
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}
