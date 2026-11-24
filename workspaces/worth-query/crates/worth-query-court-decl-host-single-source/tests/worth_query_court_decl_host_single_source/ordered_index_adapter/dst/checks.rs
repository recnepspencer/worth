//! Typed query, NotReady refusal, observed Source-row, and same-host budget checks for DST.
use super::super::queries::{FindFocusOverlap, FindTitlePrefix, ReadTasks, TaskRow};
use super::super::schema::FocusWindow;
use super::super::{keys, try_rename_task_with_policy, Host};
use super::model::{title_probe_id, OracleTask, SortedScanOracle};
use super::trace::{PostState, Trace};
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, ExecutionResourceLease, LeaseRequest};
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

pub fn prefix_q(prefix: &str) -> FindTitlePrefix {
    FindTitlePrefix {
        workspace: "main".into(),
        prefix: prefix.to_owned(),
    }
}

pub fn focus_q(lo: i64, hi: i64) -> FindFocusOverlap {
    FindFocusOverlap {
        workspace: "main".into(),
        window: FocusWindow::new(lo, hi + 1),
    }
}

pub fn rows_map(rows: &[TaskRow]) -> Result<BTreeMap<u64, OracleTask>, String> {
    let mut map = BTreeMap::new();
    for r in rows {
        if r.start_min >= r.end_min {
            return Err(format!(
                "invalid stored interval [{}, {}) for row {}",
                r.start_min, r.end_min, r.key
            ));
        }
        map.insert(
            r.key,
            OracleTask {
                title: r.title.clone(),
                lo: r.start_min,
                hi: r.end_min - 1,
            },
        );
    }
    Ok(map)
}

pub fn read_observed_rows(host: &Host) -> Result<BTreeMap<u64, OracleTask>, String> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let req = host.runtime().request(&principal, &scope);
    let rows = req
        .query(ReadTasks {
            workspace: "main".into(),
        })
        .execute()
        .map_err(|e| format!("ReadTasks err: {e:?}"))?;
    rows_map(rows.rows())
}

fn assert_typed_not_ready<R>(
    label: &str,
    result: Result<R, WorthQueryApplicationRequestQueryDenial>,
) -> Result<(), String> {
    match result {
        Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
            if matches!(
                cause.cause(),
                WorthQueryManagedIndexDenialCause::Refused(
                    WorthQueryManagedIndexRefusal::NotReady
                        | WorthQueryManagedIndexRefusal::RootUnavailable,
                )
            ) =>
        {
            Ok(())
        }
        Err(other) => Err(format!(
            "cold {label} query expected typed Refused(NotReady), got {other:?}"
        )),
        Ok(_) => Err(format!("cold {label} query succeeded before rebuild")),
    }
}

pub fn post_state<'a>(
    lifecycle: &'a str,
    rows: &'a BTreeMap<u64, OracleTask>,
    oracle: &SortedScanOracle,
) -> PostState<'a> {
    PostState {
        lifecycle,
        rows,
        watch_active: oracle.watch_active,
        title_watch_pending: oracle.title_watch_pending,
        focus_watch_pending: oracle.focus_watch_pending,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn verify_query_both(
    host: &Host,
    lifecycle: &str,
    tracing: bool,
    prefix: &str,
    lo: i64,
    hi: i64,
    oracle: &SortedScanOracle,
    trace: &mut Trace,
) -> Result<(), String> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let req = host.runtime().request(&principal, &scope);
    let t_res = req.query(prefix_q(prefix)).execute();
    let f_res = req.query(focus_q(lo, hi)).execute();
    let obs = rows_map(
        req.query(ReadTasks {
            workspace: "main".into(),
        })
        .execute()
        .map_err(|e| format!("ReadTasks err: {e:?}"))?
        .rows(),
    )?;
    if obs != oracle.rows {
        return Err(format!("observed rows {obs:?} != oracle {:?}", oracle.rows));
    }
    if lifecycle == "Created" {
        assert_typed_not_ready("title", t_res)?;
        assert_typed_not_ready("focus", f_res)?;
        if tracing {
            trace.record_refuse_not_ready(
                title_probe_id(prefix),
                lo,
                hi,
                &post_state("Created", &obs, oracle),
            );
        }
        return Ok(());
    }
    let got_t = keys(t_res.map_err(|e| format!("title query err: {e:?}"))?.rows());
    let exp_t = oracle.prefix_matches(prefix);
    if got_t != exp_t {
        return Err(format!(
            "title prefix {prefix:?}: got {got_t:?}, oracle {exp_t:?}"
        ));
    }
    let got_f = keys(f_res.map_err(|e| format!("focus query err: {e:?}"))?.rows());
    let exp_f = oracle.focus_matches(lo, hi);
    if got_f != exp_f {
        return Err(format!(
            "focus [{lo},{hi}]: got {got_f:?}, oracle {exp_f:?}"
        ));
    }
    if tracing {
        trace.record_query(
            title_probe_id(prefix),
            &got_t,
            (lo, hi),
            &got_f,
            &post_state("Ready", &obs, oracle),
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn verify_same_host_budget_refusal(
    host: &Host,
    lifecycle: &str,
    tracing: bool,
    idem: u64,
    row: u64,
    attempted: &OracleTask,
    prefix: &str,
    lo: i64,
    hi: i64,
    oracle: &SortedScanOracle,
    trace: &mut Trace,
) -> Result<(), String> {
    let tight_lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(64), Some(1)),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap();
    let refused = try_rename_task_with_policy(
        host,
        row,
        &attempted.title,
        idem,
        ExecutionAllocationPolicy::Execution(&tight_lease),
    );
    let detail = format!("{refused:?}");
    let is_budget_refusal = !matches!(
        refused,
        Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
    ) && (detail.contains("WorkExhausted")
        || detail.contains("WorkCeiling")
        || detail.contains("MemoryExhausted")
        || detail.contains("MemoryLimit"));
    if !is_budget_refusal {
        return Err(format!("expected same-host budget refusal, got {detail}"));
    }
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let req = host.runtime().request(&principal, &scope);
    let obs = rows_map(
        req.query(ReadTasks {
            workspace: "main".into(),
        })
        .execute()
        .map_err(|e| format!("post-refusal ReadTasks err: {e:?}"))?
        .rows(),
    )?;
    if obs != oracle.rows {
        return Err(format!(
            "post-refusal rows {obs:?} != oracle {:?}",
            oracle.rows
        ));
    }
    let got_t = keys(
        req.query(prefix_q(prefix))
            .execute()
            .map_err(|e| format!("post-refusal title query err: {e:?}"))?
            .rows(),
    );
    let got_f = keys(
        req.query(focus_q(lo, hi))
            .execute()
            .map_err(|e| format!("post-refusal focus query err: {e:?}"))?
            .rows(),
    );
    if got_t != oracle.prefix_matches(prefix) || got_f != oracle.focus_matches(lo, hi) {
        return Err("post-refusal index query diverged from oracle".to_owned());
    }
    if tracing {
        trace.record_budget_refusal(
            row,
            attempted,
            title_probe_id(prefix),
            &got_t,
            (lo, hi),
            &got_f,
            &post_state(lifecycle, &obs, oracle),
        );
    }
    Ok(())
}
