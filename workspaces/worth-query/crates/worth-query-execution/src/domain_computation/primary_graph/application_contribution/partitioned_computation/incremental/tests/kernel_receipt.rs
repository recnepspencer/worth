//! Kernel completion receipts settle at the same boundary with reuse either way.
use super::*;

#[test]
fn a_tree_stop_keeps_no_kernel_completion_receipt_with_reuse_either_way() {
    let _observation = QueryObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let installed = installed(StatusRead::Gather(1), sum);
    let request = live_scope();
    let policy = super::super::super::super::request_execution::test_policy(
        std::num::NonZeroUsize::MIN,
        None,
    );
    let placement = RuntimeWorldExecutionPlacement::Serial(policy);
    let first = projected_attempt::attempt_billed(
        &world,
        &installed,
        Some(ComputationPrior::new(edition(), Err(Cause::FirstRun), None)),
        &request,
        placement,
        None,
    );
    let ceiling = first
        .bill
        .expect("the original compute stage ran")
        .charged_work()
        - 1;

    // All kernels fit; the ceiling omits the tree's last accepted unit.

    let prior = prior_of(first, false);
    let mut stops = Vec::new();
    for prior in [Some(prior), None] {
        let reuse = prior.is_some();
        let receipts = Arc::new(Mutex::new(Vec::new()));
        let stopped = projected_attempt::attempt_billed_with_receipts(
            &world,
            &installed,
            prior,
            &request,
            placement,
            ceiling,
            Arc::clone(&receipts),
        );
        if reuse {
            assert!(
                stopped.gathered.is_empty(),
                "the incremental row carries every kernel"
            );
        }
        assert!(
            !stopped.tree_runs.is_empty(),
            "the denial reached the tree stage"
        );
        assert!(stopped.outcome.is_err());
        assert_eq!(
            *receipts.lock().unwrap(),
            Vec::<(u64, u64)>::new(),
            "a tree-stage stop settles no kernel completion receipt, reuse={reuse}"
        );
        stops.push((
            stopped.outcome,
            stopped
                .bill
                .expect("the original compute stage ran")
                .charged_work(),
            stopped
                .bill
                .expect("the original compute stage ran")
                .charged_span(),
        ));
    }
    assert_eq!(stops[0], stops[1], "tree-stop kind and bill agree by value");
}
