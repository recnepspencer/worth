//! The real caller loop refreshes a Ready whose producer changed a gathered fact.
use super::super::own_write::{arm_own_write, OwnWrite, OwnWriteRead};
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
// Both producers retain their declared work; a successful run measures delivery.
const RUNS: usize = 1;
const ENTRY_WORK: usize = 1_000_000;
const PRODUCER_WORK: usize = ENTRY_WORK + WORK_BESIDE_ENTRIES;
// A caller allows one declared entry checkpoint with generous headroom,
// but cannot fund two. Resetting its request on retry would admit both.
const CALLER_WORK: u64 = ENTRY_WORK as u64 + ENTRY_WORK as u64 / 2;

#[test]
fn caller_refresh_retry_spends_what_the_first_attempt_left() {
    let _guard = checkpoint_recovery_test_guard();
    let _restore = Restore(place(Placement::Serial), bound(None));
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        place(placement);
        run();
    }
}

#[path = "advancement_custody/application_fixture.rs"]
mod application_fixture;
#[path = "advancement_custody/post_producer_delivery.rs"]
mod post_producer_delivery;

fn run() {
    use worth_query_host::facade::{
        application_contribution::WorthQueryAdvancementDenial as Denial,
        application_entry::WorthQueryApplicationOutputDemandDenial as OutputDenial,
    };
    // Retry-only calibration: first publication and refresh fit; two actual
    // million-unit producer checkpoints still cannot fit this finite caller.
    let ceiling = 2 * ENTRY_WORK as u64 - 1;
    assert!(ceiling < 2 * ENTRY_WORK as u64);
    bound(None);
    let application = application_fixture::application();
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
    let policy = support::observation::CHECKPOINT_EXECUTION_POLICY
        .accounting()
        .expect("this fixture declares accounting");
    bound(Some(worth_foundational::ExecutionBudget::new(
        NonZeroUsize::MIN,
        policy.charged_memory_bytes(),
        Some(ceiling),
    )));
    reports();
    passes();
    room().clear();
    let outcome = interest.settle(&request);
    let requests = reports();
    let attempts = passes();
    let charged = requests[0].as_ref().unwrap().charged_work();
    eprintln!("finite retry: ceiling={ceiling}; charged={charged}; passes={attempts:?}; openings={}; producer_runs={}", requests.len(), room().len());
    assert!(charged <= ceiling);
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
        WorthQueryOutputDemandDenialKind::ExecutionRequest(Denial::NestedStopped),
        "charged={charged}; attempts={attempts:?}; diagnostics={diagnostics}"
    );
    assert_eq!(attempts.len(), 3, "charged first pass, refresh, then retry");
    assert!(
        room().first().unwrap().outcome.is_ok(),
        "the first producer completed"
    );
    assert_eq!(
        room().len(),
        1,
        "the spent request cannot admit another producer"
    );
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
            let policy = support::observation::CHECKPOINT_EXECUTION_POLICY
                .accounting()
                .expect("this fixture declares accounting");
            let _restore = Restore(
                place(placement),
                bound(Some(worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    if zero_memory {
                        Some(0)
                    } else {
                        policy.charged_memory_bytes()
                    },
                    if zero_memory {
                        policy.work_ceiling()
                    } else {
                        Some(0)
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
                    Placement::Serial | Placement::Leased(_) => {
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
