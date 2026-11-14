//! A real inverse saves an undefined reciprocal source; only its native leaf may complete it.
use super::super::{
    authentication::*,
    correction::RecordedEdit,
    edit::{Edit, Value},
    host,
    schema::NumericSchema,
};
use super::{
    program::{Program, Root},
    reciprocal,
};
use std::num::NonZeroUsize;
use worth_query_host::facade::declaration::application_program::*;
use worth_query_host::facade::{
    application_entry::*, application_installation::*, primary_graph::*,
    runtime::ExecutionAllocationPolicy,
};
use worth_relational::facade::inspection::observe_maintenance_work;
type Host = WorthQueryProgramApplicationRuntime<NumericSchema, Program>;
pub(super) fn install(worker: bool) -> Host {
    host::install_program_with_seed_and_limits(
        ApplicationProgramAuthoring::<NumericSchema, Program>::begin()
            .validated_program()
            .unwrap(),
        host::roster().support(super::super::program::validated::<
            super::super::output_program::OutputProgram,
        >()),
        |_| Ok(()),
        if worker {
            host::worker_limits()
        } else {
            host::installation_limits()
        },
    )
}
fn edit(application: &Host, value: u64, key: &str) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    match request
        .advanced()
        .mutate(Edit {
            key: "numeric".into(),
            value: Value::Primary(value),
        })
        .without_source()
        .idempotency(&key.to_owned())
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        outcome => panic!("the independently declared source-only edit must commit: {outcome:?}"),
    }
}
fn scheduling(denial: &WorthQueryRequiredOutputPreparationDenial) -> bool {
    matches!(denial, WorthQueryRequiredOutputPreparationDenial::Demand(
        WorthQueryApplicationOutputDemandDenial::Demand(denial))
        if denial.kind() == WorthQueryOutputDemandDenialKind::SchedulingDeferred)
}
pub(super) fn run(worker: bool) {
    let application = install(worker);
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let contacts = application
        .runtime()
        .producer_contacts_on_this_thread_for_test();
    edit(&application, 0, "reciprocal-zero");
    let original = edit(&application, 11, "reciprocal-eleven");
    assert_eq!(
        application
            .runtime()
            .producer_contacts_on_this_thread_for_test(),
        contacts,
        "the original binding is installed as source-only, not a postponed fixed edit"
    );
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let selected = request.corrections().select(&original, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("reciprocal-reverse")
        .limits(limits)
        .prepare_performed::<RecordedEdit, Program, Root>(&application)
        .unwrap();
    let WorthQueryRequiredCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("the real recorded preimage must prepare its declared inverse");
    };
    let performed = prepared.publish().unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = performed else {
        panic!("the actual deferred inverse must retain its source custody");
    };
    let source = performed.receipt().clone();
    assert_eq!(
        source.aftermath_causality().unwrap().role(),
        WorthQueryAftermathCausalRole::RecordedReverse
    );
    assert_eq!(host::read(&application)[0].primary, 0);
    assert_eq!(
        application
            .runtime()
            .producer_contacts_on_this_thread_for_test(),
        contacts
    );
    let mut started = performed
        .start_required_outputs(
            &application,
            &request,
            WorthQueryOutputDemandControls::host_policy(),
        )
        .unwrap_or_else(|failure| panic!("real deferred source: {:?}", failure.denial()));
    let output = started.required_output_mut();
    assert!(
        scheduling(
            &output
                .finish_unavailable(&application, &request)
                .err()
                .unwrap()
        ),
        "unexecuted custody is not a terminal domain refusal"
    );
    let mut second = request
        .advanced()
        .recover_required_outputs::<Program, Root>(
            &application,
            &source,
            reciprocal::Demand {
                key: "numeric".into(),
            },
            WorthQueryOutputDemandControls::host_policy(),
        )
        .unwrap();
    let failure = output
        .settle(&application, &request)
        .err()
        .expect("division by zero refuses");
    let WorthQueryRequiredOutputPreparationDenial::Demand(
        WorthQueryApplicationOutputDemandDenial::Demand(denial),
    ) = failure
    else {
        panic!("actual producer domain refusal expected");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::ProducerDomainDenied
    );
    assert_eq!(
        denial.domain_reason(),
        Some("A reciprocal is undefined for zero.")
    );
    assert_eq!(
        denial.decode_producer_denial::<reciprocal::DenialValue>(),
        Ok(reciprocal::ReciprocalDenial::DivisionByZero)
    );
    assert_eq!(
        denial.producer_denial_bytes(),
        Some(br#""DivisionByZero""#.as_slice())
    );
    assert_eq!(
        application
            .runtime()
            .producer_contacts_on_this_thread_for_test()
            - contacts,
        1
    );
    assert!(
        scheduling(
            &output
                .finish_unavailable(&application, &request)
                .err()
                .unwrap()
        ),
        "two actual interests cannot both consume one unavailable completion"
    );
    // The second owner retains its actual preparation on refusal, then releases it.
    assert!(second.finish_unavailable(&application, &request).is_err());
    drop(second);
    super::authority::refuse(&application, output);
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let before = application
        .runtime()
        .producer_contacts_on_this_thread_for_test();
    let (unavailable, work) =
        observe_maintenance_work(8, || output.finish_unavailable(&application, &request));
    let unavailable = unavailable.unwrap();
    assert!(unavailable
        .source_receipt()
        .retained_receipt()
        .expect("the actual deferred inverse retains its performed source receipt")
        .is_same_authoritative_commit(&source));
    assert_eq!(
        unavailable
            .denial()
            .decode_producer_denial::<reciprocal::DenialValue>(),
        Ok(reciprocal::ReciprocalDenial::DivisionByZero)
    );
    assert_eq!(work.published_rows.len() + work.omitted_publications, 0);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(host::read(&application)[0].primary, 0);
    assert_eq!(
        application
            .runtime()
            .producer_contacts_on_this_thread_for_test(),
        before
    );
    assert!(matches!(
        output.finish_unavailable(&application, &request),
        Err(WorthQueryRequiredOutputPreparationDenial::Closed)
    ));
    drop(started);
    assert_eq!(application.retained_source_custody_count_for_test(), 0);
    assert_eq!(
        application.prepared_required_output_source_count_for_test(),
        0
    );
    assert_eq!(application.active_application_attempts_for_test(), 0);
    assert_eq!(application.world_active_publication_attempts_for_test(), 0);
    settled(worker);
}
fn settled(worker: bool) {
    let application = install(worker);
    let original = edit(&application, 11, "reciprocal-positive");
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let selected = request.corrections().select(&original, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("reciprocal-positive-reverse")
        .limits(limits)
        .prepare_performed::<RecordedEdit, Program, Root>(&application)
        .unwrap();
    let WorthQueryRequiredCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("real inverse");
    };
    let performed = prepared.publish().unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = performed else {
        panic!("real inverse source");
    };
    let mut started = performed
        .start_required_outputs(
            &application,
            &request,
            WorthQueryOutputDemandControls::host_policy(),
        )
        .unwrap_or_else(|_| panic!("real positive inverse"));
    assert!(matches!(
        started
            .required_output_mut()
            .settle(&application, &request)
            .unwrap(),
        WorthQueryApplicationProgramOutputProgress::Settled(_)
    ));
    assert!(
        matches!(
            started
                .required_output_mut()
                .finish_unavailable(&application, &request),
            Err(WorthQueryRequiredOutputPreparationDenial::Closed)
        ),
        "an actual successful output cannot be relabelled unavailable"
    );
}
