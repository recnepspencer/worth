//! Live single-seed Directory Home + loopback TCP execution driver for strictly passive DST.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use worth_query_host::facade::{
    shared_access::{
        RemoteAuthConfig, RemoteClientLimits, SharedAccessDenial, WorthQueryRemoteClient,
    },
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation,
};
use worth_query_testkit_host::facade::host::checkpoint_capture;

use super::dst::{
    issue_principal_token, make_auth_config, tamper_token, DstCoverage, STEPS_PER_SEED,
};
use super::dst_open_session::{OpenSession, SeedState};
use super::dst_oracle::{
    pick_next_op, DeliveredRow, DstOp, GrantState, PassivePostState, PassiveScheduleInput, Rng,
    SharedAccessOracle,
};
use super::dst_trace::TraceRecorder;
use super::mutation_handlers::SharedProjectProgram;
use super::owner_observe::observe_durable_home_state;
use super::program::open_shared_home;
use super::schema::SharedProjectSchema;

pub struct SeedRunOutput {
    pub coverage: DstCoverage,
    pub trace_ndjson: String,
}

struct SeedRunner<'a> {
    seed: u64,
    path: PathBuf,
    execution: worth_execution::ExecutionRequest<'a, 'a>,
    auth_config: RemoteAuthConfig,
    oracle: SharedAccessOracle,
    rng: Rng,
    trace: TraceRecorder,
    coverage: DstCoverage,
    state: SeedState,
    observed: PassiveScheduleInput,
    last_server_addr: Option<SocketAddr>,
    open_home: Option<WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>>,
    ops_taken: Vec<DstOp>,
    step_idx: usize,
}

impl<'a> SeedRunner<'a> {
    fn new(
        seed: u64,
        path: PathBuf,
        execution: worth_execution::ExecutionRequest<'a, 'a>,
        step_count: usize,
    ) -> Result<Self, String> {
        let auth_config = make_auth_config();
        let home = open_shared_home(
            WorthQueryApplicationHomeLocation::Directory(path.clone()),
            execution,
            true,
        )
        .map_err(|e| format!("seed {seed}: fresh open failed: {e:?}"))?;
        let incarnation = home
            .home_incarnation()
            .ok_or_else(|| format!("seed {seed}: missing Directory Home incarnation"))?
            .get();
        let valid_tokens = [
            issue_principal_token(&auth_config, &home, 0, seed ^ 0x11),
            issue_principal_token(&auth_config, &home, 1, seed ^ 0x22),
        ];
        let tampered_tokens = [
            tamper_token(&valid_tokens[0]),
            tamper_token(&valid_tokens[1]),
        ];
        let state = SeedState::new(valid_tokens, tampered_tokens);
        let (values, grants) = observe_durable_home_state(&home, &state.grant_ids)?;
        let observed = PassiveScheduleInput::initial(incarnation, values, grants);
        Ok(Self {
            seed,
            path,
            execution,
            auth_config,
            oracle: SharedAccessOracle::default(),
            rng: Rng::new(seed),
            trace: TraceRecorder::default(),
            coverage: DstCoverage::default(),
            state,
            observed,
            last_server_addr: None,
            open_home: Some(home),
            ops_taken: Vec::with_capacity(step_count),
            step_idx: 0,
        })
    }

    fn run_open_incarnation(
        &mut self,
        explicit_ops: Option<&[DstOp]>,
        step_count: usize,
    ) -> Result<(), String> {
        let Some(home) = self.open_home.take() else {
            return Ok(());
        };
        let real_incarnation = home
            .home_incarnation()
            .ok_or_else(|| format!("seed {}: missing home incarnation", self.seed))?
            .get();
        let mut close_now = false;
        {
            let server = home
                .serve_remote("127.0.0.1:0".parse().unwrap(), self.auth_config.clone())
                .map_err(|e| format!("seed {}: serve_remote failed: {e:?}", self.seed))?;
            let addr = server.local_addr();
            self.last_server_addr = Some(addr);
            let clients = [
                WorthQueryRemoteClient::connect(addr, self.state.active_tokens[0].clone()).unwrap(),
                WorthQueryRemoteClient::connect(addr, self.state.active_tokens[1].clone()).unwrap(),
            ];
            let mut session = OpenSession {
                home: &home,
                server,
                clients,
                watches: Default::default(),
            };
            while self.step_idx < step_count && !close_now {
                close_now =
                    self.run_open_step(explicit_ops, &home, real_incarnation, &mut session)?;
            }
            drop(session.watches);
            drop(session.clients);
            session.server.close();
        }
        if close_now {
            self.state.watch_last_delivered = [[0; 2]; 2];
            home.close(&self.execution)
                .map_err(|e| format!("seed {}: Home close failed: {e:?}", self.seed))?
                .reclaim();
        } else {
            self.open_home = Some(home);
        }
        Ok(())
    }

    fn run_open_step(
        &mut self,
        explicit_ops: Option<&[DstOp]>,
        home: &WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>,
        real_incarnation: u64,
        session: &mut OpenSession<'_>,
    ) -> Result<bool, String> {
        let op = next_op(explicit_ops, &mut self.rng, &self.observed, self.step_idx);
        self.ops_taken.push(op);
        let grant_before = match op {
            DstOp::RemoteWatchOpen { principal, scope } => {
                self.observed.durable_grants[principal][scope]
            }
            _ => GrantState::None,
        };
        let (close_now, outcome, rows) = match op {
            DstOp::CloseHome => (true, "Ok", Vec::new()),
            other => {
                let (out, r) =
                    session.execute(&mut self.state, other, self.observed.project_values)?;
                (false, out, r)
            }
        };
        let (values, grants) = observe_durable_home_state(home, &self.state.grant_ids)?;
        let watches = if close_now {
            [[false; 2]; 2]
        } else {
            session.observe_server_watches()
        };
        let post = PassivePostState {
            home_open: !close_now,
            incarnation: real_incarnation,
            project_values: values,
            grants,
            watches_open: watches,
        };
        self.verify_and_record(op, (outcome, rows, grant_before), post)?;
        Ok(close_now)
    }

    fn run_closed_step(&mut self, explicit_ops: Option<&[DstOp]>) -> Result<(), String> {
        let op = next_op(explicit_ops, &mut self.rng, &self.observed, self.step_idx);
        self.ops_taken.push(op);
        let (outcome, rows) = match op {
            DstOp::ReopenHome => self.reopen_home()?,
            DstOp::RemoteRead { principal, .. }
            | DstOp::RemoteWatchOpen { principal, .. }
            | DstOp::RemoteWatchPoll { principal, .. } => self.probe_closed_transport(principal)?,
            DstOp::TamperAuthToken { principal } => {
                self.state.token_tampered[principal] = true;
                self.state.active_tokens[principal] = self.state.tampered_tokens[principal].clone();
                ("Ok", Vec::new())
            }
            DstOp::RestoreAuthToken { principal } => {
                self.state.token_tampered[principal] = false;
                self.state.active_tokens[principal] = self.state.valid_tokens[principal].clone();
                ("Ok", Vec::new())
            }
            other => {
                return Err(format!(
                    "seed {}: invalid op while closed: {other:?}",
                    self.seed
                ))
            }
        };
        let post = if let Some(reopened) = self.open_home.as_ref() {
            let real_inc = reopened
                .home_incarnation()
                .ok_or_else(|| format!("seed {}: reopened home missing incarnation", self.seed))?
                .get();
            let (values, grants) = observe_durable_home_state(reopened, &self.state.grant_ids)?;
            PassivePostState {
                home_open: true,
                incarnation: real_inc,
                project_values: values,
                grants,
                watches_open: [[false; 2]; 2],
            }
        } else {
            PassivePostState {
                home_open: false,
                incarnation: self.observed.incarnation,
                project_values: self.observed.project_values,
                grants: self.observed.durable_grants,
                watches_open: [[false; 2]; 2],
            }
        };
        self.verify_and_record(op, (outcome, rows, GrantState::None), post)
    }

    /// Executes a REAL loopback TCP connect against the closed server endpoint — NEVER synthetic!
    fn probe_closed_transport(
        &self,
        principal: usize,
    ) -> Result<(&'static str, Vec<DeliveredRow>), String> {
        let addr = self
            .last_server_addr
            .ok_or_else(|| format!("seed {}: missing closed server addr", self.seed))?;
        let short = RemoteClientLimits::default()
            .with_io_timeout(Duration::from_millis(500))
            .unwrap();
        match WorthQueryRemoteClient::connect_with_limits(
            addr,
            self.state.active_tokens[principal].clone(),
            short,
        ) {
            Err(SharedAccessDenial::TransportIo(_)) => Ok(("HomeClosed", Vec::new())),
            Err(other) => Err(format!(
                "seed {}: closed-home transport returned unexpected denial: {other:?}",
                self.seed
            )),
            Ok(_) => Err(format!(
                "seed {}: closed-home transport unexpectedly accepted TCP connection",
                self.seed
            )),
        }
    }

    fn reopen_home(&mut self) -> Result<(&'static str, Vec<DeliveredRow>), String> {
        let loc = WorthQueryApplicationHomeLocation::Directory(self.path.clone());
        let reopened = open_shared_home(loc, self.execution, false)
            .map_err(|e| format!("seed {}: reopen failed: {e:?}", self.seed))?;
        self.open_home = Some(reopened);
        Ok(("Ok", Vec::new()))
    }

    fn verify_and_record(
        &mut self,
        op: DstOp,
        observed_step: (&'static str, Vec<DeliveredRow>, GrantState),
        post: PassivePostState,
    ) -> Result<(), String> {
        let (actual_outcome, actual_rows, grant_before) = observed_step;
        let expected = self.oracle.apply(op);
        if actual_outcome != expected.outcome || actual_rows != expected.rows {
            return Err(format!(
                "seed {} step {} op {op:?}: runtime=({actual_outcome}, {actual_rows:?}) != oracle=({}, {:?})\nops: {:?}",
                self.seed, self.step_idx, expected.outcome, expected.rows, self.ops_taken
            ));
        }
        self.coverage.record(op, actual_outcome, grant_before);
        self.trace.record(op, actual_outcome, &actual_rows, &post);
        self.observed = PassiveScheduleInput {
            home_open: post.home_open,
            incarnation: post.incarnation,
            project_values: post.project_values,
            durable_grants: post.grants,
            server_watches_open: post.watches_open,
            watch_last_delivered: self.state.watch_last_delivered,
            token_tampered: self.state.token_tampered,
        };
        self.step_idx += 1;
        Ok(())
    }
}

/// Runs one deterministic seed (or explicit op sequence) against a real Directory Home
/// and real loopback TCP `WorthQueryRemoteServer` + `WorthQueryRemoteClient`s.
pub fn run_seed_ops(seed: u64, explicit_ops: Option<&[DstOp]>) -> Result<SeedRunOutput, String> {
    let path = worth_query_testkit_host::facade::host::application_home::location()
        .with_extension(format!("shared-dst-{seed}"));
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let step_count = explicit_ops.map_or(STEPS_PER_SEED, |ops| ops.len());
    let mut runner = SeedRunner::new(seed, path.clone(), execution, step_count)?;
    while runner.step_idx < step_count {
        if runner.open_home.is_some() {
            runner.run_open_incarnation(explicit_ops, step_count)?;
        } else {
            runner.run_closed_step(explicit_ops)?;
        }
    }
    if let Some(home_left) = runner.open_home {
        home_left
            .close(&execution)
            .map_err(|e| format!("seed {seed}: final Home close failed: {e:?}"))?
            .reclaim();
    }
    let _ = std::fs::remove_dir_all(&path);
    Ok(SeedRunOutput {
        coverage: runner.coverage,
        trace_ndjson: runner.trace.to_ndjson(),
    })
}

fn next_op(
    explicit: Option<&[DstOp]>,
    rng: &mut Rng,
    observed: &PassiveScheduleInput,
    step: usize,
) -> DstOp {
    explicit.map_or_else(|| pick_next_op(rng, observed, step), |list| list[step])
}
