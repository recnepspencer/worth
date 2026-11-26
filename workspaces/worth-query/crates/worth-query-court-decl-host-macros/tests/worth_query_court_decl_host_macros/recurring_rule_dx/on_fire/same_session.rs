//! A rule scheduled in a session fires on that session's own catch-up, with
//! no reopen: across intervening commits, beside another rule due at the same
//! instant, idempotently, and in agreement with a cold reopen afterwards.
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    domain::WorthQueryClockCoordinate,
    temporal_contribution::{WorthQueryRecurringLifecycle, WorthQueryTemporalCatchUpPolicy},
    WorthQueryHostSession,
};

use super::program::{open, owner, FireHome, FireProgram};
use super::schema::{FireSchema, ReadRuleTasks, Recurrence, ScheduleRule};
use super::{catch_up, fired_count};

type Session<'home> = WorthQueryHostSession<'home, FireSchema, FireProgram>;

fn fresh_home(name: &str, clock: &WorthQueryAdmissionClock) -> (std::path::PathBuf, FireHome) {
    let mut path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    let home = open(&path, clock);
    (path, home)
}

fn schedule(session: &Session<'_>, rule_id: &str, due_ns: u64) {
    session
        .run(ScheduleRule {
            rule_id: rule_id.to_owned(),
            due_ns,
            interval_ns: 1_000,
            max_fires: 10,
            lifecycle: WorthQueryRecurringLifecycle::Active,
        })
        .unwrap_or_else(|denial| panic!("schedule {rule_id}: {denial:?}"));
}

fn catch_up_in(session: &Session<'_>, now_ns: u64) -> usize {
    session
        .catch_up_temporal::<Recurrence>(
            WorthQueryClockCoordinate::from_nanoseconds(now_ns),
            WorthQueryTemporalCatchUpPolicy::bounded(16).unwrap(),
        )
        .unwrap_or_else(|denial| panic!("same-session catch-up: {denial}"))
        .fired()
}

fn task_keys(session: &Session<'_>, rule: &str) -> Vec<String> {
    session
        .query(ReadRuleTasks {
            rule: rule.to_owned(),
        })
        .expect("rule tasks read")
        .into_iter()
        .map(|task| task.task_key)
        .collect()
}

#[test]
fn on_fire_rule_scheduled_in_session_fires_on_same_session_catch_up() {
    let clock = WorthQueryAdmissionClock::system();
    let (path, home) = fresh_home("recurring-on-fire-session", &clock);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    schedule(&session, "daily", 1_000);
    // Occurrences 1000 and 2000 are due without any reopen.
    assert_eq!(catch_up_in(&session, 2_500), 2);
    assert_eq!(task_keys(&session, "daily"), ["daily#1", "daily#2"]);
    assert_eq!(catch_up_in(&session, 2_500), 0);
    assert_eq!(task_keys(&session, "daily"), ["daily#1", "daily#2"]);
    drop(session);
    home.close_clean().unwrap();

    let home = open(&path, &clock);
    assert_eq!(catch_up(&home, &clock, 2_500), 0);
    assert_eq!(fired_count(&home, &clock), 2);
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}

/// Each authoritative commit between catch-ups advances the shared
/// conditional node's version; the next occurrence must still fire.
#[test]
fn on_fire_rule_fires_its_next_occurrence_after_intervening_commits() {
    let clock = WorthQueryAdmissionClock::system();
    let (path, home) = fresh_home("recurring-on-fire-intervening", &clock);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    schedule(&session, "daily", 1_000);
    assert_eq!(catch_up_in(&session, 1_500), 1);
    assert_eq!(task_keys(&session, "daily"), ["daily#1"]);
    // Commits between catch-ups: another rule, due much later, is scheduled.
    schedule(&session, "later", 90_000);
    assert_eq!(
        catch_up_in(&session, 2_500),
        1,
        "occurrence 2 after one commit"
    );
    schedule(&session, "later-still", 95_000);
    schedule(&session, "latest", 99_000);
    assert_eq!(
        catch_up_in(&session, 3_500),
        1,
        "occurrence 3 after two commits"
    );
    assert_eq!(
        task_keys(&session, "daily"),
        ["daily#1", "daily#2", "daily#3"]
    );
    assert_eq!(catch_up_in(&session, 3_500), 0);
    drop(session);
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}

/// Two rules due at once retain one wake each and both fire, across one
/// catch-up and the next, with no wake read as an unchanged recompute.
#[test]
fn on_fire_rules_due_together_in_session_each_fire() {
    let clock = WorthQueryAdmissionClock::system();
    let (path, home) = fresh_home("recurring-on-fire-together", &clock);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    schedule(&session, "daily", 1_000);
    schedule(&session, "nightly", 1_000);
    assert_eq!(catch_up_in(&session, 1_500), 2);
    assert_eq!(catch_up_in(&session, 2_500), 2);
    assert_eq!(task_keys(&session, "daily"), ["daily#1", "daily#2"]);
    assert_eq!(task_keys(&session, "nightly"), ["nightly#1", "nightly#2"]);
    assert_eq!(catch_up_in(&session, 2_500), 0);
    drop(session);
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}
