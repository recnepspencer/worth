//! A real producer partial stays owned through native release refusal and retry.
use super::*;
use worth_query_host::facade::primary_graph::{
    RuntimeWorldRecoveryDenial, WorthQueryApplicationUncommitted, WorthQueryOutputReadCloseDenial,
    WorthQueryProductUnpublishedRecoveryReleaseDenial,
};

#[test]
fn ordinary_unpublished_producer_close_retries_original_native_recovery() {
    let _guard = checkpoint_recovery_test_guard();
    let mut application = support::install(None);
    crate::producer::reset_provider_contacts();
    let decisions_before = crate::handler::handler_contacts();
    {
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let before = request.retain_current_read().unwrap();
        let mut pending = request
            .demand(PlanarOutputDemand::new("anchor-a"))
            .retain_program_outputs::<CheckpointProgram, CheckpointRoot>(&application)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
        application
            .with_host_advancement(|phase| {
                application.fail_next_durable_append_for_test(
                    &phase
                        .checked_execution_request_for(application.product_runtime())
                        .unwrap(),
                )
            })
            .unwrap();
        let mut terminal = None;
        for _ in 0..256 {
            pending = match pending.advance(&request) {
                WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
                WorthQueryProgramOutputOperationAdvance::Terminal(value) => {
                    terminal = Some(value);
                    break;
                }
                _ => panic!("real producer publication must retain its failed append"),
            };
        }
        let mut terminal = terminal.expect("actual unpublished producer terminal is owned");
        assert!(matches!(
            terminal.producer_outcome(),
            Some(WorthQueryApplicationUncommitted::ProductUnpublished(_))
        ));
        let contacts = crate::producer::provider_contacts();
        let producer_decisions = crate::handler::handler_contacts();
        assert_eq!(producer_decisions, decisions_before + 1);
        assert!(contacts > 0);
        assert_eq!(
            before.selected_commit(),
            request.retain_current_read().unwrap().selected_commit(),
            "unpublished effects cannot become a current output"
        );
        // This independently opened inspection loan is not the continuation.
        // The operation still owns the original actual producer partial.
        let catalog = application
            .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
            .unwrap();
        assert_eq!(catalog.rows().len(), 1);
        let inspector = application
            .readmit_product_publication_recovery(catalog.rows()[0].handle())
            .unwrap();
        let held = inspector.inspect().unwrap();
        let refused = match terminal.close(&request) {
            Err(refused) => refused,
            Ok(_) => panic!("live original effects loan must refuse native retirement"),
        };
        assert!(matches!(
            refused.denial(),
            WorthQueryProgramOutputCloseDenial::Native(WorthQueryOutputReadCloseDenial::Recovery(
                WorthQueryProductUnpublishedRecoveryReleaseDenial::World(
                    RuntimeWorldRecoveryDenial::CallerCapabilityLive
                )
            ))
        ));
        assert!(refused.terminal_denial().is_some());
        let cancelled = authentication::WorthQueryCancellationSource::new();
        cancelled.cancel();
        let stopped = authentication::WorthQueryRequestScope::new(None, cancelled.token());
        let refused = match refused.retry_close(&application.request(&principal, &stopped)) {
            Err(refused) => refused,
            Ok(_) => panic!("cancelled close retains the same recovery owner"),
        };
        assert!(matches!(
            refused.denial(),
            WorthQueryProgramOutputCloseDenial::Execution(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            )
        ));
        drop(held);
        let fresh_scope = authentication::WorthQueryRequestScope::new(
            None,
            authentication::WorthQueryCancellationSource::new().token(),
        );
        let fresh = application.request(&principal, &fresh_scope);
        let mut closed = refused
            .retry_close(&fresh)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
        assert!(
            closed.retired_component_count() > 0,
            "real native recovery owner retirement must complete"
        );
        assert_eq!(
            crate::producer::provider_contacts(),
            contacts,
            "close never reenters provider callbacks"
        );
        assert_eq!(
            crate::handler::handler_contacts(),
            producer_decisions,
            "retiring the original unpublished record never reexecutes its handler"
        );
        assert_eq!(
            before.selected_commit(),
            fresh.retain_current_read().unwrap().selected_commit()
        );
        assert!(application
            .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
            .unwrap()
            .rows()
            .is_empty());
    }
    application
        .close_conditional_runtime()
        .expect("all genuine read and failed-producer custody discharged");
}
