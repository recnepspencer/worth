//! Single-harness 64-seed DST + TLC trace emitter.
//!
//! Seeds 0..56 run on MemoryOnly Homes; seeds 56..64 run on real Directory Homes with
//! genuine close/reopen cycles and offline clock jumps. Set `WORTH_DST_TRACE_DIR` to emit
//! `temporal_catchup.ndjson` for `python3 tools/tla/run_tlc.py trace
//! specs/tla/temporal_catchup/TemporalCatchupTrace.tla`.

mod actions;
mod reference_model;

use actions::*;
use reference_model::*;
use worth_execution::ExecutionRequest;

use super::{
    adapters::{AuthControl, ReminderFireLog},
    program::*,
    schema::{MutateReminderInput, ReminderRow},
    with_leased_request,
};

const SEED_COUNT: u64 = 64;
const DIRECTORY_SEED_START: u64 = 56;

struct SeedState {
    oracle: ScheduleOracle,
    clock: u64,
    cmd_key: u64,
    last_status: &'static str,
    closed_rows: std::collections::BTreeMap<String, ReminderRow>,
}

fn passive_observed_clock(home: &ReminderHome) -> u64 {
    let inst = installed(home);
    if inst.desktop_clock().last_sequence() == 0 {
        0
    } else {
        inst.desktop_clock().current_time().nanoseconds()
    }
}

#[test]
fn temporal_catchup_dst_64_seeds() {
    let mut trace = TraceEmitter::new();
    for seed in 0..SEED_COUNT {
        if seed > 0 {
            trace.push_reset();
        }
        run_seed(seed, seed >= DIRECTORY_SEED_START, &mut trace);
    }
    trace.flush_if_configured();
}

fn run_seed(seed: u64, is_directory: bool, trace: &mut TraceEmitter) {
    let mut rng = Rng::new(seed);
    let fire_log = ReminderFireLog::default();
    let auth = AuthControl::valid();
    let dir = is_directory.then(|| unique_temp_dir(&format!("dst-{seed}")));

    with_leased_request(128 * 1024 * 1024, None, |execution| {
        let mut home = Some(open_seed_home(
            dir.as_ref(),
            fire_log.clone(),
            auth.clone(),
            execution,
        ));
        let mut st = SeedState {
            oracle: ScheduleOracle::new(),
            clock: 0,
            cmd_key: 1,
            last_status: "Idle",
            closed_rows: empty_rows(),
        };

        install_initial_intents(
            home.as_ref().unwrap(),
            &mut rng,
            &fire_log,
            &auth,
            &mut st,
            trace,
        );

        for _ in 0..10 {
            step_seed(
                &mut home,
                dir.as_ref(),
                is_directory,
                &mut rng,
                &fire_log,
                &auth,
                &mut st,
                execution,
                trace,
            );
        }

        if home.is_none() {
            let reopened = open_seed_home(dir.as_ref(), fire_log.clone(), auth.clone(), execution);
            let rows = read_reminders(&reopened);
            st.oracle.verify_against_rows(&rows);
            trace.push(
                "ReopenApp",
                serde_json::json!({}),
                "Open",
                st.clock,
                passive_observed_clock(&reopened),
                auth.is_valid(),
                "Idle",
                passive_total_fired(&rows),
                &rows,
            );
            home = Some(reopened);
        }
        let h = home.as_ref().unwrap();
        for _ in 0..16 {
            st.last_status = pump_one_step(
                h,
                st.clock,
                &fire_log,
                &auth,
                &mut st.oracle,
                execution,
                trace,
            );
            if st.last_status == "CaughtUp" {
                break;
            }
        }
        assert_eq!(st.last_status, "CaughtUp", "seed {seed} must converge");
        assert_eq!(st.oracle.due_active_count(st.clock), 0);
        if let Some(h) = home.take()
            && is_directory
        {
            h.close(execution).unwrap().reclaim();
        }
    });

    if let Some(dir) = dir {
        let _ = std::fs::remove_dir_all(&dir);
    }
}

fn install_initial_intents(
    home: &ReminderHome,
    rng: &mut Rng,
    _fire_log: &ReminderFireLog,
    auth: &AuthControl,
    st: &mut SeedState,
    trace: &mut TraceEmitter,
) {
    for &id in &INTENTS {
        let max_fires = rng.range(1, 3);
        let due = rng.range(2, 10);
        let interval = if max_fires == 1 { 0 } else { rng.range(3, 7) };
        mutate_reminder(
            home,
            MutateReminderInput {
                identity: id.to_owned(),
                revision: 1,
                due,
                interval,
                max_fires,
                fired_count: 0,
                lifecycle: "active".to_owned(),
            },
            st.cmd_key,
        );
        st.cmd_key += 1;
        st.oracle.intents.insert(
            id,
            OracleIntent {
                lifecycle: "Active",
                rev: 1,
                due,
                interval,
                max_fires,
                fired_count: 0,
            },
        );
        let rows = read_reminders(home);
        st.oracle.verify_against_rows(&rows);
        if due <= st.clock {
            st.last_status = "Idle";
        }
        trace.push(
            "InstallIntent",
            serde_json::json!({ "id": id, "due": due, "interval": interval, "maxFires": max_fires }),
            "Open",
            st.clock,
            passive_observed_clock(home),
            auth.is_valid(),
            st.last_status,
            passive_total_fired(&rows),
            &rows,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn step_seed(
    home: &mut Option<ReminderHome>,
    dir: Option<&std::path::PathBuf>,
    is_directory: bool,
    rng: &mut Rng,
    fire_log: &ReminderFireLog,
    auth: &AuthControl,
    st: &mut SeedState,
    execution: &ExecutionRequest<'_, '_>,
    trace: &mut TraceEmitter,
) {
    if home.is_none() {
        if rng.pick(2) == 0 && st.clock < 80 {
            st.clock += rng.range(3, 12);
            st.last_status = "Closed";
            trace.push(
                "AdvanceClock",
                serde_json::json!({ "newClock": st.clock }),
                "Closed",
                st.clock,
                0,
                auth.is_valid(),
                st.last_status,
                passive_total_fired(&st.closed_rows),
                &st.closed_rows,
            );
        } else {
            let reopened = open_seed_home(dir, fire_log.clone(), auth.clone(), execution);
            let rows = read_reminders(&reopened);
            st.oracle.verify_against_rows(&rows);
            st.last_status = "Idle";
            trace.push(
                "ReopenApp",
                serde_json::json!({}),
                "Open",
                st.clock,
                passive_observed_clock(&reopened),
                auth.is_valid(),
                st.last_status,
                passive_total_fired(&rows),
                &rows,
            );
            *home = Some(reopened);
        }
        return;
    }

    match rng.pick(10) {
        0..=2 if st.clock < 80 => {
            st.clock += rng.range(3, 12);
            st.last_status = "Idle";
            let h = home.as_ref().unwrap();
            let rows = read_reminders(h);
            trace.push(
                "AdvanceClock",
                serde_json::json!({ "newClock": st.clock }),
                "Open",
                st.clock,
                passive_observed_clock(h),
                auth.is_valid(),
                st.last_status,
                passive_total_fired(&rows),
                &rows,
            );
        }
        3 if is_directory => {
            let h = home.take().unwrap();
            st.closed_rows = read_reminders(&h);
            h.close(execution).expect("directory close").reclaim();
            st.last_status = "Closed";
            trace.push(
                "CloseApp",
                serde_json::json!({}),
                "Closed",
                st.clock,
                0,
                auth.is_valid(),
                st.last_status,
                passive_total_fired(&st.closed_rows),
                &st.closed_rows,
            );
        }
        4 => exercise_cancelled_pump(home.as_ref().unwrap(), fire_log, auth, st, trace),
        5 => exercise_budget_pump(home.as_ref().unwrap(), fire_log, auth, st, trace),
        6 => exercise_auth_refusal(home.as_ref().unwrap(), fire_log, auth, st, execution, trace),
        7 => exercise_user_edit(home.as_ref().unwrap(), rng, fire_log, auth, st, trace),
        _ => {
            let h = home.as_ref().unwrap();
            st.last_status = pump_one_step(
                h,
                st.clock,
                fire_log,
                auth,
                &mut st.oracle,
                execution,
                trace,
            );
        }
    }
}

fn open_seed_home(
    dir: Option<&std::path::PathBuf>,
    fire_log: ReminderFireLog,
    auth: AuthControl,
    execution: &ExecutionRequest<'_, '_>,
) -> ReminderHome {
    match dir {
        Some(path) => open_directory_home(path.clone(), fire_log, auth, execution),
        None => open_memory_home(fire_log, auth, execution),
    }
}
