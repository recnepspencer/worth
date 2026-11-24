//! Per-epoch randomized mutation and subscription schedule driver for the
//! 64-seed Differential Live-Subscription DST.
use std::collections::BTreeMap;

use super::super::mutations::MutateLiveBoard;
use super::super::world::{run_mutate_as, Host, FUTURE_DUE_DAYS, TODAY_CUTOFF_DAYS};
use super::observe::{observe_poll, observe_subscribe, AuthCtx, LiveSubs};
use super::reference::{BoardOracle, Kind};
use super::trace::Trace;
use super::{Coverage, Rng};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, authenticate_reviewer, request_scope,
};

fn due_days_for_bucket(bucket: u8) -> i64 {
    match bucket {
        0 => -1,
        1 => TODAY_CUTOFF_DAYS,
        _ => FUTURE_DUE_DAYS,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_epoch(
    seed: u64,
    host: &Host,
    oracle: &mut BoardOracle,
    version_map: &mut BTreeMap<String, usize>,
    rng: &mut Rng,
    trace: &mut Trace,
    cov: &mut Coverage,
    idem: &mut u64,
    steps: usize,
) -> Result<(), String> {
    let scope = request_scope();
    let subscriber = authenticate_operator(host.installed_schema(), &scope);
    let admin = authenticate_reviewer(host.installed_schema(), &scope);
    let auth = AuthCtx {
        scope: &scope,
        subscriber: &subscriber,
    };
    let mut mutate = |project: &str, op: u8, task: u64, other: u64, due_days: i64, text: &str| {
        let receipt = run_mutate_as(
            host,
            &admin,
            &scope,
            MutateLiveBoard {
                project: project.to_owned(),
                op,
                task,
                other,
                due_days,
                text: text.to_owned(),
            },
            *idem,
        );
        *idem += 1;
        format!("{:?}", receipt.commit_reference().version_id)
    };
    let mut subs = LiveSubs::new();
    for kind in [Kind::Today, Kind::Ready] {
        observe_subscribe(
            seed, 0, host, &auth, &mut subs, oracle, version_map, trace, cov, kind,
        )?;
    }
    for step in 1..=steps {
        let choice = rng.pick(12);
        if choice < 2 {
            let task = (rng.pick(6) as u64) + 1;
            let proj = oracle.project[&task];
            let next = oracle.toggle_status(task);
            let ver = mutate(proj, 0, task, 0, 0, next);
            version_map.insert(ver, oracle.history.len());
            trace.toggle_status(task);
            cov.toggles += 1;
            if proj == "p2" {
                cov.p2_edits += 1;
            }
        } else if choice < 5 {
            let task = (rng.pick(6) as u64) + 1;
            let bucket = rng.pick(3) as u8;
            if oracle.set_due(task, bucket) {
                let proj = oracle.project[&task];
                let ver = mutate(proj, 1, task, 0, due_days_for_bucket(bucket), "");
                version_map.insert(ver, oracle.history.len());
                trace.set_due(task, bucket);
                cov.due_edits += 1;
                if proj == "p2" {
                    cov.p2_edits += 1;
                }
            }
        } else if choice < 7 {
            let from = (rng.pick(6) as u64) + 1;
            let to = (rng.pick(6) as u64) + 1;
            if oracle.add_edge(from, to) {
                let ver = mutate(oracle.project[&from], 2, from, to, 0, "");
                version_map.insert(ver, oracle.history.len());
                trace.add_edge(from, to);
                cov.edge_adds += 1;
            }
        } else if choice < 9 && !oracle.current().edges.is_empty() {
            let idx = rng.pick(oracle.current().edges.len());
            let (from, to) = *oracle.current().edges.iter().nth(idx).unwrap();
            oracle.remove_edge(from, to);
            let ver = mutate(oracle.project[&from], 3, from, to, 0, "");
            version_map.insert(ver, oracle.history.len());
            trace.remove_edge(from, to);
            cov.edge_removes += 1;
        } else if choice == 9 {
            let (op_code, granted) = if oracle.grant { (2, false) } else { (1, true) };
            oracle.grant = granted;
            let ver = mutate("p1", 6, 0, op_code, 0, "grant-p1");
            version_map.insert(ver, oracle.history.len());
            if granted {
                trace.restore_grant();
                cov.grant_restores += 1;
            } else {
                trace.revoke_grant();
                cov.grant_revokes += 1;
            }
        } else if choice == 10 {
            let kind = if rng.pick(2) == 0 {
                Kind::Today
            } else {
                Kind::Ready
            };
            if subs.is_active(kind) {
                let close_outcome = subs.close_one_checked(kind)?;
                oracle.close_sub(kind);
                trace.close_sub(kind, close_outcome);
            } else {
                observe_subscribe(
                    seed, step, host, &auth, &mut subs, oracle, version_map, trace, cov, kind,
                )?;
            }
        }
        for kind in [Kind::Today, Kind::Ready] {
            if subs.is_active(kind) {
                observe_poll(
                    seed, step, host, &auth, &mut subs, oracle, version_map, trace, cov, kind,
                )?;
            } else if rng.pick(2) == 0 {
                observe_subscribe(
                    seed, step, host, &auth, &mut subs, oracle, version_map, trace, cov, kind,
                )?;
            }
        }
    }
    subs.close_all();
    Ok(())
}
