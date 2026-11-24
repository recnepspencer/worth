//! Live single-seed Directory Home + loopback TCP execution driver for Shared Access DST.

use std::path::PathBuf;

use worth_query_host::facade::{
    shared_access::{RemoteAuthConfig, WorthQueryRemoteClient},
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation,
};
use worth_query_testkit_host::facade::host::checkpoint_capture;

use super::dst::{
    issue_principal_token, make_auth_config, tamper_token, DstCoverage, STEPS_PER_SEED,
};
use super::dst_open_session::{OpenSession, SeedState};
use super::dst_oracle::{
    pick_next_op, DstOp, GrantState, PassivePostState, Rng, SharedAccessOracle,
};
use super::dst_trace::TraceRecorder;
use super::mutation_handlers::SharedProjectProgram;
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
        let valid_tokens = [
            issue_principal_token(&auth_config, &home, 0, seed ^ 0x11),
            issue_principal_token(&auth_config, &home, 1, seed ^ 0x22),
        ];
        let tampered_tokens = [
            tamper_token(&valid_tokens[0]),
            tamper_token(&valid_tokens[1]),
        ];
        Ok(Self {
            seed,
            path,
            execution,
            auth_config,
            oracle: SharedAccessOracle::default(),
            rng: Rng::new(seed),
            trace: TraceRecorder::default(),
            coverage: DstCoverage::default(),
            state: SeedState::new(valid_tokens, tampered_tokens),
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
        let mut close_now = false;
        {
            let server = home
                .serve_remote("127.0.0.1:0".parse().unwrap(), self.auth_config.clone())
                .map_err(|e| format!("seed {}: serve_remote failed: {e:?}", self.seed))?;
            let addr = server.local_addr();
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
                let op = next_op(explicit_ops, &mut self.rng, &self.oracle, self.step_idx);
                self.ops_taken.push(op);
                let grant_before = match op {
                    DstOp::RemoteWatchOpen { principal, scope } => {
                        self.state.grant_states[principal][scope]
                    }
                    _ => GrantState::None,
                };
                let (outcome, value) = match op {
                    DstOp::CloseHome => {
                        close_now = true;
                        ("Ok", 0)
                    }
                    other => session.execute(&mut self.state, other)?,
                };
                let watches = if close_now {
                    [[false; 2]; 2]
                } else {
                    session.observe_watches()
                };
                self.verify_and_record(op, (outcome, value, grant_before), !close_now, watches)?;
            }
            drop(session.watches);
            drop(session.clients);
            session.server.close();
        }
        if close_now {
            home.close(&self.execution)
                .map_err(|e| format!("seed {}: Home close failed: {e:?}", self.seed))?
                .reclaim();
        } else {
            self.open_home = Some(home);
        }
        Ok(())
    }

    fn run_closed_step(&mut self, explicit_ops: Option<&[DstOp]>) -> Result<(), String> {
        let op = next_op(explicit_ops, &mut self.rng, &self.oracle, self.step_idx);
        self.ops_taken.push(op);
        let (outcome, value) = match op {
            DstOp::ReopenHome => self.reopen_home()?,
            DstOp::RemoteRead { .. }
            | DstOp::RemoteWatchOpen { .. }
            | DstOp::RemoteWatchPoll { .. } => ("HomeClosed", 0),
            DstOp::TamperAuthToken { principal } => {
                self.state.active_tokens[principal] = self.state.tampered_tokens[principal].clone();
                ("Ok", 0)
            }
            DstOp::RestoreAuthToken { principal } => {
                self.state.active_tokens[principal] = self.state.valid_tokens[principal].clone();
                ("Ok", 0)
            }
            other => {
                return Err(format!(
                    "seed {}: invalid op while closed: {other:?}",
                    self.seed
                ))
            }
        };
        self.verify_and_record(
            op,
            (outcome, value, GrantState::None),
            self.open_home.is_some(),
            [[false; 2]; 2],
        )
    }

    fn reopen_home(&mut self) -> Result<(&'static str, u64), String> {
        let loc = WorthQueryApplicationHomeLocation::Directory(self.path.clone());
        let reopened = open_shared_home(loc, self.execution, false)
            .map_err(|e| format!("seed {}: reopen failed: {e:?}", self.seed))?;
        self.state.incarnation += 1;
        self.open_home = Some(reopened);
        Ok(("Ok", 0))
    }

    fn verify_and_record(
        &mut self,
        op: DstOp,
        observed: (&'static str, u64, GrantState),
        home_open: bool,
        watches_open: [[bool; 2]; 2],
    ) -> Result<(), String> {
        let (actual_outcome, actual_value, grant_before) = observed;
        let expected = self.oracle.apply(op);
        if actual_outcome != expected.outcome || actual_value != expected.value {
            return Err(format!(
                "seed {} step {} op {op:?}: runtime=({actual_outcome}, {actual_value}) != oracle=({}, {})\nops: {:?}",
                self.seed, self.step_idx, expected.outcome, expected.value, self.ops_taken
            ));
        }
        self.coverage.record(op, actual_outcome, grant_before);
        let post = PassivePostState {
            home_open,
            incarnation: self.state.incarnation,
            project_values: self.state.project_values,
            grants: self.state.grant_states,
            watches_open,
        };
        self.trace.record(op, actual_outcome, actual_value, &post);
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
    oracle: &SharedAccessOracle,
    step: usize,
) -> DstOp {
    explicit.map_or_else(|| pick_next_op(rng, oracle, step), |list| list[step])
}
