//! The default desktop policy covers a cold Directory Home reopen under a new
//! build: managed-index rebuild, queries and actions need no custom policy.
//! Reads go through one generic `WorthQuerySessionQuery` bound and the Home
//! retires through `WorthQueryHomeRetirement`.
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_installation::ApplicationBuildIdentity, WorthQueryApplicationHome,
    WorthQueryHomeConfig, WorthQueryHomeOpenDisposition, WorthQuerySessionIndexedQuery,
    WorthQuerySessionQuery, DEFAULT_HOST_EXECUTION_POLICY,
};

use super::actions::SetTaskState;
use super::courts::{build_id, temp_home_path};
use super::program::{owner, seed, DaymarkHome, DaymarkRuntime};
use super::queries::FindTasksByTitle;
use super::schema::DaymarkSchema;
use super::rename_task::RenameTask;

fn next_build_id() -> ApplicationBuildIdentity {
    ApplicationBuildIdentity::parse("5a5a5a5a-5a5a-4a5a-8a5a-5a5a5a5a5a5b").unwrap()
}

fn open_with_default_policy(
    path: &std::path::Path,
    build: ApplicationBuildIdentity,
    clock: &WorthQueryAdmissionClock,
) -> DaymarkHome {
    WorthQueryApplicationHome::open_directory(
        path,
        WorthQueryHomeConfig::new(build, (DaymarkRuntime::new(),))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| seed(graph, installed)),
    )
    .unwrap_or_else(|denial| panic!("default-policy open: {denial}"))
}

/// One bound names everything a generic indexed read needs.
fn read_indexed<Q>(home: &DaymarkHome, clock: &WorthQueryAdmissionClock, intent: Q) -> Vec<Q::Row>
where
    Q: WorthQuerySessionQuery<DaymarkSchema> + WorthQuerySessionIndexedQuery<DaymarkSchema> + Clone,
{
    let me = owner(home, clock);
    home.session(&me)
        .query_indexed(intent)
        .unwrap_or_else(|denial| panic!("default-policy indexed read: {denial:?}"))
}

fn titles(home: &DaymarkHome, clock: &WorthQueryAdmissionClock, prefix: &str) -> Vec<u64> {
    let intent = FindTasksByTitle {
        project: 1,
        prefix: prefix.to_owned(),
    };
    read_indexed(home, clock, intent)
        .iter()
        .map(|row| row.task_key)
        .collect()
}

fn retire(home: DaymarkHome) {
    if let Err((_, stop)) = home
        .into_retirement()
        .try_retire(DEFAULT_HOST_EXECUTION_POLICY)
    {
        panic!("the Home retires: {stop:?}");
    }
}

#[test]
fn default_policy_reopens_under_next_build_rebuilds_index_and_runs_actions() {
    let path = temp_home_path("cold-reopen-default-policy");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_with_default_policy(&path, build_id(), &clock);
    {
        let me = owner(&home, &clock);
        let session = home.session(&me);
        for round in 0..32_u64 {
            let title = format!("Draft round {round}");
            session
                .run(RenameTask { task: 1, title })
                .unwrap_or_else(|denial| panic!("history-building rename: {denial:?}"));
        }
    }
    assert_eq!(titles(&home, &clock, "Draft"), vec![1, 2]);
    retire(home);

    let home = open_with_default_policy(&path, next_build_id(), &clock);
    assert_eq!(
        home.open_disposition(),
        WorthQueryHomeOpenDisposition::Opened
    );
    assert_eq!(titles(&home, &clock, "Draft round 31"), vec![1]);
    {
        let me = owner(&home, &clock);
        home.session(&me)
            .run(SetTaskState {
                task: 1,
                state: "DONE".to_owned(),
            })
            .unwrap_or_else(|denial| panic!("default-policy action after reopen: {denial:?}"));
    }
    retire(home);
    std::fs::remove_dir_all(path).unwrap();
}
