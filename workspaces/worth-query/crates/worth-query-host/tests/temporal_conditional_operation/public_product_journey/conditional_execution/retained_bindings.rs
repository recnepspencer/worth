//! Each exact occurrence creates or reuses its own conditional binding.
use super::*;

pub(super) fn assert_retained_bindings(
    world: &CourtroomWorld,
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    source_branch: worth_query_host::facade::product::WorthQueryProductBranch,
    sibling: worth_query_host::facade::product::WorthQueryProductBranch,
) {
    for (label, branch, expected_suppressed, expected_clocks) in [
        ("B", sibling, 1, 3),
        ("new A occurrence", source_branch, 0, 4),
        ("B-return", sibling, 1, 4),
    ] {
        let outcome = world
            .application
            .with_host_advancement(|phase| {
                world
                    .application
                    .on_branch(branch)
                    .select(&phase)
                    .unwrap()
                    .conditional_clock(
                        &phase
                            .execution_request_for(world.application.product_runtime())
                            .expect("phase belongs to application"),
                        &world.clock,
                    )
                    .unwrap()
            })
            .expect("installed host call opens")
            .observe(request_execution);
        let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) =
            outcome
        else {
            panic!(
                "a retained product binding must remain observable: {}",
                outcome_kind(&outcome)
            )
        };
        assert_eq!(
            receipt.retained_suppressed_wake_count(),
            expected_suppressed
        );
        assert_eq!(
            world
                .application
                .inspect_conditional_runtime()
                .managed_clock_count(),
            expected_clocks,
            "{label} must create or reuse the binding for its exact product occurrence"
        );
    }
}
