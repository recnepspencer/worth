//! Original publication continuity differs from qualification at a later head.
use super::*;
mod memory;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_contribution::{
    WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
};

#[test]
fn no_publication_tree_keeps_its_source_endpoint_after_unrelated_head_movement() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let mut world = installation::install(&crate::installed_schema());
    {
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        settle_tree(&world, &request)
            .close(&request)
            .unwrap_or_else(|stop| panic!("original warm close: {:?}", stop.denial()));
        let source = request.retain_current_read().unwrap();
        let contacts = world
            .application
            .producer_contacts_on_this_thread_for_test();
        let tree = settle_tree(&world, &request);
        assert!(tree.settlement().performed_source().is_none());
        assert_eq!(tree.settlement().root_producer_contacts_in_this_demand(), 0);
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            source.selected_commit(),
            "the complete original graph produced no new publication"
        );
        let closed = tree
            .close_for_delivery(&request)
            .unwrap_or_else(|stop| panic!("reused close: {:?}", stop.denial()));
        let original = closed.qualify_original(&request).unwrap();
        assert_eq!(original.source_commit(), source.selected_commit());
        assert_eq!(original.output_commit(), source.selected_commit());

        // Real unrelated work changes the World head without changing this tree.
        edit_source(&world, &request, "remote-a", 42, 0x9182_3040);
        let later = request.retain_current_read().unwrap();
        assert_ne!(later.selected_commit(), original.output_commit());
        let current = closed.qualify_at(&request.at(&later)).unwrap();
        assert_eq!(
            current.output_commit(),
            later.selected_commit(),
            "ordinary currentness remains lawful"
        );
        let historical = closed.qualify_original(&request).unwrap();
        assert_eq!(
            historical.output_commit(),
            original.output_commit(),
            "no later-head capture"
        );
        assert_eq!(historical.source_commit(), source.selected_commit());
        assert_eq!(
            world
                .application
                .producer_contacts_on_this_thread_for_test(),
            contacts,
            "both kinds of qualification preserve normal reuse without producer work"
        );

        let other = world
            .application
            .with_host_advancement(|phase| {
                world
                    .application
                    .branches()
                    .fork(world.application.current_world())
                    .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                    .create(&phase)
                    .unwrap()
            })
            .unwrap();
        let foreign = world
            .application
            .request(&principal, &scope)
            .on_branch(other);
        assert!(matches!(
            closed.qualify_original(&foreign),
            Err(WorthQueryOperationDeliveryDenial::FreshRequestMismatch)
        ));
        let cancellation = WorthQueryCancellationSource::new();
        let stopped = WorthQueryRequestScope::new(None, cancellation.token());
        let stopped_principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &stopped,
        ))
        .unwrap();
        cancellation.cancel();
        assert!(matches!(
            closed.qualify_original(&world.application.request(&stopped_principal, &stopped)),
            Err(WorthQueryOperationDeliveryDenial::Execution(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            ))
        ));
        assert_eq!(
            closed.qualify_original(&request).unwrap().output_commit(),
            source.selected_commit(),
            "the same original carrier remains retryable after both refusals"
        );
        {
            let child_source = foreign.retain_current_read().unwrap();
            let child = settle_tree(&world, &foreign)
                .close_for_delivery(&foreign)
                .unwrap_or_else(|stop| panic!("genuine fork tree close: {:?}", stop.denial()));
            let child_current = foreign.retain_current_read().unwrap();
            child
                .qualify_at(&foreign.at(&child_current))
                .expect("the native owner admits this actual fork's output reuse or rebuild");
            let child_original = child
                .qualify_original(&foreign)
                .expect("original endpoint qualification preserves lawful native fork behavior");
            assert_eq!(
                child_original.source_commit(),
                child_source.selected_commit()
            );
            assert_eq!(
                child_original.output_commit(),
                child_current.selected_commit()
            );
        }
        world
            .application
            .with_host_advancement(|phase| world.application.on_branch(other).close(&phase))
            .unwrap()
            .unwrap();
    }
    world
        .application
        .close_conditional_runtime()
        .expect("all original read custody released");
}

#[test]
fn unrelated_publication_between_original_outputs_cannot_become_the_tree_endpoint() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let mut world = installation::install(&crate::installed_schema());
    {
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        edit_source(&world, &request, "anchor-a", 2, 0x9182_3041);
        let source = request.retain_current_read().unwrap();
        let mut operation = request
            .demand(PlanarOutputDemand::new("anchor-a"))
            .retain_program_outputs::<ConsumerProgram, ConsumerProgramRoot>(&world.application)
            .unwrap_or_else(|stop| panic!("original construction: {:?}", stop.denial()));
        let mut first_publication = None;
        for _ in 0..256 {
            operation = match operation.advance(&request) {
                TreeAdvance::Pending(original) => original,
                TreeAdvance::Settled(_) => panic!(
                    "the complete graph cannot settle before observing its first root publication"
                ),
                TreeAdvance::Refused { denial, .. } => panic!("original progress: {denial:?}"),
                TreeAdvance::Terminal(stop) => panic!("original terminal: {:?}", stop.denial()),
            };
            let current = request.retain_current_read().unwrap();
            if current.selected_commit() != source.selected_commit() {
                first_publication = Some(current.selected_commit().clone());
                break;
            }
        }
        let first_publication =
            first_publication.expect("actual original root publication precedes dependent work");
        edit_source(&world, &request, "remote-a", 43, 0x9182_3042);
        let unrelated = request.retain_current_read().unwrap();
        assert_ne!(unrelated.selected_commit(), &first_publication);
        let tree = settle_original_tree(operation, &request);
        assert_eq!(tree.settlement().output_count(), 4);
        let closed = tree
            .close_for_delivery(&request)
            .unwrap_or_else(|stop| panic!("original complete close: {:?}", stop.denial()));
        let current = request.retain_current_read().unwrap();
        assert_ne!(
            current.selected_commit(),
            unrelated.selected_commit(),
            "actual dependent publications follow the unrelated gap"
        );
        closed.qualify_at(&request.at(&current))
            .expect("this original complete tree is genuinely current at the explicitly supplied later head");
        let stopped = closed
            .qualify_original(&request)
            .map(|_| ())
            .expect_err("an unrelated publication cannot bridge the original movement chain");
        let intended = match &stopped {
            WorthQueryOperationDeliveryDenial::Currentness(
                WorthQueryOutputCurrentnessDenial::Source(_),
            ) => true,
            WorthQueryOperationDeliveryDenial::Currentness(
                WorthQueryOutputCurrentnessDenial::Output(cause),
            ) => {
                cause.kind() == WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable
                    || cause.kind() == WorthQueryOutputDemandDenialKind::Superseded
            }
            _ => false,
        };
        assert!(
            intended,
            "the connected prefix has no qualified complete original endpoint: {stopped:?}"
        );
        closed
            .qualify_at(&request.at(&current))
            .expect("original-endpoint refusal preserves the same closed carrier");
    }
    world
        .application
        .close_conditional_runtime()
        .expect("gap refusal preserves native cleanup and releases all readonly history");
}
