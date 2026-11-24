//! Live-subscription open/poll/close observation & projection helpers for the
//! 64-seed Differential Live-Subscription DST.
use std::collections::BTreeMap;

use super::super::access::{InspectBoard, ReadBoard, ReadBoardAccess};
use super::super::queries::{date, GovReadyIntent, GovTodayIntent, TaskRow};
use super::super::schema::Schema;
use super::super::world::{live_limits, Host, FUTURE_DUE_DAYS, TODAY_CUTOFF_DAYS};
use super::reference::{BoardOracle, Kind, ProjectedRow};
use super::trace::Trace;
use super::Coverage;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationLiveOpenRequestDenial, WorthQueryApplicationRequestExt,
    WorthQueryApplicationWholeResultLiveSubscription,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationLiveCloseOutcome, WorthQueryApplicationLiveOpenDenialKind,
    WorthQueryApplicationQueryAccessReceipt, WorthQueryApplicationQueryAdmissionDenialKind,
    WorthQueryApplicationWholeResultLiveOutcome, WorthQueryOperationAuthorizationDenialKind,
};

pub(super) struct LiveSubs<'a> {
    today: Option<WorthQueryApplicationWholeResultLiveSubscription<'a, Schema, GovTodayIntent>>,
    ready: Option<WorthQueryApplicationWholeResultLiveSubscription<'a, Schema, GovReadyIntent>>,
}

impl<'a> LiveSubs<'a> {
    pub(super) fn new() -> Self {
        Self {
            today: None,
            ready: None,
        }
    }
    pub(super) fn is_active(&self, kind: Kind) -> bool {
        match kind {
            Kind::Today => self.today.is_some(),
            Kind::Ready => self.ready.is_some(),
        }
    }
    pub(super) fn close_one_checked(&mut self, kind: Kind) -> Result<&'static str, String> {
        let outcome = match kind {
            Kind::Today => self.today.take().map(|s| s.close()),
            Kind::Ready => self.ready.take().map(|s| s.close()),
        };
        match outcome {
            Some(WorthQueryApplicationLiveCloseOutcome::Completed(_)) => Ok("Completed"),
            other => Err(format!("CloseSub({kind:?}) unexpected outcome: {other:?}")),
        }
    }
    fn drop_terminated(&mut self, kind: Kind) {
        match kind {
            Kind::Today => self.today = None,
            Kind::Ready => self.ready = None,
        }
    }
    pub(super) fn close_all(self) {
        if let Some(sub) = self.today {
            assert!(matches!(
                sub.close(),
                WorthQueryApplicationLiveCloseOutcome::Completed(_)
            ));
        }
        if let Some(sub) = self.ready {
            assert!(matches!(
                sub.close(),
                WorthQueryApplicationLiveCloseOutcome::Completed(_)
            ));
        }
    }
}

pub(super) struct AuthCtx<'a> {
    pub(super) scope: &'a WorthQueryRequestScope,
    pub(super) subscriber: &'a WorthQueryAuthenticatedExternalPrincipal<Schema>,
}

fn project_rows(rows: &[TaskRow]) -> Result<Vec<ProjectedRow>, String> {
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let status = match row.status.as_str() {
            "open" => "open",
            "done" => "done",
            other => return Err(format!("unexpected projected status: {other}")),
        };
        let due = match row.due {
            None => 0,
            Some(d) if d == date(TODAY_CUTOFF_DAYS) => 1,
            Some(d) if d == date(FUTURE_DUE_DAYS) => 2,
            Some(other) => return Err(format!("unexpected projected due date: {other:?}")),
        };
        out.push((row.key, status, due));
    }
    Ok(out)
}

fn is_exact_auth_open_denial(denial: &WorthQueryApplicationLiveOpenRequestDenial) -> bool {
    match denial {
        WorthQueryApplicationLiveOpenRequestDenial::CapabilityAdmission(auth) => {
            matches!(
                auth.kind(),
                WorthQueryOperationAuthorizationDenialKind::Refused(_)
            )
        }
        WorthQueryApplicationLiveOpenRequestDenial::Open(open) => matches!(
            open.kind(),
            WorthQueryApplicationLiveOpenDenialKind::Admission(
                WorthQueryApplicationQueryAdmissionDenialKind::Authorization(_)
            )
        ),
        _ => false,
    }
}

type ObsTuple = (&'static str, &'static str, usize, Vec<ProjectedRow>);

fn handle_sub_open<T>(
    res: Result<T, WorthQueryApplicationLiveOpenRequestDenial>,
    seed: u64,
    step: usize,
    cov: &mut Coverage,
    store: impl FnOnce(T) -> Result<(usize, Vec<ProjectedRow>), String>,
) -> Result<ObsTuple, String> {
    match res {
        Ok(sub) => {
            let (commit_idx, rows) = store(sub)?;
            Ok(("Initial", "Ok", commit_idx, rows))
        }
        Err(denial) if is_exact_auth_open_denial(&denial) => {
            cov.auth_denials += 1;
            Ok(("AuthDenied", "GrantRevoked", 0, Vec::new()))
        }
        Err(other) => Err(format!(
            "seed {seed} step {step}: unexpected subscribe err: {other:?}"
        )),
    }
}

fn record_initial_snapshot(
    rows_raw: &[TaskRow],
    receipt: &WorthQueryApplicationQueryAccessReceipt,
    version_map: &mut BTreeMap<String, usize>,
    history_len: usize,
    seed: u64,
    step: usize,
) -> Result<(usize, Vec<ProjectedRow>), String> {
    let rows = project_rows(rows_raw)?;
    let ver = format!("{:?}", receipt.basis_version());
    if version_map.is_empty() && history_len == 1 {
        version_map.insert(ver.clone(), 1);
    }
    let commit_idx = *version_map
        .get(&ver)
        .ok_or_else(|| format!("seed {seed} step {step}: unmapped initial basis version {ver}"))?;
    Ok((commit_idx, rows))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn observe_subscribe<'a>(
    seed: u64,
    step: usize,
    host: &'a Host,
    auth: &AuthCtx<'_>,
    subs: &mut LiveSubs<'a>,
    oracle: &mut BoardOracle,
    version_map: &mut BTreeMap<String, usize>,
    trace: &mut Trace,
    cov: &mut Coverage,
    kind: Kind,
) -> Result<(), String> {
    let access = ReadBoardAccess {
        project: "p1".into(),
    };
    let req = host.runtime().request(auth.subscriber, auth.scope);
    let history_len = oracle.history.len();
    let (obs_outcome, obs_reason, obs_commit_idx, obs_rows) = match kind {
        Kind::Today => handle_sub_open(
            req.query(GovTodayIntent {
                project: "p1".into(),
                cutoff_days: TODAY_CUTOFF_DAYS,
            })
            .subscribe_whole_result_governed(
                ReadBoard::reference(),
                InspectBoard::reference(),
                access,
                live_limits(),
            ),
            seed,
            step,
            cov,
            |sub| {
                let init = sub.initial().unwrap();
                let res = record_initial_snapshot(
                    init.rows(),
                    init.receipt(),
                    version_map,
                    history_len,
                    seed,
                    step,
                )?;
                subs.today = Some(sub);
                Ok(res)
            },
        )?,
        Kind::Ready => handle_sub_open(
            req.query(GovReadyIntent {
                project: "p1".into(),
            })
            .subscribe_whole_result_governed(
                ReadBoard::reference(),
                InspectBoard::reference(),
                access,
                live_limits(),
            ),
            seed,
            step,
            cov,
            |sub| {
                let init = sub.initial().unwrap();
                let res = record_initial_snapshot(
                    init.rows(),
                    init.receipt(),
                    version_map,
                    history_len,
                    seed,
                    step,
                )?;
                subs.ready = Some(sub);
                Ok(res)
            },
        )?,
    };
    let (exp_outcome, exp_reason, exp_commit_idx, exp_rows) = oracle.expected_subscribe(kind);
    if obs_outcome != exp_outcome
        || obs_reason != exp_reason
        || (exp_outcome == "Initial" && (obs_commit_idx != exp_commit_idx || obs_rows != exp_rows))
    {
        return Err(format!(
            "seed {seed} step {step} Subscribe({kind:?}) mismatch: got ({obs_outcome}, {obs_reason}, {obs_commit_idx}, {obs_rows:?}), expected ({exp_outcome}, {exp_reason}, {exp_commit_idx}, {exp_rows:?})"
        ));
    }
    trace.subscribe(kind, obs_outcome, obs_reason, obs_commit_idx, &obs_rows);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn observe_poll<'a>(
    seed: u64,
    step: usize,
    host: &'a Host,
    auth: &AuthCtx<'_>,
    subs: &mut LiveSubs<'a>,
    oracle: &mut BoardOracle,
    version_map: &BTreeMap<String, usize>,
    trace: &mut Trace,
    cov: &mut Coverage,
    kind: Kind,
) -> Result<(), String> {
    let req = host.runtime().request(auth.subscriber, auth.scope);
    let ((obs_outcome, obs_reason, obs_commit_idx, obs_rows), drop_sub) = match kind {
        Kind::Today => classify_poll(
            subs.today.as_mut().unwrap().next(&req),
            seed,
            step,
            kind,
            version_map,
            cov,
        )?,
        Kind::Ready => classify_poll(
            subs.ready.as_mut().unwrap().next(&req),
            seed,
            step,
            kind,
            version_map,
            cov,
        )?,
    };
    if drop_sub {
        subs.drop_terminated(kind);
    }
    let (exp_outcome, exp_reason, exp_commit_idx, exp_rows) = oracle.expected_poll(kind);
    if obs_outcome != exp_outcome
        || obs_reason != exp_reason
        || (exp_outcome == "Delivered"
            && (obs_commit_idx != exp_commit_idx || obs_rows != exp_rows))
    {
        return Err(format!(
            "seed {seed} step {step} Poll({kind:?}) mismatch: got ({obs_outcome}, {obs_reason}, {obs_commit_idx}, {obs_rows:?}), expected ({exp_outcome}, {exp_reason}, {exp_commit_idx}, {exp_rows:?})"
        ));
    }
    trace.poll(kind, obs_outcome, obs_reason, obs_commit_idx, &obs_rows);
    Ok(())
}

fn classify_poll<Q>(
    res: Result<
        WorthQueryApplicationWholeResultLiveOutcome<Q, TaskRow>,
        worth_query_host::facade::application_entry::WorthQueryApplicationLiveNextDenial,
    >,
    seed: u64,
    step: usize,
    kind: Kind,
    version_map: &BTreeMap<String, usize>,
    cov: &mut Coverage,
) -> Result<(ObsTuple, bool), String> {
    match res.map_err(|e| format!("seed {seed} step {step} Poll({kind:?}) err: {e:?}"))? {
        WorthQueryApplicationWholeResultLiveOutcome::Delivered(u) => {
            cov.deliveries += 1;
            let basis_ver = format!("{:?}", u.snapshot().receipt().basis_version());
            let pub_ver = format!("{:?}", u.product_publication().relational_commit().version_id);
            if basis_ver != pub_ver {
                return Err(format!(
                    "seed {seed} step {step} Poll({kind:?}): pub ver {pub_ver} != receipt ver {basis_ver}"
                ));
            }
            let commit_idx = *version_map.get(&basis_ver).ok_or_else(|| {
                format!("seed {seed} step {step} Poll({kind:?}): unmapped ver {basis_ver}")
            })?;
            let rows = project_rows(u.snapshot().rows())?;
            Ok((("Delivered", "Ok", commit_idx, rows), false))
        }
        WorthQueryApplicationWholeResultLiveOutcome::Pending => {
            cov.pendings += 1;
            Ok((("Pending", "Ok", 0, Vec::new()), false))
        }
        WorthQueryApplicationWholeResultLiveOutcome::AuthorizationDenied(auth) => {
            if !matches!(
                auth.kind(),
                WorthQueryOperationAuthorizationDenialKind::Refused(_)
            ) {
                return Err(format!(
                    "seed {seed} step {step} Poll({kind:?}): expected Refused, got {auth:?}"
                ));
            }
            cov.auth_denials += 1;
            Ok((("AuthDenied", "GrantRevoked", 0, Vec::new()), true))
        }
        other => Err(format!(
            "seed {seed} step {step} Poll({kind:?}) unexpected outcome: {other:?}"
        )),
    }
}
