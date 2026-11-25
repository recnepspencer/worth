//! A recurring rule's `on_fire` hook creates one linked Task per fire inside
//! the fire commit: caught up after reopen, idempotent on re-catch-up, and
//! silent once the rule is cancelled.
mod program;
mod schema;

use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    domain::WorthQueryClockCoordinate,
    temporal_contribution::{WorthQueryRecurringLifecycle, WorthQueryTemporalCatchUpPolicy},
};

use program::{open, owner, FireHome};
use schema::{ReadRuleTasks, ReadRules, Recurrence, ScheduleRule, TaskView, HUB};

fn tasks(home: &FireHome, clock: &WorthQueryAdmissionClock) -> Vec<TaskView> {
    home.session(owner(home, clock))
        .query(ReadRuleTasks {
            rule: "daily".to_owned(),
        })
        .expect("rule tasks read")
}

fn fired_count(home: &FireHome, clock: &WorthQueryAdmissionClock) -> u64 {
    let rules = home
        .session(owner(home, clock))
        .query(ReadRules {
            hub: HUB.to_owned(),
        })
        .expect("rules read");
    rules[0].fired_count
}

fn catch_up(home: &FireHome, clock: &WorthQueryAdmissionClock, now_ns: u64) -> usize {
    home.session(owner(home, clock))
        .catch_up_temporal::<Recurrence>(
            WorthQueryClockCoordinate::from_nanoseconds(now_ns),
            WorthQueryTemporalCatchUpPolicy::bounded(16).unwrap(),
        )
        .unwrap_or_else(|denial| panic!("bounded catch-up: {denial}"))
        .fired()
}

fn schedule(
    home: &FireHome,
    clock: &WorthQueryAdmissionClock,
    lifecycle: WorthQueryRecurringLifecycle,
) {
    home.session(owner(home, clock))
        .run(ScheduleRule {
            rule_id: "daily".to_owned(),
            due_ns: 1_000,
            interval_ns: 1_000,
            max_fires: 10,
            lifecycle,
        })
        .unwrap_or_else(|denial| panic!("schedule: {denial:?}"));
}

#[test]
fn on_fire_creates_one_task_per_fire_idempotently_until_cancelled() {
    let mut path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("recurring-on-fire");
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    let clock = WorthQueryAdmissionClock::system();

    let home = open(&path, &clock);
    schedule(&home, &clock, WorthQueryRecurringLifecycle::Active);
    assert!(tasks(&home, &clock).is_empty(), "scheduling fires nothing");
    home.close_clean().unwrap();

    // Two occurrences (1000, 2000) were missed while closed.
    let home = open(&path, &clock);
    assert_eq!(catch_up(&home, &clock, 2_500), 2);
    let created = tasks(&home, &clock);
    let keys: Vec<_> = created
        .iter()
        .map(|task| (task.task_key.as_str(), task.fire_number, task.scheduled_ns))
        .collect();
    assert_eq!(keys, [("daily#1", 1, 1_000), ("daily#2", 2, 2_000)]);
    assert_eq!(fired_count(&home, &clock), 2);
    home.close_clean().unwrap();

    // Re-catch-up after reopen creates nothing more.
    let home = open(&path, &clock);
    assert_eq!(catch_up(&home, &clock, 2_500), 0);
    assert_eq!(tasks(&home, &clock), created);

    // A cancelled rule fires no further occurrences, so creates no Tasks.
    schedule(&home, &clock, WorthQueryRecurringLifecycle::Cancelled);
    assert_eq!(catch_up(&home, &clock, 10_000), 0);
    home.close_clean().unwrap();
    let home = open(&path, &clock);
    assert_eq!(catch_up(&home, &clock, 20_000), 0);
    assert_eq!(tasks(&home, &clock), created);
    assert_eq!(fired_count(&home, &clock), 2);
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}
