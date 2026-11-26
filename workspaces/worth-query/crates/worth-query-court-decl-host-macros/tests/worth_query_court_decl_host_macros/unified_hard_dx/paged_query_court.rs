//! W8B-4: a collection larger than its query's result ceiling (`limits = 8`)
//! refuses a plain `query`, but pages to exhaustion through `query_page`: one
//! snapshot walks every row exactly once in the declared order while another
//! session commits, including across runs of equal ordering keys.
use std::num::NonZeroUsize;
use std::path::Path;

use worth_foundational::facade::CanonicalDate;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock, primary_graph::*,
    WorthQueryApplicationHome, WorthQueryHomeConfig, WorthQueryPageCursor,
    DEFAULT_HOST_EXECUTION_POLICY,
};

use super::actions::SetTaskState;
use super::courts::{build_id, temp_home_path};
use super::program::{owner, seed, DaymarkHome, DaymarkRuntime};
use super::queries::{BacklogTasks, ReadTaskDetails, TasksByPriority};
use super::schema::*;

/// Project 2 holds tasks 100..120, more than the 8-row ceiling.
const BACKLOG: std::ops::Range<u64> = 100..120;

fn width(rows: usize) -> NonZeroUsize {
    NonZeroUsize::new(rows).unwrap()
}

fn backlog() -> BacklogTasks {
    BacklogTasks { project: 2 }
}

fn open_backlog(path: &Path, clock: &WorthQueryAdmissionClock) -> DaymarkHome {
    WorthQueryApplicationHome::open_directory(
        path,
        WorthQueryHomeConfig::new(build_id(), (DaymarkRuntime::new(),))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| {
                seed(graph, installed)?;
                seed_backlog(graph)
            }),
    )
    .expect("open_directory succeeds")
}

fn seed_backlog(
    graph: &mut WorthQueryPrimaryGraphBootstrap<DaymarkSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let due = CanonicalDate::parse_iso("2026-12-01").unwrap();
    for task in BACKLOG {
        let task_key = || WorthQueryApplicationEntityKey::new(format!("task-{task}")).unwrap();
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Task::reference(), task_key())
                .field(TaskKey::at("TaskKey"), task)
                .field(Title::at("Title"), format!("Backlog {task}"))
                .field(State::at("State"), "OPEN".to_owned())
                .field(Priority::at("Priority"), task % 3)
                .field(DueDate::at("DueDate"), due),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            Contains::reference(),
            format!("contains-{task}"),
            WorthQueryApplicationEntityKey::new("project-2".to_owned()).unwrap(),
            task_key(),
        ))?;
    }
    Ok(())
}

/// Every page of `read` from the start, following each next cursor.
fn walk<Intent>(
    mut read: impl FnMut(
        Option<&WorthQueryPageCursor<Intent>>,
    ) -> (Vec<TaskRow>, Option<WorthQueryPageCursor<Intent>>),
) -> Vec<Vec<TaskRow>> {
    let mut pages = Vec::new();
    let mut cursor = None;
    loop {
        let (rows, next) = read(cursor.as_ref());
        pages.push(rows);
        match next {
            Some(next) => cursor = Some(next),
            None => return pages,
        }
    }
}

#[test]
fn a_snapshot_pages_past_the_result_ceiling_to_exhaustion_on_one_basis() {
    let path = temp_home_path("paged-query");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_backlog(&path, &clock);
    let me = owner(&home, &clock);
    let session = home.session(&me);

    // The unchanged one-shot query still refuses a collection past its ceiling.
    let refused = session.query(backlog()).unwrap_err();
    assert!(
        format!("{refused:?}").contains("ResultLimitExceeded"),
        "{refused:?}"
    );

    let snapshot = session.snapshot().unwrap();
    let mut committed = false;
    let pages = walk(|after| {
        let page = snapshot.query_page(backlog(), width(3), after).unwrap();
        if !committed {
            // Another session commits after the first page: the snapshot's
            // later pages still read the pinned basis.
            home.session(owner(&home, &clock))
                .run(SetTaskState {
                    task: 110,
                    state: "DONE".to_owned(),
                })
                .unwrap();
            committed = true;
        }
        let has_more = page.has_more();
        let (rows, next) = page.into_parts();
        assert_eq!(has_more, next.is_some());
        (rows, next)
    });
    assert_eq!(
        pages.iter().map(Vec::len).collect::<Vec<_>>(),
        [3, 3, 3, 3, 3, 3, 2]
    );
    let rows = pages.concat();
    assert_eq!(
        rows.iter().map(|row| row.task_key).collect::<Vec<_>>(),
        BACKLOG.collect::<Vec<_>>()
    );
    assert!(rows.iter().all(|row| row.state == "OPEN"), "pinned basis");

    // Through the session each page reads the current basis.
    let first = session.query_page(backlog(), width(8), None).unwrap();
    let second = session
        .query_page(backlog(), width(8), first.next_cursor())
        .unwrap();
    let last = session
        .query_page(backlog(), width(8), second.next_cursor())
        .unwrap();
    assert!(first.has_more() && second.has_more() && !last.has_more());
    assert_eq!(last.rows().len(), 4);
    let task_110 = second
        .rows()
        .iter()
        .find(|row| row.task_key == 110)
        .unwrap();
    assert_eq!(task_110.state, "DONE");

    drop(snapshot);
    drop(session);
    drop(me);
    if let Err((_, stop)) = home
        .into_retirement()
        .try_retire(DEFAULT_HOST_EXECUTION_POLICY)
    {
        panic!("no page or snapshot pins the Home: {stop:?}");
    }
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn pages_cross_equal_ordering_keys_exactly_once_and_refuse_what_cannot_page() {
    let path = temp_home_path("paged-query-ties");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_backlog(&path, &clock);
    let session = home.session(owner(&home, &clock)).into_owned();

    // Priority is `task % 3`: about 7 rows share each value, so 3-row pages
    // cut inside runs of equal keys and resume by row identity.
    let pages = walk(|after| {
        session
            .query_page(TasksByPriority { project: 2 }, width(3), after)
            .unwrap()
            .into_parts()
    });
    let rows = pages.concat();
    assert_eq!(rows.len(), BACKLOG.count());
    let mut keys = rows.iter().map(|row| row.task_key).collect::<Vec<_>>();
    assert!(rows
        .windows(2)
        .all(|pair| pair[0].priority >= pair[1].priority));
    keys.sort_unstable();
    assert_eq!(keys, BACKLOG.collect::<Vec<_>>(), "each row exactly once");

    // A page cannot be wider than the ceiling; an exactly-one query cannot page.
    let too_wide = session.query_page(backlog(), width(9), None).unwrap_err();
    assert!(
        format!("{too_wide:?}").contains("ResultLimitExceeded"),
        "{too_wide:?}"
    );
    let exact = session
        .query_page(ReadTaskDetails { task: 1 }, width(1), None)
        .unwrap_err();
    assert!(
        format!("{exact:?}").contains("CardinalityMismatch"),
        "{exact:?}"
    );

    drop(session);
    if let Err((_, stop)) = home
        .into_retirement()
        .try_retire(DEFAULT_HOST_EXECUTION_POLICY)
    {
        panic!("the Home retires: {stop:?}");
    }
    std::fs::remove_dir_all(path).unwrap();
}
