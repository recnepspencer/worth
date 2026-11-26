//! An `on_fire ... in Inbox by InboxFacts::inbox_id()` hook creates a Task
//! with a `unique` task id inside each fire commit and links it from the
//! target Inbox: listed after cold reopen, idempotent on re-catch-up, and a
//! fire whose task id another Task holds fails closed with a typed cause and
//! still lets the Home retire cleanly. A rule scheduled in a session fires
//! on that same session's catch-up without a reopen.
mod program;
mod schema;

use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    domain::WorthQueryClockCoordinate,
    temporal_contribution::{
        WorthQueryRecurringClock, WorthQueryRecurringLifecycle, WorthQueryTemporalCatchUpPolicy,
        WorthQueryTemporalCatchUpReport, WorthQueryTemporalCatchUpStatus,
        WorthQueryTemporalOccurrenceRefusalKind,
    },
    WorthQueryRefusalCategory, WorthQueryUserFacingRefusal, DEFAULT_HOST_EXECUTION_POLICY,
};

use program::{open, owner, TargetHome, TAKEN_TASK_ID};
use schema::{ReadInboxTasks, ReadRules, Recurrence, RuleView, ScheduleRule, HUB, INBOX};

fn inbox_tasks(home: &TargetHome, clock: &WorthQueryAdmissionClock) -> Vec<(String, u64)> {
    let tasks = home
        .session(owner(home, clock))
        .query(ReadInboxTasks {
            inbox: INBOX.to_owned(),
        })
        .expect("inbox tasks read");
    tasks
        .into_iter()
        .map(|task| (task.task_id, task.fire_number))
        .collect()
}

fn rule(home: &TargetHome, clock: &WorthQueryAdmissionClock, rule_id: &str) -> RuleView {
    let rules = home
        .session(owner(home, clock))
        .query(ReadRules {
            hub: HUB.to_owned(),
        })
        .expect("rules read");
    rules
        .into_iter()
        .find(|rule| rule.rule_id == rule_id)
        .expect("scheduled rule")
}

fn catch_up(
    home: &TargetHome,
    clock: &WorthQueryAdmissionClock,
    now_ns: u64,
) -> WorthQueryTemporalCatchUpReport<WorthQueryRecurringClock<Recurrence>> {
    home.session(owner(home, clock))
        .catch_up_temporal::<Recurrence>(
            WorthQueryClockCoordinate::from_nanoseconds(now_ns),
            WorthQueryTemporalCatchUpPolicy::bounded(16).unwrap(),
        )
        .unwrap_or_else(|denial| panic!("bounded catch-up: {denial}"))
}

fn schedule(home: &TargetHome, clock: &WorthQueryAdmissionClock, rule_id: &str, due_ns: u64) {
    home.session(owner(home, clock))
        .run(ScheduleRule {
            rule_id: rule_id.to_owned(),
            due_ns,
            interval_ns: 1_000,
            max_fires: 10,
            lifecycle: WorthQueryRecurringLifecycle::Active,
        })
        .unwrap_or_else(|denial| panic!("schedule {rule_id}: {denial:?}"));
}

#[test]
fn targeted_on_fire_files_unique_tasks_into_its_inbox_and_refuses_a_taken_id() {
    let mut path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("recurring-target");
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    let clock = WorthQueryAdmissionClock::system();

    let home = open(&path, &clock);
    schedule(&home, &clock, "home/daily", 1_000);
    home.close_clean().unwrap();

    // Two occurrences (1000, 2000) were missed while closed.
    let home = open(&path, &clock);
    let report = catch_up(&home, &clock, 2_500);
    assert_eq!(
        report.status(),
        &WorthQueryTemporalCatchUpStatus::CaughtUp,
        "{:?}",
        report.refusal()
    );
    assert_eq!((report.fired(), report.failed()), (2, 0));
    assert!(report.refusal().is_none());
    home.close_clean().unwrap();

    // The created Tasks are listed by their Inbox after a cold reopen; the
    // seeded Task holding the colliding id was never linked from it.
    let home = open(&path, &clock);
    let filed = vec![
        ("home/daily#1".to_owned(), 1),
        ("home/daily#2".to_owned(), 2),
    ];
    assert_eq!(inbox_tasks(&home, &clock), filed);
    // Re-catch-up at the same instant fires nothing more.
    assert_eq!(catch_up(&home, &clock, 2_500).fired(), 0);
    assert_eq!(inbox_tasks(&home, &clock), filed);
    assert_eq!(rule(&home, &clock, "home/daily").fired_count, 2);

    // `home/taken` fires task id `home/taken#1`, which the seeded Task holds.
    assert_eq!(TAKEN_TASK_ID, "home/taken#1");
    schedule(&home, &clock, "home/taken", 2_600);
    let refused = catch_up(&home, &clock, 2_700);
    assert_eq!(
        refused.status(),
        &WorthQueryTemporalCatchUpStatus::OperationRefused
    );
    assert_eq!((refused.fired(), refused.failed()), (0, 1));
    let refusal = refused.refusal().expect("typed refusal cause");
    assert_eq!(
        refusal.kind(),
        WorthQueryTemporalOccurrenceRefusalKind::UniqueValueTaken,
        "{refusal:?}"
    );
    assert_eq!(
        refusal.refusal_category(),
        WorthQueryRefusalCategory::Conflict
    );
    let message = refusal.user_message();
    assert!(message.starts_with(char::is_uppercase) && message.ends_with('.'));
    for jargon in ["::", "Worth", "Denial", "_"] {
        assert!(!message.contains(jargon), "{message}");
    }
    // The refused fire committed nothing, `fired_count` included.
    assert_eq!(rule(&home, &clock, "home/taken").fired_count, 0);
    assert_eq!(inbox_tasks(&home, &clock), filed);
    // The refused fire committed nothing and owes no cleanup, so once its
    // report (which carries invalidation deliveries) is dropped the Home
    // retires cleanly; the reopened Home holds no partial effect of it.
    drop(refused);
    if let Err((_, stop)) = home
        .into_retirement()
        .try_retire(DEFAULT_HOST_EXECUTION_POLICY)
    {
        panic!("a permanently refused fire leaves nothing to retain: {stop:?}");
    }
    let home = open(&path, &clock);
    assert_eq!(rule(&home, &clock, "home/taken").fired_count, 0);
    assert_eq!(inbox_tasks(&home, &clock), filed);
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn targeted_rule_scheduled_in_session_files_its_tasks_on_same_session_catch_up() {
    let mut path =
        std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("recurring-target-session");
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    let clock = WorthQueryAdmissionClock::system();

    let home = open(&path, &clock);
    let session = home.session(owner(&home, &clock));
    session
        .run(ScheduleRule {
            rule_id: "home/daily".to_owned(),
            due_ns: 1_000,
            interval_ns: 1_000,
            max_fires: 10,
            lifecycle: WorthQueryRecurringLifecycle::Active,
        })
        .unwrap_or_else(|denial| panic!("schedule: {denial:?}"));
    let catch_up_in_session = |now_ns| {
        session
            .catch_up_temporal::<Recurrence>(
                WorthQueryClockCoordinate::from_nanoseconds(now_ns),
                WorthQueryTemporalCatchUpPolicy::bounded(16).unwrap(),
            )
            .unwrap_or_else(|denial| panic!("same-session catch-up: {denial}"))
            .fired()
    };
    // Occurrences 1000 and 2000 are due in the session that scheduled them.
    assert_eq!(catch_up_in_session(2_500), 2);
    let filed = vec![
        ("home/daily#1".to_owned(), 1),
        ("home/daily#2".to_owned(), 2),
    ];
    assert_eq!(inbox_tasks(&home, &clock), filed);
    assert_eq!(catch_up_in_session(2_500), 0);
    assert_eq!(inbox_tasks(&home, &clock), filed);
    drop(session);
    home.close_clean().unwrap();

    let home = open(&path, &clock);
    assert_eq!(catch_up(&home, &clock, 2_500).fired(), 0);
    assert_eq!(inbox_tasks(&home, &clock), filed);
    assert_eq!(rule(&home, &clock, "home/daily").fired_count, 2);
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}
