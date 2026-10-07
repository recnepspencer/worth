use super::super::super::support::*;
use super::fixtures::local_identity_read;
use crate::ordinary::history::{
    at, declare as declare_history, WorthQueryHistoricalNextAction, WorthQueryHistoricalStopSource,
};
use crate::ordinary::read::{current, declare as declare_read};
use crate::runtime::tests::support::{
    insert_command, stateful_bridge_task_runtime, test_string_aspect_value,
};

#[test]
fn retained_history_preserves_current_result_meaning_on_the_same_truth_basis() {
    let current_declaration =
        declare_read(local_identity_read).expect("current declaration should build");
    let historical_declaration = declare_history(local_identity_read)
        .expect("historical declaration should build")
        .retained_snapshot();
    let mut workspace = read_runtime()
        .workspace("ordinary-retained-history-parity")
        .expect("workspace should open");
    let historical_context = at(&workspace).expect("the fixture requires an open workspace");

    let current = current_declaration
        .using(current())
        .run(&mut workspace)
        .into_result()
        .expect("current read should execute")
        .into_result();
    let historical = historical_declaration
        .using(historical_context)
        .run(&mut workspace)
        .into_result()
        .expect("retained history should execute");

    assert_eq!(historical.result().rows(), current.rows());
    assert_eq!(
        historical.result().receipt().query_digest(),
        current.receipt().query_digest()
    );
    assert_eq!(
        historical.materialization().resolved_path_class().as_str(),
        "resolved_retained_snapshot_path"
    );
    assert_eq!(
        historical
            .journey_counters()
            .lower_runtime_execution_attempt_count(),
        1
    );
}

#[test]
fn unavailable_replay_stops_before_context_planning_or_runtime_contact() {
    let declaration = declare_history(local_identity_read)
        .expect("historical declaration should build")
        .delta_replay(8);
    let mut workspace = read_runtime()
        .workspace("ordinary-history-replay-unavailable")
        .expect("workspace should open");
    let context = at(&workspace).expect("the fixture requires an open workspace");
    let stop = declaration
        .using(context)
        .run(&mut workspace)
        .into_result()
        .expect_err("runtime replay should be explicitly unavailable");

    assert_eq!(
        stop.source(),
        WorthQueryHistoricalStopSource::HistoryUnavailable
    );
    assert_eq!(
        stop.next_action(),
        WorthQueryHistoricalNextAction::SupplyAvailableHistory
    );
    assert_eq!(stop.journey_counters().context_admission_attempt_count(), 0);
    assert_eq!(stop.journey_counters().planning_attempt_count(), 0);
    assert_eq!(
        stop.journey_counters()
            .lower_runtime_execution_attempt_count(),
        0
    );
}

#[test]
fn stale_retained_context_stops_before_planning_or_runtime_contact() {
    let declaration = declare_history(local_identity_read)
        .expect("historical declaration should build")
        .retained_snapshot();
    let mut workspace = stateful_bridge_task_runtime()
        .workspace("ordinary-history-stale")
        .expect("workspace should open");
    let context = at(&workspace).expect("the fixture requires an open workspace");
    workspace
        .write(insert_command(
            "Task",
            [
                ("identity.id", test_string_aspect_value("changed")),
                ("title.value", test_string_aspect_value("Changed")),
            ],
        ))
        .expect("write should advance the truth snapshot");

    let stop = declaration
        .using(context)
        .run(&mut workspace)
        .into_result()
        .expect_err("stale history context must stop");
    assert_eq!(stop.source(), WorthQueryHistoricalStopSource::StaleContext);
    assert_eq!(
        stop.next_action(),
        WorthQueryHistoricalNextAction::RefreshContext
    );
    assert_eq!(stop.journey_counters().planning_attempt_count(), 0);
    assert_eq!(
        stop.journey_counters()
            .lower_runtime_execution_attempt_count(),
        0
    );
}
