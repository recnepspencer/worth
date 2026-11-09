//! A corrected source and its real required output retain separate custody.
use super::{
    authentication::*,
    calculation_query,
    correction::RecordedEdit,
    edit::{Edit, Value},
    host,
    other_edit::OtherEdit,
    output_program::{OutputProgram, Root},
    query::Row,
    schema::NumericSchema,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::*,
    declaration::application_operation::ApplicationMutationBinding,
    primary_graph::WorthQueryApplicationRequiredOutputSource,
    runtime::ExecutionAllocationPolicy,
};
pub(super) mod co_committed;

pub fn run() {
    let application = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let key = "source-with-required-output".to_owned();
    let original = co_committed::publish_and_deliver(&application, 23, || {
        request
            .advanced()
            .mutate(Edit {
                key: "numeric".to_owned(),
                value: Value::Primary(11),
            })
            .without_source()
            .idempotency(&key)
            .execute_performed::<OutputProgram, Root>(
                &application,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    assert_eq!(read_calculation(&application), 23);
    let key = "disjoint-with-required-output".to_owned();
    co_committed::publish_and_deliver(&application, 23, || {
        request
            .advanced()
            .mutate(OtherEdit {
                key: "numeric".to_owned(),
                value: Value::Secondary(19),
            })
            .without_source()
            .idempotency(&key)
            .execute_performed::<OutputProgram, Root>(
                &application,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    let selected = request.corrections().select(&original, limits()).unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    let stopped_scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let stopped_principal = authenticate_operator(application.installed_schema(), &stopped_scope);
    let stopped_request = application
        .runtime()
        .request(&stopped_principal, &stopped_scope);
    let before_cancel = head(&application);
    let contacts = super::other_edit::handler_contacts();
    cancellation.cancel();
    let cancelled = stopped_request
        .corrections()
        .reverse(&selected)
        .idempotency("cancelled-before-correction-preparation")
        .limits(limits())
        .prepare_performed::<RecordedEdit, OutputProgram, Root>(&application);
    assert!(
        matches!(
            cancelled,
            Err(WorthQueryApplicationCorrectionDenial::ExecutionRequest(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            ))
        ),
        "preparation must preserve the actual request cancellation"
    );
    assert_eq!(super::other_edit::handler_contacts(), contacts);
    assert_eq!(head(&application), before_cancel);
    assert_eq!(
        host::read(&application),
        [Row {
            operand: None,
            primary: 11,
            secondary: 19
        }]
    );
    assert_eq!(read_calculation(&application), 23);
    assert!(matches!(
        request.corrections().reverse(&selected).idempotency("ordinary-output-refusal").limits(limits()).prepare(),
        Err(WorthQueryApplicationCorrectionDenial::Derivation(
            worth_query_host::facade::primary_graph::WorthQueryCorrectionDerivationDenial::RequiredOutputReconciliation))
    ));
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("reverse-with-required-output")
        .limits(limits())
        .prepare_performed::<RecordedEdit, OutputProgram, Root>(&application)
        .unwrap();
    let WorthQueryRequiredCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("checked required-root correction must prepare");
    };
    assert_eq!(
        host::read(&application),
        [Row {
            operand: None,
            primary: 11,
            secondary: 19
        }]
    );
    let performed = prepared.publish().unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = performed else {
        panic!("corrected source must carry native output custody");
    };
    let correction = performed.receipt().clone();
    assert_eq!(
        host::read(&application),
        [Row {
            operand: None,
            primary: 5,
            secondary: 19
        }]
    );
    assert_eq!(
        read_calculation(&application),
        23,
        "source publication precedes required output delivery"
    );
    assert_eq!(
        correction
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .relational(),
        original.commit_reference()
    );
    let before_retry = head(&application);
    let cancellation = WorthQueryCancellationSource::new();
    let stopped_scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let stopped_principal = authenticate_operator(application.installed_schema(), &stopped_scope);
    let stopped_request = application
        .runtime()
        .request(&stopped_principal, &stopped_scope);
    cancellation.cancel();
    let failure = performed
        .start_required_outputs(
            &application,
            &stopped_request,
            WorthQueryOutputDemandControls::host_policy(),
        )
        .err()
        .expect("cancelled output start must return the performed source");
    assert!(
        matches!(failure.denial(),
            WorthQueryRequiredOutputPreparationDenial::Demand(WorthQueryApplicationOutputDemandDenial::Demand(denial))
            if matches!(denial.kind(), worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::ExecutionRequest(
                WorthQueryAdvancementDenial::Interrupted(WorthQueryManagedComputationInterruption::Cancelled)
            ))
        ),
        "output handoff retains exact request cancellation: {:?}",
        failure.denial()
    );
    let performed = failure.into_performed();
    assert_eq!(
        performed.receipt().commit_reference(),
        correction.commit_reference()
    );
    assert_eq!(head(&application), before_retry);
    assert_eq!(
        host::read(&application),
        [Row {
            operand: None,
            primary: 5,
            secondary: 19
        }]
    );
    assert_eq!(
        read_calculation(&application),
        23,
        "performed correction survives while its original output delivery stays pending"
    );
    let retry = request
        .corrections()
        .reverse(&selected)
        .idempotency("reverse-with-required-output")
        .limits(limits())
        .prepare_performed::<RecordedEdit, OutputProgram, Root>(&application)
        .unwrap();
    let WorthQueryRequiredCorrectionPreparation::Settled(
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
            WorthQueryApplicationMutationOutcome::AlreadyCommitted(replayed),
        ),
    ) = retry
    else {
        panic!("an exact pending-output retry reports the original correction without replacing custody");
    };
    assert_eq!(replayed.commit_reference(), correction.commit_reference());
    assert_eq!(
        head(&application),
        before_retry,
        "retry cannot publish another inverse"
    );
    assert_eq!(read_calculation(&application), 23);
    let foreign = host::install_with_outputs();
    let foreign_scope = request_scope();
    let foreign_principal = authenticate_operator(foreign.installed_schema(), &foreign_scope);
    let foreign_request = foreign
        .runtime()
        .request(&foreign_principal, &foreign_scope);
    let failure = performed
        .start_required_outputs(
            &foreign,
            &foreign_request,
            WorthQueryOutputDemandControls::host_policy(),
        )
        .err()
        .expect("a foreign host cannot consume the original output custody");
    assert!(
        matches!(failure.denial(), WorthQueryRequiredOutputPreparationDenial::Demand(
        WorthQueryApplicationOutputDemandDenial::Demand(denial))
        if denial.kind() == worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSource),
        "the foreign delivery refusal must come from runtime ownership: {:?}",
        failure.denial()
    );
    let performed = failure.into_performed();
    assert_eq!(
        performed.receipt().commit_reference(),
        correction.commit_reference()
    );
    assert_eq!(
        head(&application),
        before_retry,
        "foreign delivery must perform no owner effects"
    );
    settle(&application, performed);
    assert_eq!(read_calculation(&application), 11);
    assert_eq!(
        host::read(&application),
        [Row {
            operand: None,
            primary: 5,
            secondary: 19
        }]
    );
    println!("required-output correction, pre-effect/post-performed cancellation, pending replay and original-owner delivery passed");
}
pub fn limits() -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN)
}
pub fn settle<B>(
    application: &host::OutputHost,
    performed: WorthQueryPerformedApplicationMutation<NumericSchema, B, OutputProgram, Root>,
) where
    B: ApplicationMutationBinding<NumericSchema>
        + WorthQueryApplicationRequiredOutputSource<NumericSchema, super::output_program::Connection>,
{
    // The installed aftermath owner supplies the entry kind, never a caller flag.
    let entry = performed
        .receipt()
        .aftermath_causality()
        .map(|fact| fact.role());
    let expected_contacts = match entry {
        None => 0,
        Some(
            worth_query_host::facade::primary_graph::WorthQueryAftermathCausalRole::RecordedReverse
            | worth_query_host::facade::primary_graph::WorthQueryAftermathCausalRole::RecordedReapply,
        ) => 1,
    };
    let source_commit = performed.receipt().commit_reference().clone();
    eprintln!(
        "required delivery entry={entry:?} expected_contacts={expected_contacts} calculation={}",
        read_calculation(application)
    );
    let before_head = head(application);
    let before_rows = host::read(application);
    let before_value = read_calculation(application);
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let ((), delivery) = worth_relational::facade::inspection::observe_maintenance_work(8, || {
        let mut started = performed
            .start_required_outputs(
                application,
                &request,
                WorthQueryOutputDemandControls::host_policy(),
            )
            .unwrap_or_else(|_| {
                panic!("lawful current request must start the original required output")
            });
        let WorthQueryApplicationProgramOutputProgress::Settled(settled) = started
            .required_output_mut()
            .settle(application, &request)
            .unwrap()
        else {
            panic!("the real arithmetic producer must settle");
        };
        assert_eq!(
            settled.root_producer_contacts_in_this_demand(),
            expected_contacts
        );
        assert!(settled.root_receipt().is_some());
        if entry.is_none() {
            assert_eq!(
                settled.root_receipt().unwrap().commit_reference(),
                &source_commit,
                "the real producer receipt belongs to this one source commit"
            );
        }
    });
    if entry.is_none() {
        assert_eq!(
            delivery.published_rows.len() + delivery.omitted_publications,
            0
        );
        assert_eq!(head(application), before_head);
        assert_eq!(host::read(application), before_rows);
        assert_eq!(read_calculation(application), before_value);
    }
}
pub fn read_calculation(application: &host::OutputHost) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    application
        .runtime()
        .request(&principal, &scope)
        .query(calculation_query::Read {
            key: "numeric".to_owned(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .value
}
pub fn head<
    P: worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition<
        NumericSchema,
    >,
>(
    application: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<NumericSchema, P>,
) -> impl Eq + std::fmt::Debug + use<P> {
    application
        .with_host_advancement(|phase| {
            application
                .runtime()
                .on_branch(application.runtime().current_world())
                .select(&phase)
                .unwrap()
                .product()
                .selected_commit()
                .clone()
        })
        .unwrap()
}
