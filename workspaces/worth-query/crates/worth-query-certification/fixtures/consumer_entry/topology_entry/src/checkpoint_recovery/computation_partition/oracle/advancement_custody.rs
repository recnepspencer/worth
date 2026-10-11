//! The real caller loop refreshes a Ready whose producer changed a gathered fact.
use super::super::region_output::{arm_own_write, OwnWrite, OwnWriteRead};
use super::*;
use worth_query_host::facade::primary_graph::{
    advancement_requests_on_this_thread_for_test as reports,
    bound_advancement_requests_on_this_thread_for_test as bound,
    caller_pass_reports_on_this_thread_for_test as passes,
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement, WorthQueryOutputDemandRecoveryPosture,
};
struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
        arm_own_write(None);
    }
}
// Each producer independently declares a million-unit entry checkpoint.
// The request funds both producers and declares half an entry checkpoint
// for all delivery. One fresh pass fits; two deliveries must share that allowance.
const RUNS: usize = 1;
const ENTRY_WORK: usize = 1_000_000;
const EVEN_WEIGHT_WORK: u64 = 1;
const PRODUCER_WORK: usize = ENTRY_WORK + WORK_BESIDE_ENTRIES;
const REQUIRED_PRODUCER_PASSES: u64 = 2;
const DELIVERY_WORK_ALLOWANCE: u64 = ENTRY_WORK as u64 / 2;
const CEILING: u64 = ENTRY_WORK as u64 * REQUIRED_PRODUCER_PASSES + DELIVERY_WORK_ALLOWANCE;

#[test]
fn caller_refresh_retry_spends_what_the_first_attempt_left() {
    let _guard = checkpoint_recovery_test_guard();
    let _restore = Restore(place(Placement::Serial), bound(None));
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        place(placement);
        run();
    }
}

#[path = "advancement_custody/post_producer_delivery.rs"]
mod post_producer_delivery;

fn run() {
    use worth_query_host::facade::{
        application_contribution::{
            WorthQueryAdvancementDenial as Denial,
            WorthQueryManagedComputationResourceDenial as Resource,
        },
        application_entry::WorthQueryApplicationOutputDemandDenial as OutputDenial,
        primary_graph::signal_request_charges_on_this_thread_for_test as signal_charges,
    };
    bound(None);
    let application = installation::install_configured::<false, PRODUCER_WORK, RUNS>(
        None,
        Default::default(),
        |graph| {
            facts::seed_set(graph, "even", -0.0);
            seed_entry(
                graph,
                &["even"],
                0,
                RegionEntry {
                    id: 0,
                    region: 0,
                    value: 1.0,
                    work: ENTRY_WORK,
                    fault: None,
                },
            );
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut interest = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<OracleProgram<false, PRODUCER_WORK, RUNS>, RegionConnection>(
            &application,
        )
        .unwrap();
    arm_own_write(Some(OwnWrite {
        number: 0,
        bits: 2.5_f64.to_bits(),
        read: OwnWriteRead::Blind,
    }));
    let policy = support::CHECKPOINT_EXECUTION_POLICY.budget();
    bound(Some(worth_foundational::ExecutionBudget::new(
        NonZeroUsize::MIN,
        policy.charged_memory_bytes(),
        CEILING,
    )));
    reports();
    passes();
    signal_charges();
    room().clear();
    kernel_charges();
    let outcome = interest.settle(&request);
    let requests = reports();
    let attempts = passes();
    let charged = requests[0].as_ref().unwrap().charged_work();
    assert!(charged <= CEILING);
    let diagnostics = {
        let runs = room();
        format!(
            "runs={}; first={:?}; last={:?}",
            runs.len(),
            runs.first().map(|run| &run.outcome),
            runs.last().map(|run| &run.outcome)
        )
    };
    let refusal = outcome.err().unwrap_or_else(|| {
        panic!(
            "the retry cannot reset the spent request: \
                 charged={charged}; passes={attempts:?}; last={:?}",
            room().last().map(|run| &run.outcome)
        )
    });
    let OutputDenial::Demand(denial) = refusal else {
        panic!("the caller must carry its execution stop");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::ExecutionRequest(Denial::Resource(
            Resource::WorkExhausted
        )),
        "charged={charged}; attempts={attempts:?}; diagnostics={diagnostics}"
    );
    assert_eq!(attempts.len(), 3, "charged first pass, refresh, then retry");
    let kernel_work = kernel_charges();
    assert_eq!(
        kernel_work[0],
        ENTRY_WORK as u64 + EVEN_WEIGHT_WORK,
        "the first producer performs its declared entry and weight checkpoints"
    );
    // One declared producer attempt fits alone; the refusal requires carried spend.
    assert!(kernel_work[0] <= CEILING);
    assert_eq!(
        attempts[1].1, attempts[0].1,
        "Ready refresh adds no Signal work"
    );
    assert_eq!(
        requests.len(),
        1,
        "settlement and its caller retry share one request"
    );
    // A retained terminal stop is returned before any owner work.
    let producer_attempts = room().len();
    reports();
    passes();
    let outcome = interest.settle(&request);
    let requests = reports();
    let Err(OutputDenial::Demand(retained)) = outcome else {
        panic!("the same handle must return its retained demand denial");
    };
    assert_eq!(retained, denial, "the exact terminal denial is retained");
    assert_eq!(
        retained.recovery_posture(),
        WorthQueryOutputDemandRecoveryPosture::Terminal
    );
    assert_eq!(requests.len(), 1, "the fresh settlement opens exactly once");
    assert_eq!(passes().len(), 1, "Initial returns the retained stop");
    assert_eq!(room().len(), producer_attempts, "no producer runs again");
    assert!(kernel_charges().is_empty(), "no new kernel work is charged");
    assert_eq!(requests[0].as_ref().unwrap().charged_work(), 0);
}

#[test]
fn producer_caller_pass_refuses_before_its_source_or_handler_reads() {
    use worth_query_host::facade::{
        application_contribution::{
            WorthQueryAdvancementDenial as Denial,
            WorthQueryManagedComputationResourceDenial as Resource,
            WorthQueryMemoryLimitLevel as Level,
        },
        application_entry::WorthQueryApplicationOutputDemandDenial as OutputDenial,
        primary_graph::installed_source_reads_on_this_thread_for_test as reads,
    };
    let _guard = checkpoint_recovery_test_guard();
    let application = install(|graph| {
        facts::seed_set(graph, "even", -0.0);
        seed_entry(
            graph,
            &["even"],
            0,
            RegionEntry {
                id: 0,
                region: 0,
                value: 1.0,
                work: 1,
                fault: None,
            },
        );
    });
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut interest = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<OracleProgram, RegionConnection>(&application)
        .unwrap();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        for zero_memory in [false, true] {
            let policy = support::CHECKPOINT_EXECUTION_POLICY.budget();
            let _restore = Restore(
                place(placement),
                bound(Some(worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    if zero_memory {
                        0
                    } else {
                        policy.charged_memory_bytes()
                    },
                    if zero_memory {
                        policy.work_ceiling()
                    } else {
                        0
                    },
                ))),
            );
            let before = reads();
            reports();
            take_calls();
            let OutputDenial::Demand(denial) = interest.advance(&request).err().unwrap() else {
                panic!("the caller must retain its opening cause");
            };
            let WorthQueryOutputDemandDenialKind::ExecutionRequest(cause) = denial.kind() else {
                panic!("the caller opens before its source: {denial:?}");
            };
            if !zero_memory {
                assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
            } else {
                match placement {
                    Placement::Serial => {
                        assert_eq!(cause, Denial::Resource(Resource::PolicyMemoryLimit))
                    }
                    Placement::Leased(_) => {
                        let Denial::Resource(Resource::MemoryLimit {
                            level,
                            requested,
                            admitted,
                        }) = cause
                        else {
                            panic!("lease bytes must be retained");
                        };
                        assert_eq!(level, Level::Policy);
                        assert!(requested > 0);
                        assert_eq!(admitted, 0);
                    }
                    Placement::World | Placement::Certified { .. } => {
                        unreachable!("this probe names its placements")
                    }
                }
            }
            assert_eq!(reads(), before);
            assert_eq!(take_calls(), OwnerCalls::default());
            assert_eq!(reports(), vec![Err(cause)]);
        }
        bound(None);
        let before = reads();
        interest.advance(&request).unwrap();
        assert!(
            reads() > before,
            "admitted caller advance enters its counted source reader"
        );
    }
}
