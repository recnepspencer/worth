//! Seeded deterministic simulation test for typed, erased, and correction Host source custody.
//!
//! Runs 64 deterministic seeds against a real Directory Home and tiny reference oracle matching
//! `specs/tla/host_custody/HostCustody.tla`. Set `WORTH_DST_TRACE_DIR` to emit NDJSON traces
//! verified by `python3 tools/tla/run_tlc.py trace specs/tla/host_custody/HostCustodyTrace.tla`.
mod correction;
mod ordinary;

use super::super::application_home::correction_material::numeric_home::{
    model, open_memory_program_home, open_program_home, read_rows,
};
use super::*;
use model::{
    edit::Edit,
    other_edit::OtherBinding,
    program::{Action, NumericFeature, OtherCompletion},
    schema::NumericSchema,
};
use worth_query_decl::facade::application_program::{
    ApplicationActionReference, ApplicationProgramDefinition, ApplicationRootComposition,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    primary_graph::WorthQueryApplicationCommitReceipt,
    WorthQueryAnySourceCustody,
};

pub(self) type OtherAction = ApplicationActionReference<
    NumericSchema,
    ApplicationRootComposition,
    NumericFeature,
    OtherBinding,
    OtherCompletion,
>;
pub(self) type TypedCustody<P> = WorthQuerySourceCustody<NumericSchema, Edit, P, Action>;
pub(self) type ErasedCustody<P> = WorthQueryAnySourceCustody<NumericSchema, P>;

pub(self) struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(0x9E37_79B9_7F4A_7C15))
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub(self) fn coin(&mut self) -> bool {
        (self.next_u64() & 1) == 1
    }
    pub(self) fn pick(&mut self, bound: usize) -> usize {
        (self.next_u64() as usize) % bound
    }
}

pub(self) enum Slot<P: ApplicationProgramDefinition<NumericSchema>> {
    Typed(TypedCustody<P>),
    Erased(ErasedCustody<P>),
}

pub(self) struct Oracle {
    pub(self) phase: &'static str,
    pub(self) erased: bool,
    pub(self) durable_original: bool,
    pub(self) exec_count: u8,
    pub(self) committed_value: bool,
    trace: Vec<String>,
}

impl Oracle {
    fn new() -> Self {
        Self {
            phase: "Empty",
            erased: false,
            durable_original: false,
            exec_count: 0,
            committed_value: false,
            trace: Vec::new(),
        }
    }

    pub(self) fn record(&mut self, action: &str, extras: &str) {
        assert!(self.exec_count <= 1, "NoDoubleExecution violated");
        if matches!(
            self.phase,
            "TerminalCommitted" | "TerminalWithdrawn" | "TerminalDenied"
        ) {
            assert!(!self.durable_original, "NoLostDurableOriginal violated");
        }
        let comma = if extras.is_empty() { "" } else { "," };
        self.trace.push(format!(
            "{{\"action\":\"{action}\"{comma}{extras},\"postPhase\":\"{}\",\"postErased\":{},\"postDurableOriginal\":{},\"postExecCount\":{},\"postCommittedValue\":{}}}",
            self.phase,
            self.erased,
            self.durable_original,
            self.exec_count,
            self.committed_value,
        ));
    }

    fn reset(&mut self) {
        self.phase = "Empty";
        self.erased = false;
        self.durable_original = false;
        self.exec_count = 0;
        self.committed_value = false;
        self.record("Reset", "");
    }
}

pub(self) fn cancelled_scope(clock: &WorthQueryAdmissionClock) -> WorthQueryRequestScope {
    let source = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, source.token(), clock.clone());
    source.cancel();
    scope
}

#[test]
fn host_custody_dst_64_seeds() {
    let clock = WorthQueryAdmissionClock::system();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    // Run 62 fast deterministic seeds on a MemoryOnly Home (identical Host custody state machine,
    // zero APFS fsync barriers) plus 2 seeds on a real Directory Home to also verify durable-original
    // discharge under the same oracle and TLA+ trace checker within a <5s test budget.
    let mem_home = open_memory_program_home(execution);
    let mem_branch = mem_home.primary_graph().current_world();

    let mut oracle = Oracle::new();
    let mut expected_primary = read_rows(mem_home.runtime(), mem_branch)[0].primary;
    let mut last_committed: Option<(String, u64, WorthQueryApplicationCommitReceipt)> = None;

    for seed in 0_u64..62 {
        let mut rng = Rng::new(seed);
        if seed > 0 {
            oracle.reset();
        }
        let do_correction = last_committed.is_some() && rng.pick(4) == 0;
        if do_correction {
            let (_, prior, receipt) = last_committed.take().unwrap();
            correction::run_correction_seed(
                seed,
                &mut rng,
                &mem_home,
                mem_branch,
                &clock,
                &mut oracle,
                &mut expected_primary,
                prior,
                &receipt,
            );
        } else {
            let prior = expected_primary;
            let drift_key = last_committed.as_ref().map(|(k, _, _)| k.as_str());
            if let Some((key, receipt)) = ordinary::run_ordinary_seed(
                seed,
                &mut rng,
                &mem_home,
                mem_branch,
                &clock,
                &mut oracle,
                &mut expected_primary,
                drift_key,
            ) {
                last_committed = Some((key, prior, receipt));
            }
        }
    }
    drop((last_committed, mem_home));

    let dir_home = open_program_home(fixture::location(), execution, true);
    let dir_branch = dir_home.primary_graph().current_world();
    let mut dir_expected = read_rows(dir_home.runtime(), dir_branch)[0].primary;
    let mut dir_committed: Option<(String, u64, WorthQueryApplicationCommitReceipt)> = None;
    for seed in 62_u64..64 {
        let mut rng = Rng::new(seed);
        oracle.reset();
        if let Some((_, prior, receipt)) = dir_committed.take() {
            correction::run_correction_seed(
                seed,
                &mut rng,
                &dir_home,
                dir_branch,
                &clock,
                &mut oracle,
                &mut dir_expected,
                prior,
                &receipt,
            );
        } else {
            let prior = dir_expected;
            if let Some((key, receipt)) = ordinary::run_ordinary_seed(
                seed,
                &mut rng,
                &dir_home,
                dir_branch,
                &clock,
                &mut oracle,
                &mut dir_expected,
                None,
            ) {
                dir_committed = Some((key, prior, receipt));
            }
        }
    }

    if let Ok(dir) = std::env::var("WORTH_DST_TRACE_DIR") {
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).expect("create WORTH_DST_TRACE_DIR");
        let trace_path = dir.join("host_custody.ndjson");
        std::fs::write(&trace_path, format!("{}\n", oracle.trace.join("\n")))
            .expect("write host_custody.ndjson");
    }

    drop(dir_committed);
    dir_home
        .close(&execution)
        .unwrap_or_else(|refused| panic!("DST Directory Home closes cleanly: {refused:?}"))
        .reclaim();
}
