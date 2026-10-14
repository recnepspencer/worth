//! A stopped source read retains opaque generated suspension recovery.
use super::output_correspondence::observed_source;
use super::*;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::primary_graph::WorthQueryProviderSessionControlStopKind as Stop;
use worth_query_host::facade::{
    primary_graph, product::WorthQueryProductBranchAdmissionDenial as Branch,
};
use worth_query_topology_entry::{PlanarFinalOutputProducer, PlanarSourceAdjustment};

#[test]
fn suspension_recovery_keeps_the_cancelled_source_read_and_live_retry() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| prove(false))
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn suspension_recovery_keeps_cancellation_inside_adoption_preparation() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| prove(true))
        .unwrap()
        .join()
        .unwrap();
}

fn prove(at_preparation: bool) {
    let schema = crate::installed_schema();
    let world = installation::install(&schema);
    let application = &world.application;
    let scope = authentication::request_scope();
    let adapter = authentication::admit(application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = application.request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-b".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(observed_source(&request, "anchor-b"))
        .idempotency(&982)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else { panic!("source mutation publishes") };
    let branch = performed.receipt().product_branch();
    let mut performed = performed
        .start_required_outputs(
            application,
            &request,
            worth_query_host::facade::application_entry::WorthQueryOutputDemandControls::new(
                std::num::NonZeroUsize::new(4096).unwrap(),
                std::num::NonZeroUsize::new(8192).unwrap(),
            ),
        )
        .unwrap_or_else(|failure| panic!("output graph starts: {:?}", failure.denial()));
    assert!(matches!(
        performed
            .required_output_mut()
            .settle(application, &request)
            .unwrap(),
        WorthQueryApplicationProgramOutputProgress::Settled(_)
    ));
    drop(performed);
    let retained = read_y(&request, "final:anchor-b");
    let source = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    application
        .with_host_advancement(|phase| {
            application.fail_next_durable_append_for_test(
                &phase
                    .checked_execution_request_for(application.product_runtime())
                    .unwrap(),
            )
        })
        .unwrap();
    let selected = application
        .with_host_advancement(|phase| application.on_branch(branch).select(&phase).unwrap())
        .unwrap();
    let recovery = match selected
        .suspend_current_generated_output::<PlanarFinalOutputProducer<crate::ConsumerSchema>>(
            &scope, source,
        ) {
        Err(primary_graph::WorthQueryGeneratedOutputSuspensionFailure::ProductUnpublished(
            recovery,
        )) => recovery,
        _ => panic!("durability refusal retains actual World recovery"),
    };
    let cancellation = WorthQueryCancellationSource::new();
    let interrupted = WorthQueryRequestScope::new(None, cancellation.token());
    let preparation_cancellation = cancellation.clone();
    let _preparation_guard = at_preparation.then(|| {
        application
            .product_runtime()
            .before_adoption_preparation_for_test(move || preparation_cancellation.cancel())
    });
    let _named = application
        .product_runtime()
        .observe_named_read_consults_for_test();
    let observed_runtime = application.product_runtime().clone();
    let at_read = std::rc::Rc::new(std::cell::Cell::new(false));
    let fired = std::rc::Rc::clone(&at_read);
    let (result, _) = worth_execution::with_request_consult_observer_for_test(
        move |_at| {
            if !at_preparation
                && observed_runtime.generated_recovery_source_selection_is_active_for_test()
                && !fired.get()
            {
                fired.set(true);
                cancellation.cancel();
            }
        },
        || {
            application.continue_generated_output_suspension_recovery::<PlanarFinalOutputProducer<crate::ConsumerSchema>>(recovery, &interrupted)
        },
    );
    if !at_preparation {
        assert!(
            at_read.get(),
            "the generated recovery source selection consult was reached"
        );
    }
    let failure = match result {
        Err(failure) => failure,
        Ok(_) => panic!("cancelled source read retains recovery"),
    };
    assert_eq!(
        failure.stage(),
        if at_preparation {
            primary_graph::WorthQueryGeneratedOutputSuspensionRecoveryStage::ExecutionDenied(
                worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(
                    worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled,
                ),
            )
        } else {
            primary_graph::WorthQueryGeneratedOutputSuspensionRecoveryStage::SourceSelection(
                Branch::Stopped(Err(Stop::Cancelled)),
            )
        }
    );
    let recovery = failure.into_recovery();
    assert_eq!(recovery.product_branch(), branch);
    assert_eq!(read_y(&request, "final:anchor-b"), retained);
    let suspended = application
        .continue_generated_output_suspension_recovery::<PlanarFinalOutputProducer<crate::ConsumerSchema>>(recovery, &scope)
        .unwrap_or_else(|failure| panic!("live retry adopts the retained effects: {:?}", failure.stage()));
    assert_eq!(suspended.product_branch(), branch);
}
