//! Single-harness Seeded Differential Query DST + Reference Graph Oracle
//! covering optional parameters, literal constants, multi-predicate exact-root
//! queries, related-record absence/existence (forward & reverse, with & without
//! target predicates/filters), out-of-scope blocker invalidation, real native
//! footprint currentness checks, and pinned/retained historical reads.
//!
//! `WORTH_DST_SEEDS` (default 64) controls seed count, `WORTH_DST_SEED` runs
//! one seed, and `WORTH_DST_TRACE_DIR` emits TLA+ NDJSON traces for
//! `tools/tla/run_tlc.py trace`.
use super::mutations::MutateBoard;
use super::world::{install, observe_ready_source, ready_with_source, run_mutate, Host, TaskSeed};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationReadObservation, WorthQueryApplicationRequestExt,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

mod reference;
mod trace;
mod verify;

use reference::{GraphSnapshot, TaskNode};
use trace::Trace;
use verify::{verify_step, Coverage, Rng};

const STEPS_PER_SEED: usize = 16;
const NOTES_POOL: [&str; 4] = ["alpha core", "beta flange", "gamma pin", "delta bolt"];

fn build_initial_graph(rng: &mut Rng, tracing: bool) -> (GraphSnapshot, Vec<TaskSeed>) {
    let mut oracle = GraphSnapshot::default();
    let mut seeds = Vec::with_capacity(6);
    for key in 1..=6_u64 {
        // When tracing, tasks 1..=5 are in project 1 (`ScopeTasks`) and task 6 is in
        // project 2 (an out-of-scope blocker!) matching `RelatedAbsenceTrace.tla`.
        // Otherwise tasks 1..=4 are in project 1 and 5..=6 are in project 2.
        let project = if tracing {
            if key <= 5 {
                1
            } else {
                2
            }
        } else if key <= 4 {
            1
        } else {
            2
        };
        let status = if tracing || rng.pick(3) != 0 {
            "open"
        } else {
            "done"
        };
        let rank = (rng.pick(10) as i64) + 1;
        let due_days = (rng.pick(3) != 0).then(|| 50 + (rng.pick(5) as i64) * 50);
        let notes = (rng.pick(3) != 0).then(|| NOTES_POOL[rng.pick(NOTES_POOL.len())].to_owned());
        oracle.tasks.insert(
            key,
            TaskNode {
                key,
                project,
                title: format!("Task {key}"),
                notes: notes.clone(),
                status: status.to_owned(),
                rank,
                due_days,
            },
        );
        seeds.push(TaskSeed {
            key,
            project,
            title: format!("Task {key}"),
            notes,
            status: status.to_owned(),
            rank,
            due_days,
        });
    }
    (oracle, seeds)
}

fn retain_basis(host: &Host) -> WorthQueryApplicationReadObservation {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .retain_current_read()
        .expect("retain basis succeeds")
}

#[allow(clippy::too_many_arguments)]
fn apply_random_step(
    host: &Host,
    oracle: &mut GraphSnapshot,
    rng: &mut Rng,
    trace: &mut Trace,
    cov: &mut Coverage,
    tracing: bool,
    idem: &mut u64,
) -> bool {
    let choice = rng.pick(10);
    if choice < 3 {
        let task = (rng.pick(6) as u64) + 1;
        let next = if oracle.tasks[&task].status == "open" {
            "done"
        } else {
            "open"
        };
        let proj = oracle.tasks[&task].project;
        run_mutate(
            host,
            MutateBoard {
                project: proj,
                op: 0,
                task,
                other: 0,
                text: next.to_owned(),
            },
            *idem,
        );
        *idem += 1;
        oracle.set_status(task, next);
        trace.toggle_status(task);
        cov.toggles += 1;
        if proj == 2 {
            cov.out_of_scope_toggles += 1;
        }
        true
    } else if choice < 6 {
        let from = (rng.pick(6) as u64) + 1;
        let to = (rng.pick(6) as u64) + 1;
        if !oracle.link(from, to) {
            return false;
        }
        let proj = oracle.tasks[&from].project;
        run_mutate(
            host,
            MutateBoard {
                project: proj,
                op: 2,
                task: from,
                other: to,
                text: String::new(),
            },
            *idem,
        );
        *idem += 1;
        trace.add_edge(from, to);
        cov.edge_adds += 1;
        if from == to {
            cov.self_loops += 1;
        }
        if oracle.blocked_by.contains(&(to, from))
            || oracle.blocked_by.iter().filter(|&&(_, b)| b == to).count() >= 2
        {
            cov.cycles_or_shared += 1;
        }
        true
    } else if choice < 8 && !oracle.blocked_by.is_empty() {
        let idx = rng.pick(oracle.blocked_by.len());
        let (from, to) = *oracle.blocked_by.iter().nth(idx).unwrap();
        oracle.unlink(from, to);
        let proj = oracle.tasks[&from].project;
        run_mutate(
            host,
            MutateBoard {
                project: proj,
                op: 3,
                task: from,
                other: to,
                text: String::new(),
            },
            *idem,
        );
        *idem += 1;
        trace.remove_edge(from, to);
        cov.edge_removes += 1;
        true
    } else if !tracing {
        let task = (rng.pick(6) as u64) + 1;
        let note = NOTES_POOL[rng.pick(NOTES_POOL.len())];
        let proj = oracle.tasks[&task].project;
        run_mutate(
            host,
            MutateBoard {
                project: proj,
                op: 1,
                task,
                other: 0,
                text: note.to_owned(),
            },
            *idem,
        );
        *idem += 1;
        oracle.set_notes(task, note);
        cov.note_edits += 1;
        true
    } else {
        false
    }
}

fn run_seed(seed: u64, trace_dir: Option<&std::path::Path>) -> Result<Coverage, String> {
    let mut rng = Rng::new(seed);
    let tracing = trace_dir.is_some();
    let (mut oracle, seeds) = build_initial_graph(&mut rng, tracing);
    let host = install(&seeds, &[]);
    let mut trace = Trace::default();
    let mut cov = Coverage::default();
    let mut history = vec![(retain_basis(&host), oracle.clone())];
    let mut idem = 1_u64;

    // Seed initial Ready observation & source before first mutation.
    let (init_ready, init_source) = ready_with_source(&host, 1);
    if init_ready != oracle.ready_tasks(1) {
        return Err(format!("seed {seed} init ready mismatch"));
    }
    trace.observe("Absent", "Forward", true, &init_ready);
    let mut last_ready = (init_ready, init_source);

    for step in 0..STEPS_PER_SEED {
        if apply_random_step(
            &host,
            &mut oracle,
            &mut rng,
            &mut trace,
            &mut cov,
            tracing,
            &mut idem,
        ) {
            history.push((retain_basis(&host), oracle.clone()));
            let (ref prev_rows, ref prev_source) = last_ready;
            let still_current =
                observe_ready_source(&host, 1, prev_source, &format!("dst-cur-{seed}-{step}"))
                    .is_ok();
            trace.check_currentness(still_current);
            cov.currentness_checks += 1;
            if still_current && oracle.ready_tasks(1) != *prev_rows {
                return Err(format!(
                    "seed {seed} step {step}: STALE READ! source stayed current but ready changed from {prev_rows:?} to {:?}",
                    oracle.ready_tasks(1)
                ));
            }
        }
        last_ready = verify_step(
            seed, step, &host, &oracle, &history, &mut rng, &mut trace, &mut cov, tracing,
        )?;
    }
    if let Some(dir) = trace_dir {
        // Emit real governed denial observations into the trace (no hardcoded booleans!).
        let (rel_denied, field_denied) = super::governance::governed_trace_denial_probes();
        trace.observe_with_governance("Absent", "Forward", true, false, true, rel_denied, &[]);
        trace.observe_with_governance("Absent", "Forward", true, true, false, field_denied, &[]);
        trace
            .write(dir, seed)
            .map_err(|err| format!("trace write failed: {err}"))?;
    }
    Ok(cov)
}

fn seeds() -> Vec<u64> {
    let var = |name: &str| std::env::var(name).ok().and_then(|v| v.parse::<u64>().ok());
    match var("WORTH_DST_SEED") {
        Some(seed) => vec![seed],
        None => (0..var("WORTH_DST_SEEDS").unwrap_or(64)).collect(),
    }
}

#[test]
fn seeded_differential_query_dst_agrees_with_reference_graph_oracle() {
    let seeds = seeds();
    let trace_dir = std::env::var_os("WORTH_DST_TRACE_DIR").map(std::path::PathBuf::from);
    let mut failures = Vec::new();
    let mut total = Coverage::default();
    for &seed in &seeds {
        match run_seed(seed, trace_dir.as_deref()) {
            Ok(cov) => total.add(cov),
            Err(err) => failures.push(err),
        }
    }
    assert!(
        failures.is_empty(),
        "{} seeds failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
    eprintln!(
        "related-absence differential DST: {} seeds x {STEPS_PER_SEED} steps, {total:?}",
        seeds.len()
    );
    if seeds.len() >= 64 {
        assert!(total.toggles > 0, "must exercise status toggles");
        assert!(
            total.out_of_scope_toggles > 0,
            "must exercise out-of-scope blocker toggles"
        );
        assert!(
            total.currentness_checks > 0,
            "must exercise live currentness checks"
        );
        assert!(
            total.edge_adds > 0,
            "must exercise BlockedBy edge additions"
        );
        assert!(
            total.edge_removes > 0,
            "must exercise BlockedBy edge removals"
        );
        assert!(
            total.pinned_checks > 0,
            "must exercise pinned snapshot reads"
        );
        assert!(
            total.self_loops > 0,
            "must exercise self-loop BlockedBy edges"
        );
        assert!(
            total.cycles_or_shared > 0,
            "must exercise cycles or shared blockers"
        );
    }
}
