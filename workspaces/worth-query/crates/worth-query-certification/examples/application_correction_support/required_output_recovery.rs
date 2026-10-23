//! A real durability failure preserves the correction's original required-output owner.
use super::{
    authentication::*,
    correction::RecordedEdit,
    edit::{Edit, Value},
    host,
    other_edit::{self, OtherEdit},
    output_program::{OutputProgram, Root},
    required_output_courts::{head, limits, read_calculation, settle},
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::*,
    declaration::application_aftermath::ApplicationCorrectionBinding,
    primary_graph::{
        WorthQueryApplicationIdempotencyResolution, WorthQueryManagedApplicationRecoveryDenial,
        WorthQueryOutputDemandDenialKind,
    },
    runtime::{ExecutionAllocationPolicy, ProductUnpublishedCause},
};
pub fn run() {
    let application = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let source_key = "source-for-unpublished-correction".to_owned();
    let source = request
        .advanced()
        .mutate(Edit {
            key: "numeric".to_owned(),
            value: Value::Primary(11),
        })
        .without_source()
        .idempotency(&source_key)
        .execute_performed::<OutputProgram, Root>(
            &application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(source) = source else {
        panic!("ordinary source must perform");
    };
    let original = source.receipt().clone();
    settle(&application, source);
    assert_eq!(read_calculation(&application), 23);
    let selected = request.corrections().select(&original, limits()).unwrap();
    let before = other_edit::handler_contacts();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("unpublished-required-correction")
        .limits(limits())
        .prepare_performed::<RecordedEdit, OutputProgram, Root>(&application)
        .unwrap();
    let WorthQueryRequiredCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("inverse must prepare once");
    };
    let after_prepare = other_edit::handler_contacts();
    assert_eq!(after_prepare, (before.0 + 1, before.1 + 1));
    let target = head(&application);
    application
        .with_host_advancement(|phase| {
            let execution = phase
                .checked_execution_request_for(application.runtime().product_runtime())
                .unwrap();
            application
                .runtime()
                .fail_next_durable_append_for_test(&execution);
        })
        .unwrap();
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::ProductUnpublished(mut recovery) = outcome
    else {
        panic!("the real append failure must preserve the original required-source partial");
    };
    assert_eq!(
        recovery.initial_cause(),
        ProductUnpublishedCause::SettlementPending
    );
    assert_eq!(
        head(&application),
        target,
        "unpublished owner effects cannot advance the product"
    );
    assert_eq!(other_edit::handler_contacts(), after_prepare);
    let retry = request
        .corrections()
        .reverse(&selected)
        .idempotency("unpublished-required-correction")
        .limits(limits())
        .prepare_performed::<RecordedEdit, OutputProgram, Root>(&application)
        .unwrap();
    assert!(
        matches!(
            retry,
            WorthQueryRequiredCorrectionPreparation::ProductUnpublished(_)
        ),
        "equivalent correction retry reports the original partial without replacing its owner"
    );
    assert_eq!(other_edit::handler_contacts(), after_prepare);

    let key = RecordedEdit::inverse_key("unpublished-required-correction");
    let foreign_key = "different-required-correction".to_owned();
    let wrong_key = request
        .advanced()
        .mutate(inverse())
        .without_source()
        .idempotency(&foreign_key)
        .recover_unpublished_required_in_program(&mut recovery, &application);
    assert!(
        matches!(wrong_key, Err(WorthQueryApplicationRecoveryRequestDenial::Recovery(
        WorthQueryManagedApplicationRecoveryDenial::Demand(ref denial))) if denial.kind() == WorthQueryOutputDemandDenialKind::ForeignSource),
        "fresh recovery must retain the original inverse key: {wrong_key:?}"
    );
    assert!(recovery.performed().is_none());
    let wrong = request
        .advanced()
        .mutate(OtherEdit {
            key: "numeric".to_owned(),
            value: Value::Primary(6),
        })
        .without_source()
        .idempotency(&key)
        .recover_unpublished_required_in_program(&mut recovery, &application);
    assert!(
        matches!(wrong, Err(WorthQueryApplicationRecoveryRequestDenial::Recovery(
        WorthQueryManagedApplicationRecoveryDenial::Demand(ref denial))) if denial.kind() == WorthQueryOutputDemandDenialKind::ForeignSource),
        "fresh recovery cannot replace the actual retained inverse input: {wrong:?}"
    );
    assert!(recovery.performed().is_none());

    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    let stopped_principal = authenticate_operator(application.installed_schema(), &stopped);
    cancellation.cancel();
    let stopped_request = application.runtime().request(&stopped_principal, &stopped);
    let stopped_result = recovery.recover_recorded_correction(&application, &stopped_request);
    assert!(matches!(stopped_result, Err(WorthQueryApplicationRecoveryRequestDenial::Recovery(
        WorthQueryManagedApplicationRecoveryDenial::ExecutionDenied(
            worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(
                worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled))))),
        "cancelled recovery must preserve its native cancellation: {stopped_result:?}");
    assert!(recovery.performed().is_none());
    assert_eq!(other_edit::handler_contacts(), after_prepare);

    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(application.installed_schema(), &fresh_scope);
    let fresh = application
        .runtime()
        .request(&fresh_principal, &fresh_scope);
    let progress = recovery
        .recover_recorded_correction(&application, &fresh)
        .unwrap();
    assert!(matches!(
        progress,
        WorthQueryProgramSourceRecoveryProgress::Performed
    ));
    let (outputs, initial, performed, prior) = recovery
        .promote_recorded_correction(&application, &fresh)
        .unwrap_or_else(|(denial, _)| panic!("original correction promotion: {denial:?}"))
        .into_parts();
    assert_eq!(initial, ProductUnpublishedCause::SettlementPending);
    assert!(prior.is_empty());
    let (read, publication, cleanup, pending_cleanup) = performed.into_parts();
    assert!(publication.is_none());
    assert!(cleanup.is_none());
    let released = application
        .with_application_advancement(&fresh_scope, |phase| {
            application.release_product_publication_recovery_in_advancement(
                &phase,
                pending_cleanup.expect("performed recovery retains its cleanup"),
                0,
            )
        })
        .unwrap()
        .unwrap();
    assert!(released.is_complete());
    let WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) =
        read.unwrap().into_resolution()
    else {
        panic!("fresh recovery must read the exact canonical correction key");
    };
    assert_eq!(
        outputs.receipt().commit_reference(),
        receipt.commit_reference()
    );
    assert_eq!(
        receipt
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .unwrap()
            .relational(),
        original.commit_reference()
    );
    assert_eq!(
        other_edit::handler_contacts(),
        after_prepare,
        "recovery never executes the inverse again"
    );
    assert_eq!(read_calculation(&application), 23);
    let mut outputs = outputs
        .start_required_outputs(
            &application,
            &fresh,
            WorthQueryOutputDemandControls::host_policy(),
        )
        .unwrap_or_else(|(_, denial)| panic!("the original recovered demand starts: {denial:?}"));
    let WorthQueryApplicationProgramOutputProgress::Settled(settled) =
        outputs.settle(&application, &fresh).unwrap()
    else {
        panic!("recovered correction's native arithmetic output must settle");
    };
    assert_eq!(settled.root_producer_contacts_in_this_demand(), 1);
    assert_eq!(read_calculation(&application), 11);
    assert_eq!(other_edit::handler_contacts(), after_prepare);
    println!(
        "genuine unpublished correction recovery preserves input, cancellation and output custody"
    );
}
fn inverse() -> OtherEdit {
    OtherEdit {
        key: "numeric".to_owned(),
        value: Value::Primary(5),
    }
}
