//! Stops on the real performed lineage are not evidence of supersession.
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest};

#[test]
fn receipt_lineage_preserves_original_cancellation_and_deadline() {
    crate::domain_computation::primary_graph::output_lineage::own_write_fixture::with_committed_own_write(|world, receipt, _cell, _resources| {
        let selected = world.application.with_host_advancement(|phase| {
            world.selected_product(&phase.test_host_request())
        }).unwrap();
        let graph = world.application.runtime.primary_graph().unwrap();
        let handle = graph.integration_handle();
        let lineage = handle.output_lineage.lock().unwrap();

        let check = |execution: &ExecutionRequest<'_, '_>| lineage.current_output_matches_receipt(
            world.application.runtime.authority_identity().as_u64(),
            &world.application.installed_schema.binding_identity(),
            selected.product().observation(), &receipt, execution);
        let live = crate::test_request_policy::serial_request();

        let cancelled = CancellationSource::new();
        cancelled.cancel();
        let stopped = live.clone().with_cancellation(cancelled.token());
        let expired = live.clone().with_deadline(Some(std::time::Instant::now()));
        for (request, expected) in [
            (&stopped, worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled),
            (&expired, worth_relational::facade::transactions::RelationalExecutionDenialCause::DeadlineElapsed),
        ] {
            assert!(matches!(check(&ExecutionRequest::serial(request)),
                Err(CompanionPreflightStop::Execution(cause)) if cause == expected));
        }
        // This actual own-write computation is stale. A healthy read may
        // report that truth; an interrupted or exhausted read must not.
        assert_eq!(check(&ExecutionRequest::serial(&live)), Ok(false));
    });
}
