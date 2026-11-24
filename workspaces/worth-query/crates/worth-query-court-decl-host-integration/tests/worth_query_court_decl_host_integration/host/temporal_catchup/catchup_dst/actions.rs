//! Individual DST step actions and strictly passive-observation pump helpers.
//!
//! Action eligibility, mutation inputs, and emitted trace coordinates are
//! derived strictly from the RNG seed, live Relational rows, and real Host
//! catch-up receipts — NEVER from the reference oracle.

use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    domain::WorthQueryClockCoordinate,
    temporal_contribution::{WorthQueryTemporalCatchUpPolicy, WorthQueryTemporalCatchUpStatus},
};

use super::{
    reference_model::{passive_due_active_count, passive_total_fired, Rng, TraceEmitter, INTENTS},
    SeedState,
};
use crate::host::temporal_catchup::{
    adapters::{AuthControl, ReminderFireLog},
    program::*,
    schema::MutateReminderInput,
    with_cancelled_request, with_leased_request,
};

pub(super) fn exercise_cancelled_pump(
    h: &ReminderHome,
    _fire_log: &ReminderFireLog,
    auth: &AuthControl,
    st: &mut SeedState,
    trace: &mut TraceEmitter,
) {
    let mut accepted_now = st.last_accepted_clock;
    with_cancelled_request(|c_exec| {
        let report = h
            .pump_temporal::<DesktopReminderConditional>(
                c_exec,
                WorthQueryTemporalCatchUpPolicy::bounded(1).unwrap(),
            )
            .unwrap();
        assert_eq!(report.status(), &WorthQueryTemporalCatchUpStatus::Cancelled);
        assert_eq!(report.fired(), 0);
        if let Some(observed) = report.last_observed_time() {
            accepted_now = observed.nanoseconds();
        }
    });
    st.last_accepted_clock = accepted_now;
    let rows = read_reminders(h);
    st.oracle.verify_against_rows(&rows);
    st.last_status = "Cancelled";
    trace.push(
        "PumpCancelled",
        serde_json::json!({ "targetNow": accepted_now }),
        "Open",
        st.clock,
        st.last_accepted_clock,
        auth.is_valid(),
        st.last_status,
        passive_total_fired(&rows),
        &rows,
    );
}

pub(super) fn exercise_budget_pump(
    h: &ReminderHome,
    _fire_log: &ReminderFireLog,
    auth: &AuthControl,
    st: &mut SeedState,
    trace: &mut TraceEmitter,
) {
    let pre_rows = read_reminders(h);
    if passive_due_active_count(&pre_rows, st.last_accepted_clock) == 0 {
        return;
    }
    let mut accepted_now = st.last_accepted_clock;
    with_leased_request(128 * 1024 * 1024, Some(1), |tiny_exec| {
        let report = h
            .pump_temporal::<DesktopReminderConditional>(
                tiny_exec,
                WorthQueryTemporalCatchUpPolicy::bounded(1).unwrap(),
            )
            .unwrap();
        assert_eq!(
            report.status(),
            &WorthQueryTemporalCatchUpStatus::BudgetExhausted
        );
        assert_eq!(report.fired(), 0);
        if let Some(observed) = report.last_observed_time() {
            accepted_now = observed.nanoseconds();
        }
    });
    st.last_accepted_clock = accepted_now;
    let rows = read_reminders(h);
    st.oracle.verify_against_rows(&rows);
    st.last_status = "BudgetExhausted";
    trace.push(
        "PumpBudgetExhausted",
        serde_json::json!({ "targetNow": accepted_now }),
        "Open",
        st.clock,
        st.last_accepted_clock,
        auth.is_valid(),
        st.last_status,
        passive_total_fired(&rows),
        &rows,
    );
}

pub(super) fn exercise_auth_refusal(
    h: &ReminderHome,
    _fire_log: &ReminderFireLog,
    auth: &AuthControl,
    st: &mut SeedState,
    execution: &ExecutionRequest<'_, '_>,
    trace: &mut TraceEmitter,
) {
    let pre_rows = read_reminders(h);
    if !auth.is_valid() || passive_due_active_count(&pre_rows, st.last_accepted_clock) == 0 {
        return;
    }
    auth.set_valid(false);
    trace.push(
        "ExpireAuth",
        serde_json::json!({}),
        "Open",
        st.clock,
        st.last_accepted_clock,
        false,
        st.last_status,
        passive_total_fired(&pre_rows),
        &pre_rows,
    );
    let report = h
        .pump_temporal::<DesktopReminderConditional>(
            execution,
            WorthQueryTemporalCatchUpPolicy::bounded(1).unwrap(),
        )
        .unwrap();
    assert_eq!(
        report.status(),
        &WorthQueryTemporalCatchUpStatus::AuthorizationRefused
    );
    assert_eq!(report.fired(), 0);
    if let Some(observed) = report.last_observed_time() {
        st.last_accepted_clock = observed.nanoseconds();
    }
    let rows = read_reminders(h);
    st.oracle.verify_against_rows(&rows);
    st.last_status = "AuthorizationRefused";
    trace.push(
        "PumpUnauthorized",
        serde_json::json!({ "targetNow": st.last_accepted_clock }),
        "Open",
        st.clock,
        st.last_accepted_clock,
        false,
        st.last_status,
        passive_total_fired(&rows),
        &rows,
    );
    auth.set_valid(true);
    trace.push(
        "RefreshAuth",
        serde_json::json!({}),
        "Open",
        st.clock,
        st.last_accepted_clock,
        true,
        st.last_status,
        passive_total_fired(&rows),
        &rows,
    );
}

pub(super) fn exercise_user_edit(
    h: &ReminderHome,
    rng: &mut Rng,
    _fire_log: &ReminderFireLog,
    auth: &AuthControl,
    st: &mut SeedState,
    trace: &mut TraceEmitter,
) {
    let id = INTENTS[rng.pick(INTENTS.len())];
    let pre_rows = read_reminders(h);
    let cur = &pre_rows[id];
    if cur.lifecycle != "active" {
        return;
    }
    if rng.pick(2) == 0 {
        mutate_reminder(
            h,
            MutateReminderInput {
                identity: id.to_owned(),
                revision: cur.revision + 1,
                due: cur.due,
                interval: cur.interval,
                max_fires: cur.max_fires,
                fired_count: cur.fired_count,
                lifecycle: "cancelled".to_owned(),
            },
            st.cmd_key,
        );
        st.cmd_key += 1;
        let entry = st.oracle.intents.get_mut(id).unwrap();
        entry.lifecycle = "Cancelled";
        entry.rev += 1;
        let rows = read_reminders(h);
        st.oracle.verify_against_rows(&rows);
        trace.push(
            "CancelIntent",
            serde_json::json!({ "id": id }),
            "Open",
            st.clock,
            st.last_accepted_clock,
            auth.is_valid(),
            st.last_status,
            passive_total_fired(&rows),
            &rows,
        );
    } else {
        let new_due = cur.due + rng.range(1, 5);
        mutate_reminder(
            h,
            MutateReminderInput {
                identity: id.to_owned(),
                revision: cur.revision + 1,
                due: new_due,
                interval: cur.interval,
                max_fires: cur.max_fires,
                fired_count: cur.fired_count,
                lifecycle: "active".to_owned(),
            },
            st.cmd_key,
        );
        st.cmd_key += 1;
        let entry = st.oracle.intents.get_mut(id).unwrap();
        entry.rev += 1;
        entry.due = new_due;
        if new_due <= st.clock {
            st.last_status = "Idle";
        }
        let rows = read_reminders(h);
        st.oracle.verify_against_rows(&rows);
        trace.push(
            "AmendIntentDue",
            serde_json::json!({ "id": id, "newDue": new_due }),
            "Open",
            st.clock,
            st.last_accepted_clock,
            auth.is_valid(),
            st.last_status,
            passive_total_fired(&rows),
            &rows,
        );
    }
}

pub(super) fn pump_one_step(
    home: &ReminderHome,
    _fire_log: &ReminderFireLog,
    auth: &AuthControl,
    st: &mut SeedState,
    execution: &ExecutionRequest<'_, '_>,
    trace: &mut TraceEmitter,
) -> &'static str {
    let target_now = st.clock;
    let before_rows = read_reminders(home);
    let report = home
        .catch_up_temporal::<DesktopReminderConditional>(
            WorthQueryClockCoordinate::from_nanoseconds(target_now),
            execution,
            WorthQueryTemporalCatchUpPolicy::bounded(1).unwrap(),
        )
        .unwrap();
    let accepted_now = report
        .last_observed_time()
        .expect("accepted catch-up step must report its actual observed coordinate")
        .nanoseconds();
    st.last_accepted_clock = accepted_now;
    let rows = read_reminders(home);
    let committed: Vec<crate::host::temporal_catchup::adapters::FiredOccurrence> = INTENTS
        .iter()
        .filter_map(|&id| {
            let before = &before_rows[id];
            let after = &rows[id];
            (after.fired_count > before.fired_count).then(|| {
                assert_eq!(after.fired_count, before.fired_count + 1);
                crate::host::temporal_catchup::adapters::FiredOccurrence {
                    id: id.to_owned(),
                    rev: before.revision,
                    due: before.due,
                    fired_count_after: after.fired_count,
                }
            })
        })
        .collect();
    assert_eq!(
        committed.len(),
        report.fired(),
        "committed Relational rows must match receipt fired count"
    );
    for fired in &committed {
        st.oracle.record_fire(fired, accepted_now);
    }
    st.oracle.verify_against_rows(&rows);
    assert!(
        committed.len() <= 1,
        "single-commit-per-snapshot advancement commits at most one wake per step"
    );
    if let [fired] = committed.as_slice() {
        trace.push(
            "PumpFire",
            serde_json::json!({ "id": fired.id, "targetNow": accepted_now }),
            "Open",
            target_now,
            accepted_now,
            auth.is_valid(),
            "Progress",
            passive_total_fired(&rows),
            &rows,
        );
    }
    match report.status() {
        WorthQueryTemporalCatchUpStatus::CaughtUp => {
            assert_eq!(passive_due_active_count(&rows, accepted_now), 0);
            assert_eq!(st.oracle.due_active_count(accepted_now), 0);
            trace.push(
                "PumpCatchUpComplete",
                serde_json::json!({ "targetNow": accepted_now }),
                "Open",
                target_now,
                accepted_now,
                auth.is_valid(),
                "CaughtUp",
                passive_total_fired(&rows),
                &rows,
            );
            "CaughtUp"
        }
        WorthQueryTemporalCatchUpStatus::StepLimitReached => {
            assert!(passive_due_active_count(&rows, accepted_now) > 0);
            assert!(st.oracle.due_active_count(accepted_now) > 0);
            trace.push(
                "PumpStepLimit",
                serde_json::json!({ "targetNow": accepted_now }),
                "Open",
                target_now,
                accepted_now,
                auth.is_valid(),
                "StepLimitReached",
                passive_total_fired(&rows),
                &rows,
            );
            "StepLimitReached"
        }
        other => {
            let obligations = home.runtime().conditional_decision_obligations();
            let runs = home.runtime().managed_run_obligations();
            panic!(
                "unexpected pump status in DST step: {other:?}, report={report:?}, target_now={target_now}, accepted_now={accepted_now}, rows={rows:?}, obligations={obligations:?}, runs={runs:?}"
            );
        }
    }
}
