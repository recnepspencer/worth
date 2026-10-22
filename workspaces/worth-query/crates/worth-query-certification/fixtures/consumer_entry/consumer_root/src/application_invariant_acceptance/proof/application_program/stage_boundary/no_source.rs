//! Ordinary public advancement renews authority without replacing the original tree.
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::{
        WorthQueryProgramOutputAdvanceDenial as Denial, WorthQueryProgramOutputCloseDenial,
        WorthQueryProgramOutputOperationAdvance as Advance,
    },
};
use worth_query_topology_entry::PlanarOutputDemand;

#[test]
fn no_source_tree_keeps_yielded_custody_across_refusals_and_fresh_completion() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let mut world = installation::install(&crate::installed_schema());
    let other = installation::install(&crate::installed_schema());
    {
        let adapter = authentication::admit(world.application.installed_schema());
        let scope = authentication::request_scope();
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let operation = request
            .demand(PlanarOutputDemand::new("anchor-a"))
            .retain_program_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
                &world.application,
            )
            .unwrap_or_else(|stop| panic!("original tree: {:?}", stop.denial()));
        let operation = match operation.advance(&request) {
            Advance::Pending(operation) => operation,
            _ => panic!("the public caller receives custody before child discovery"),
        };
        assert_eq!(
            world
                .application
                .producer_contacts_on_this_thread_for_test(),
            1
        );
        let operation = match operation.advance(&other.application.request(&principal, &scope)) {
            Advance::Refused {
                operation,
                denial: Denial::FreshRequestMismatch,
            } => operation,
            _ => panic!("a foreign runtime must return the same yielded owner"),
        };
        let cancellation = WorthQueryCancellationSource::new();
        let stopped = WorthQueryRequestScope::new(None, cancellation.token());
        let stopped_principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &stopped,
        ))
        .unwrap();
        cancellation.cancel();
        let operation =
            match operation.advance(&world.application.request(&stopped_principal, &stopped)) {
                Advance::Refused {
                    operation,
                    denial:
                        Denial::Execution(WorthQueryAdvancementDenial::Interrupted(
                            WorthQueryManagedComputationInterruption::Cancelled,
                        )),
                } => operation,
                _ => panic!("cancelled authority must refuse before dependent work"),
            };
        assert_eq!(
            world
                .application
                .producer_contacts_on_this_thread_for_test(),
            1
        );
        let mut operation = Some(operation);
        let settled = super::super::super::settle(|| {
            let scope = authentication::request_scope();
            let principal = authentication::block_on(adapter.authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ))
            .unwrap();
            match operation
                .take()
                .unwrap()
                .advance(&world.application.request(&principal, &scope))
            {
                Advance::Pending(next) => {
                    operation = Some(next);
                    None
                }
                Advance::Settled(settled) => Some(settled),
                Advance::Refused { denial, .. } => panic!("fresh original tree: {denial:?}"),
                Advance::Terminal(terminal) => {
                    panic!("fresh original tree: {:?}", terminal.denial())
                }
            }
        });
        assert_complete_original_tree(settled.settlement());
        let contacts = world
            .application
            .producer_contacts_on_this_thread_for_test();
        assert_eq!(contacts, 5);
        // This native gate checks complete original settlement and discovery evidence.
        settled.deliver_current(&request).unwrap();
        settled
            .close(&request)
            .unwrap_or_else(|stop| panic!("original cleanup: {:?}", stop.denial()));
        assert_eq!(
            world
                .application
                .producer_contacts_on_this_thread_for_test(),
            contacts,
            "delivery and cleanup cannot restart settled producers"
        );
        let early = request
            .demand(PlanarOutputDemand::new("anchor-a"))
            .retain_program_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
                &world.application,
            )
            .unwrap_or_else(|stop| panic!("reused original tree: {:?}", stop.denial()));
        let early = match early.advance(&request) {
            Advance::Pending(early) => early,
            _ => panic!("a reused root also returns before independent discovery"),
        };
        let refused = match early.close(&world.application.request(&stopped_principal, &stopped)) {
            Err(refused) => refused,
            Ok(_) => panic!("cancelled close must keep the original root interest"),
        };
        assert!(matches!(
            refused.denial(),
            WorthQueryProgramOutputCloseDenial::Execution(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            )
        ));
        refused
            .retry_close(&request)
            .unwrap_or_else(|stop| panic!("same early native close: {:?}", stop.denial()));
        assert_eq!(
            world
                .application
                .producer_contacts_on_this_thread_for_test(),
            contacts,
            "early close cannot discover or execute children"
        );
    }
    world
        .application
        .close_conditional_runtime()
        .expect("all original native interests close");
}
