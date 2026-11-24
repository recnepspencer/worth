//! Per-epoch execution and live-subscription observation driver for the
//! 64-seed Differential Live-Subscription DST.
use super::super::access::{InspectBoard, ReadBoard, ReadBoardAccess};
use super::super::mutations::MutateLiveBoard;
use super::super::queries::{GovReadyIntent, GovTodayIntent, TaskRow};
use super::super::schema::Schema;
use super::super::world::{live_limits, run_mutate_as, Host, FUTURE_DUE_DAYS, TODAY_CUTOFF_DAYS};
use super::reference::{BoardOracle, Kind};
use super::trace::Trace;
use super::{Coverage, Rng};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationLiveOpenRequestDenial, WorthQueryApplicationRequestExt,
    WorthQueryApplicationWholeResultLiveSubscription,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationLiveCloseOutcome, WorthQueryApplicationLiveOpenDenialKind,
    WorthQueryApplicationQueryAdmissionDenialKind, WorthQueryApplicationWholeResultLiveOutcome,
    WorthQueryOperationAuthorizationDenialKind,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, authenticate_reviewer, request_scope,
};

struct LiveSubs<'a> {
    today: Option<WorthQueryApplicationWholeResultLiveSubscription<'a, Schema, GovTodayIntent>>,
    ready: Option<WorthQueryApplicationWholeResultLiveSubscription<'a, Schema, GovReadyIntent>>,
}

impl<'a> LiveSubs<'a> {
    fn new() -> Self {
        Self {
            today: None,
            ready: None,
        }
    }
    fn is_active(&self, kind: Kind) -> bool {
        match kind {
            Kind::Today => self.today.is_some(),
            Kind::Ready => self.ready.is_some(),
        }
    }
    fn close_one(&mut self, kind: Kind) {
        match kind {
            Kind::Today => {
                if let Some(s) = self.today.take() {
                    let _ = s.close();
                }
            }
            Kind::Ready => {
                if let Some(s) = self.ready.take() {
                    let _ = s.close();
                }
            }
        }
    }
    fn close_all(self) {
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

fn due_days_for_bucket(bucket: u8) -> i64 {
    match bucket {
        0 => -1,
        1 => TODAY_CUTOFF_DAYS,
        _ => FUTURE_DUE_DAYS,
    }
}

struct AuthCtx<'a> {
    scope: &'a WorthQueryRequestScope,
    subscriber: &'a WorthQueryAuthenticatedExternalPrincipal<Schema>,
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

fn handle_sub_open<T>(
    res: Result<T, WorthQueryApplicationLiveOpenRequestDenial>,
    seed: u64,
    step: usize,
    cov: &mut Coverage,
    store: impl FnOnce(T) -> Vec<u64>,
) -> Result<(&'static str, Vec<u64>), String> {
    match res {
        Ok(sub) => Ok(("Initial", store(sub))),
        Err(denial) if is_exact_auth_open_denial(&denial) => {
            cov.auth_denials += 1;
            Ok(("AuthDenied", Vec::new()))
        }
        Err(other) => Err(format!(
            "seed {seed} step {step}: unexpected subscribe err: {other:?}"
        )),
    }
}

fn observe_subscribe<'a>(
    seed: u64,
    step: usize,
    host: &'a Host,
    auth: &AuthCtx<'_>,
    subs: &mut LiveSubs<'a>,
    oracle: &mut BoardOracle,
    trace: &mut Trace,
    cov: &mut Coverage,
    kind: Kind,
) -> Result<(), String> {
    let access = ReadBoardAccess {
        project: "p1".into(),
    };
    let req = host.runtime().request(auth.subscriber, auth.scope);
    let (obs_outcome, obs_rows) = match kind {
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
                let rows = sub
                    .initial()
                    .unwrap()
                    .rows()
                    .iter()
                    .map(|r| r.key)
                    .collect();
                subs.today = Some(sub);
                rows
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
                let rows = sub
                    .initial()
                    .unwrap()
                    .rows()
                    .iter()
                    .map(|r| r.key)
                    .collect();
                subs.ready = Some(sub);
                rows
            },
        )?,
    };
    let (exp_outcome, exp_rows) = oracle.expected_subscribe(kind);
    if obs_outcome != exp_outcome || (exp_outcome == "Initial" && obs_rows != exp_rows) {
        return Err(format!(
            "seed {seed} step {step} Subscribe({kind:?}) mismatch: got ({obs_outcome}, {obs_rows:?}), expected ({exp_outcome}, {exp_rows:?})"
        ));
    }
    trace.subscribe(kind, obs_outcome, &obs_rows);
    Ok(())
}

fn observe_poll<'a>(
    seed: u64,
    step: usize,
    host: &'a Host,
    auth: &AuthCtx<'_>,
    subs: &mut LiveSubs<'a>,
    oracle: &mut BoardOracle,
    trace: &mut Trace,
    cov: &mut Coverage,
    kind: Kind,
) -> Result<(), String> {
    let req = host.runtime().request(auth.subscriber, auth.scope);
    let (obs_outcome, obs_rows, drop_sub) = match kind {
        Kind::Today => classify_poll(
            subs.today.as_mut().unwrap().next(&req),
            seed,
            step,
            kind,
            cov,
        )?,
        Kind::Ready => classify_poll(
            subs.ready.as_mut().unwrap().next(&req),
            seed,
            step,
            kind,
            cov,
        )?,
    };
    if drop_sub {
        subs.close_one(kind);
    }
    let (exp_outcome, exp_rows) = oracle.expected_poll(kind);
    if obs_outcome != exp_outcome || (exp_outcome == "Delivered" && obs_rows != exp_rows) {
        return Err(format!(
            "seed {seed} step {step} Poll({kind:?}) mismatch: got ({obs_outcome}, {obs_rows:?}), expected ({exp_outcome}, {exp_rows:?})"
        ));
    }
    trace.poll(kind, obs_outcome, &obs_rows);
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
    cov: &mut Coverage,
) -> Result<(&'static str, Vec<u64>, bool), String> {
    match res.map_err(|e| format!("seed {seed} step {step} Poll({kind:?}) err: {e:?}"))? {
        WorthQueryApplicationWholeResultLiveOutcome::Delivered(u) => {
            cov.deliveries += 1;
            let rows = u.snapshot().rows().iter().map(|r| r.key).collect();
            Ok(("Delivered", rows, false))
        }
        WorthQueryApplicationWholeResultLiveOutcome::Pending => {
            cov.pendings += 1;
            Ok(("Pending", Vec::new(), false))
        }
        WorthQueryApplicationWholeResultLiveOutcome::AuthorizationDenied(_) => {
            cov.auth_denials += 1;
            Ok(("AuthDenied", Vec::new(), true))
        }
        other => Err(format!(
            "seed {seed} step {step} Poll({kind:?}) unexpected outcome: {other:?}"
        )),
    }
}

pub fn run_epoch(
    seed: u64,
    host: &Host,
    oracle: &mut BoardOracle,
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
        run_mutate_as(
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
    };
    let mut subs = LiveSubs::new();
    for kind in [Kind::Today, Kind::Ready] {
        observe_subscribe(seed, 0, host, &auth, &mut subs, oracle, trace, cov, kind)?;
    }
    for step in 1..=steps {
        let choice = rng.pick(12);
        if choice < 2 {
            let task = (rng.pick(6) as u64) + 1;
            let proj = oracle.project[&task];
            let next = oracle.toggle_status(task);
            mutate(proj, 0, task, 0, 0, next);
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
                mutate(proj, 1, task, 0, due_days_for_bucket(bucket), "");
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
                mutate(oracle.project[&from], 2, from, to, 0, "");
                trace.add_edge(from, to);
                cov.edge_adds += 1;
            }
        } else if choice < 9 && !oracle.current().edges.is_empty() {
            let idx = rng.pick(oracle.current().edges.len());
            let (from, to) = *oracle.current().edges.iter().nth(idx).unwrap();
            oracle.remove_edge(from, to);
            mutate(oracle.project[&from], 3, from, to, 0, "");
            trace.remove_edge(from, to);
            cov.edge_removes += 1;
        } else if choice == 9 {
            if oracle.grant {
                oracle.grant = false;
                mutate("p1", 6, 0, 2, 0, "grant-p1");
                trace.revoke_grant();
                cov.grant_revokes += 1;
            } else {
                oracle.grant = true;
                mutate("p1", 6, 0, 1, 0, "grant-p1");
                trace.restore_grant();
                cov.grant_restores += 1;
            }
        } else if choice == 10 {
            let kind = if rng.pick(2) == 0 {
                Kind::Today
            } else {
                Kind::Ready
            };
            if subs.is_active(kind) {
                subs.close_one(kind);
                oracle.close_sub(kind);
                trace.close_sub(kind);
            } else {
                observe_subscribe(seed, step, host, &auth, &mut subs, oracle, trace, cov, kind)?;
            }
        }

        for kind in [Kind::Today, Kind::Ready] {
            if subs.is_active(kind) {
                observe_poll(seed, step, host, &auth, &mut subs, oracle, trace, cov, kind)?;
            } else if rng.pick(2) == 0 {
                observe_subscribe(seed, step, host, &auth, &mut subs, oracle, trace, cov, kind)?;
            }
        }
    }
    subs.close_all();
    Ok(())
}
