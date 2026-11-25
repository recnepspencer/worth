//! Stock `interval_1d` and `text_prefix` managed indexes over Optional fields
//! are sparse: rows without a key never match, `None <-> Some` edits move rows
//! in and out of probe and watch results, and a cold rebuild agrees.
mod probe;
mod program;
mod schema;

use program::{open, owner, SparseHome};
use schema::*;
use worth_query_decl::facade::application_schema::CanonicalTimestamp;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;

fn at(seconds: i64) -> CanonicalTimestamp {
    CanonicalTimestamp {
        micros_since_unix_epoch: seconds * 1_000_000,
    }
}

fn window(start: i64, end: i64) -> FocusWindow {
    FocusWindow {
        start_micros: at(start).micros_since_unix_epoch,
        end_micros: at(end).micros_since_unix_epoch,
    }
}

fn ids(rows: &[TaskView]) -> Vec<u64> {
    rows.iter().map(|row| row.id).collect()
}

fn focus(home: &SparseHome, clock: &WorthQueryAdmissionClock, start: i64, end: i64) -> Vec<u64> {
    let probe = ReadFocus {
        inbox: INBOX.to_owned(),
        window: window(start, end),
    };
    ids(&home
        .session(owner(home, clock))
        .query(probe)
        .expect("focus probe"))
}

fn titled(home: &SparseHome, clock: &WorthQueryAdmissionClock, prefix: &str) -> Vec<u64> {
    let probe = FindTitle {
        inbox: INBOX.to_owned(),
        prefix: prefix.to_owned(),
    };
    ids(&home
        .session(owner(home, clock))
        .query(probe)
        .expect("title probe"))
}

fn add(id: u64, title: Option<&str>, focus: (Option<i64>, Option<i64>)) -> AddTask {
    AddTask {
        inbox: INBOX.to_owned(),
        id,
        title: title.map(str::to_owned),
        start: focus.0.map(at),
        end: focus.1.map(at),
    }
}

fn edit(id: u64, title: Option<&str>, focus: (Option<i64>, Option<i64>)) -> EditTask {
    EditTask {
        id,
        title: title.map(str::to_owned),
        start: focus.0.map(at),
        end: focus.1.map(at),
    }
}

fn ensure_ready(home: &SparseHome, clock: &WorthQueryAdmissionClock) {
    let session = home.session(owner(home, clock));
    session
        .ensure_index_ready(FindTitle {
            inbox: INBOX.to_owned(),
            prefix: String::new(),
        })
        .expect("title index ready");
    session
        .ensure_index_ready(ReadFocus {
            inbox: INBOX.to_owned(),
            window: window(0, 1),
        })
        .expect("focus index ready");
}

#[test]
fn optional_keys_are_sparse_across_edits_watches_and_a_cold_rebuild() {
    let mut path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("sparse-index-dx");
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    ensure_ready(&home, &clock);
    let session = home.session(owner(&home, &clock));
    for task in [
        add(1, Some("Alpha"), (Some(10), Some(20))),
        add(2, None, (None, None)),
        add(3, Some("Alpine"), (Some(15), None)),
        add(4, Some("Beta"), (Some(30), Some(40))),
    ] {
        session
            .run(task)
            .unwrap_or_else(|denial| panic!("add: {denial:?}"));
    }
    // Every row is in Source; only keyed rows are reachable through a probe.
    let all = session
        .query(ReadInbox {
            inbox: INBOX.to_owned(),
        })
        .unwrap();
    assert_eq!(ids(&all), vec![1, 2, 3, 4]);
    assert_eq!(all[1].title, None);
    assert_eq!(focus(&home, &clock, 0, 100), vec![1, 4]);
    assert_eq!(
        focus(&home, &clock, 12, 18),
        vec![1],
        "partial 3 is omitted"
    );
    assert_eq!(titled(&home, &clock, ""), vec![1, 3, 4]);
    assert_eq!(titled(&home, &clock, "al"), vec![1, 3]);

    let mut focus_watch = session
        .watch(ReadFocus {
            inbox: INBOX.to_owned(),
            window: window(0, 100),
        })
        .unwrap();
    let mut title_watch = session
        .watch(FindTitle {
            inbox: INBOX.to_owned(),
            prefix: "Al".to_owned(),
        })
        .unwrap();

    // None -> Some inserts, Some -> None removes, partial -> complete inserts.
    session
        .run(edit(2, Some("Alto"), (Some(50), Some(60))))
        .unwrap();
    let woke = focus_watch
        .refresh(&session)
        .unwrap()
        .expect("None -> Some wakes");
    assert_eq!(ids(&woke), vec![1, 2, 4]);
    let woke = title_watch
        .refresh(&session)
        .unwrap()
        .expect("None -> Some wakes");
    assert_eq!(ids(&woke), vec![1, 2, 3]);

    session.run(edit(1, None, (None, None))).unwrap();
    let woke = focus_watch
        .refresh(&session)
        .unwrap()
        .expect("Some -> None wakes");
    assert_eq!(ids(&woke), vec![2, 4]);
    let woke = title_watch
        .refresh(&session)
        .unwrap()
        .expect("Some -> None wakes");
    assert_eq!(ids(&woke), vec![2, 3]);

    session
        .run(edit(3, Some("Alpine"), (Some(15), Some(25))))
        .unwrap();
    let woke = focus_watch
        .refresh(&session)
        .unwrap()
        .expect("partial -> Some wakes");
    assert_eq!(ids(&woke), vec![2, 3, 4]);
    assert_eq!(focus(&home, &clock, 12, 18), vec![3]);
    title_watch.refresh(&session).unwrap();
    // None -> None leaves both indexes and their watches untouched.
    session.run(edit(1, None, (None, None))).unwrap();
    assert!(
        focus_watch.refresh(&session).unwrap().is_none(),
        "None -> None is silent"
    );
    assert!(
        title_watch.refresh(&session).unwrap().is_none(),
        "None -> None is silent"
    );
    focus_watch.close().unwrap();
    title_watch.close().unwrap();
    let warm = (focus(&home, &clock, 0, 100), titled(&home, &clock, "al"));
    assert_eq!(warm, (vec![2, 3, 4], vec![2, 3]));
    assert_eq!(titled(&home, &clock, ""), vec![2, 3, 4]);
    drop(session);
    home.close_clean().unwrap();

    // Cold reopen: the rebuild skips unkeyed rows and matches warm state exactly.
    let home = open(&path, &clock);
    ensure_ready(&home, &clock);
    assert_eq!(
        (focus(&home, &clock, 0, 100), titled(&home, &clock, "al")),
        warm
    );
    assert_eq!(focus(&home, &clock, 12, 18), vec![3]);
    assert_eq!(titled(&home, &clock, ""), vec![2, 3, 4]);
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}
