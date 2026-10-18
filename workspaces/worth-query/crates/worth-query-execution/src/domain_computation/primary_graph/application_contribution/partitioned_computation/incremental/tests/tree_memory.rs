//! An incremental run holds its next tree's declared bound before it builds
//! it. A run whose path combines no longer fit its work rebuilds the tree
//! from every leaf, as a full run builds it, and a budget one byte short of
//! that build refuses it before the reducer combines anything.

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};

use worth_execution::ReductionTree;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

use super::super::super::super::request_execution::{
    query_test_execution_authority_for_test, test_policy,
};
use super::super::super::super::WorthQueryManagedComputationResourceDenial as Resource;
use super::*;

static COMBINES: AtomicUsize = AtomicUsize::new(0);

fn counted_sum(left: &u64, right: &u64) -> u64 {
    COMBINES.fetch_add(1, Ordering::Relaxed);
    left + right
}

/// The declared work and result bytes of the test computation.
const DECLARED: u64 = 4096;

/// A second run of four partitions whose one recomputed kernel leaves one
/// unit of work short of the combines the retained tree charges, so the tree
/// is built again. Placed under `memory`, leased, then serial, each
/// with the combines it ran.
fn rebuilt_under(memory: Option<u64>, leased: bool) -> Vec<(Outcome, usize, u64)> {
    let placements: Vec<Box<dyn Fn() -> RuntimeWorldExecutionPlacement<'static>>> = match memory {
        None => vec![Box::new(|| {
            RuntimeWorldExecutionPlacement::Serial(
        crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy(
            std::num::NonZeroUsize::MIN, 1 << 30,
        ),
    )
        })],
        Some(memory) => {
            let policy = move || test_policy(NonZeroUsize::new(2).unwrap(), memory);
            if leased {
                vec![Box::new(move || RuntimeWorldExecutionPlacement::Leased {
                    authority: query_test_execution_authority_for_test(),
                    policy: policy(),
                })]
            } else {
                vec![Box::new(move || {
                    RuntimeWorldExecutionPlacement::Serial(policy())
                })]
            }
        }
    };
    let world = installed_authorization_world(true);
    placements
        .iter()
        .map(|placement| {
            let installed = installed_over(4, StatusRead::Gather(1), counted_sum);
            let first = first_run_over_four(&world, &installed);
            // The first run charged its calls, four kernels of one unit and
            // its combines. The moved kernel takes the place of one unit and
            // all but one combine, so one combine short of the tree is left.
            let charged = first.outcome.as_ref().unwrap().1;
            let moved = DECLARED + 2 - charged;
            *installed.owner.bump.lock().unwrap() = 10;
            *installed.owner.work.lock().unwrap() = [usize::try_from(moved).unwrap(), 1];
            COMBINES.store(0, Ordering::Relaxed);
            let mut live_scope_bytes = None;
            let run = projected_attempt::attempt_placed_observing(
                &world,
                &installed,
                Some(prior_of(first, true)),
                &live_scope(),
                placement(),
                |execution| {
                    let request = execution.execution_request().unwrap();
                    let worth_execution::ExecutionAllocationPolicy::Execution(lease) =
                        request.allocation_policy()
                    else {
                        panic!("the actual request holds its native lease");
                    };
                    let dispatch = execution.dispatch().unwrap();
                    let tree_scope =
                        worth_execution::ExecutionWorkCeiling::scope_memory_bound_for_test(
                            Some(dispatch.native_lease_for_test()),
                            None,
                        )
                        .unwrap();
                    // Real projection admission also remains live. Measure that
                    // custody, then quote the tree scope in its actual lineage.
                    live_scope_bytes = Some(lease.charged_memory_bytes() + tree_scope);
                },
            );
            assert_eq!(run.gathered, [1], "only the moved partition gathers again");
            (
                run.outcome,
                COMBINES.load(Ordering::Relaxed),
                live_scope_bytes.unwrap(),
            )
        })
        .collect()
}

fn first_run_over_four(world: &AuthorizationWorld, installed: &Installed) -> Attempt {
    let first = attempt(
        world,
        installed,
        Some(ComputationPrior::new(edition(), Err(Cause::FirstRun), None)),
    );
    assert_eq!(first.gathered, [0, 1, 2, 3]);
    first
}

fn refused_by_memory(outcome: &Outcome) -> Option<u64> {
    match outcome {
        Err(WorthQueryPartitionedComputationDenial::Resource(Resource::MemoryLimit {
            requested,
            ..
        })) => Some(*requested),
        _ => None,
    }
}

#[test]
fn a_rebuild_one_byte_short_of_its_bound_refuses_before_it_combines() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let unbounded = rebuilt_under(None, false);
    let [(outcome, reached, _)] = unbounded.as_slice() else {
        panic!("one unbounded placement");
    };
    let reached = *reached;
    assert_eq!(
        *outcome,
        Err(WorthQueryPartitionedComputationDenial::Resource(
            Resource::WorkExhausted
        )),
        "the rebuild stops where a full run's build stops"
    );
    assert!(reached > 0, "it combined as far as its work reached");

    // Native width two and serial width one hold different worker contexts.
    // Each posture must refuse at its own physical-memory boundary.
    for leased in [false, true] {
        let policy = test_policy(NonZeroUsize::new(2).unwrap(), 1 << 30);
        let token = worth_execution::CancellationToken::new();
        let policy = if leased {
            policy
        } else {
            worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                policy.determinism(),
                worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    policy.budget().charged_memory_bytes(),
                    policy.budget().work_ceiling(),
                ),
            )
        };
        let authority = if leased {
            query_test_execution_authority_for_test()
        } else {
            worth_execution::ExecutionAuthority::serial()
        };
        let root = authority
            .request_lease(worth_execution::LeaseRequest {
                policy,
                deadline: None,
                cancellation: token,
            })
            .unwrap();
        let baseline = rebuilt_under(Some(1 << 30), leased);
        let [(baseline_outcome, baseline_combines, scope)] = baseline.as_slice() else {
            panic!("one actual placement");
        };
        assert_eq!(baseline_outcome, outcome);
        assert_eq!(*baseline_combines, reached);
        let build = ReductionTree::<u64, fn(&u64, &u64) -> u64>::checked_serial_build_memory_bound(
            4, DECLARED,
        )
        .unwrap();
        // The rebuild holds its tree bound, four inline leaf copies, one
        // replacement result, actual projection custody and tree scope.
        // The recomputed kernel records one RLE run (all advances are one
        // unit). Its reserved vector remains live through the rebuild.
        let storage =
            worth_execution::ExecutionKernelReceiptStorage::new(root.reserve_memory(0).unwrap())
                .unwrap();
        let (receipt, _) = worth_execution::ExecutionWorkCeiling::new(u64::MAX)
            .run_with_checkpoint(Some(&root), None, |context| {
                let (result, receipt) = context.record_kernel(&storage, |context| {
                    context.checkpoint(1)?;
                    Ok::<_, worth_execution::MapKernelFailure<()>>(())
                });
                result.unwrap();
                receipt.unwrap()
            })
            .unwrap();
        let checkpoint_hold = worth_execution::ChargedBytes::additional_charged_bytes(&receipt);
        drop((receipt, storage));
        let admitted = build
            + 5 * u64::try_from(size_of::<(worth_foundational::PartitionIdentity, u64)>()).unwrap()
            + *scope
            + checkpoint_hold;
        let refused = admitted - 1;
        for (outcome, combines, _) in rebuilt_under(Some(refused), leased) {
            let requested = refused_by_memory(&outcome)
                .unwrap_or_else(|| panic!("one byte short is a memory refusal: {outcome:?}"));
            assert!(
                requested > build,
                "the refusal is the rebuild's own bound: {requested} against a build of {build}"
            );
            assert_eq!(combines, 0, "nothing is combined before the bound is held");
        }
        for (outcome, combines, _) in rebuilt_under(Some(admitted), leased) {
            assert_eq!(combines, reached, "with its bound held the rebuild runs: leased={leased} build={build} scope={scope} steps={checkpoint_hold} admitted={admitted} outcome={outcome:?}");
        }
    }
}
