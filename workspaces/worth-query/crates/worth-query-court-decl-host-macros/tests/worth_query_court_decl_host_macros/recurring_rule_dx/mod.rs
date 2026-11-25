//! Stock recurring rule installed declaratively: scheduling, bounded catch-up
//! after reopen, completion and idempotent re-catch-up on a Directory Home.
mod on_fire;
mod program;
mod schema;

use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    domain::WorthQueryClockCoordinate,
    temporal_contribution::{WorthQueryRecurringLifecycle, WorthQueryTemporalCatchUpPolicy},
};

use program::{open, owner, RecurringHome};
use schema::{ReadRules, Recurrence, RuleView, ScheduleRule, HUB};

fn rules(home: &RecurringHome, clock: &WorthQueryAdmissionClock) -> Vec<RuleView> {
    home.session(owner(home, clock))
        .query(ReadRules {
            hub: HUB.to_owned(),
        })
        .expect("rules read")
}

fn catch_up(home: &RecurringHome, clock: &WorthQueryAdmissionClock, now_ns: u64) -> usize {
    let report = home
        .session(owner(home, clock))
        .catch_up_temporal::<Recurrence>(
            WorthQueryClockCoordinate::from_nanoseconds(now_ns),
            WorthQueryTemporalCatchUpPolicy::bounded(16).unwrap(),
        )
        .unwrap_or_else(|denial| panic!("bounded catch-up: {denial}"));
    report.fired()
}

#[test]
fn recurring_rule_fires_each_missed_occurrence_once_and_completes_across_reopen() {
    let mut path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("recurring-rule-dx");
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    let clock = WorthQueryAdmissionClock::system();

    let home = open(&path, &clock);
    home.session(owner(&home, &clock))
        .run(ScheduleRule {
            rule_id: "weekly".to_owned(),
            due_ns: 1_000,
            interval_ns: 1_000,
            max_fires: 3,
            lifecycle: WorthQueryRecurringLifecycle::Active,
        })
        .unwrap_or_else(|denial| panic!("schedule: {denial:?}"));
    let scheduled = rules(&home, &clock);
    assert_eq!(scheduled.len(), 1);
    assert_eq!((scheduled[0].fired_count, scheduled[0].revision), (0, 1));
    assert_eq!(scheduled[0].lifecycle, WorthQueryRecurringLifecycle::Active);
    let hub_overwrite = home.session(owner(&home, &clock)).run(ScheduleRule {
        rule_id: HUB.to_owned(),
        due_ns: 1,
        interval_ns: 1,
        max_fires: 1,
        lifecycle: WorthQueryRecurringLifecycle::Active,
    });
    assert!(
        hub_overwrite.is_err(),
        "the hub row is never rescheduled as a rule"
    );
    assert_eq!(rules(&home, &clock), scheduled);
    home.close_clean().unwrap();

    // Three occurrences (1000, 2000, 3000) were missed while closed.
    let home = open(&path, &clock);
    assert_eq!(catch_up(&home, &clock, 5_000), 3);
    let completed = rules(&home, &clock);
    assert_eq!(completed[0].fired_count, 3);
    assert_eq!(completed[0].due_ns, 3_000);
    assert_eq!(
        completed[0].lifecycle,
        WorthQueryRecurringLifecycle::Completed
    );
    home.close_clean().unwrap();

    let home = open(&path, &clock);
    assert_eq!(catch_up(&home, &clock, 10_000), 0);
    assert_eq!(rules(&home, &clock), completed);
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}
