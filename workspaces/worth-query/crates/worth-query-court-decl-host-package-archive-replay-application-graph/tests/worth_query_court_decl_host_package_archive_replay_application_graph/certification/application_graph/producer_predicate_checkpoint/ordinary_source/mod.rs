pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::producer_predicate_checkpoint::ordinary_source::*;

#[test]
fn performed_mutation_attempt_report_preserves_source_custody() {
    let program =
        ApplicationProgramAuthoring::<DocumentRetentionSchema, OrdinaryAssessmentProgram>::begin()
            .validated_program()
            .expect("the ordinary source graph must validate");
    let host = publish(
        program,
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .unwrap();
    let runtime = host.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime.request(&principal, &scope);
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let before = runtime.producer_contacts_on_this_thread_for_test();
    let report = request
        .advanced()
        .mutate(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.into(),
                retention_days: 6,
            },
        })
        .without_source()
        .idempotency(&0x86_u64)
        .execute_performed_report::<OrdinaryAssessmentProgram, OrdinaryRoot>(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    let WorthQueryMutationHandlerWork::Captured(work) = report.decision_work() else {
        panic!("the performed source must carry its actual execution capture")
    };
    assert!(work.handler_contacted());
    assert!(work.projection_work().field_reads() > 0);
    let outcome = report
        .into_outcome()
        .expect("the actual source reaches publication");
    let performed = match outcome {
        WorthQueryApplicationPerformedMutationOutcome::Performed(performed) => performed,
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome) => {
            panic!("source must perform: {outcome:?}")
        }
        WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied(failure) => {
            panic!("required output denial: {:?}", failure.denial())
        }
        _ => panic!("source has not reached its performed owner"),
    };
    assert_eq!(
        request
            .query(DocumentRetentionRead {
                identity: DOCUMENT_IDENTITY.into(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        6
    );
    let receipt = performed.receipt().clone();
    drop(performed);
    let controls = WorthQueryOutputDemandControls::default();
    let mut recovered = request
        .advanced()
        .recover_required_outputs::<OrdinaryAssessmentProgram, OrdinaryRoot>(
            &host,
            &receipt,
            RetentionAssessmentDemand::new(DOCUMENT_IDENTITY),
            controls,
        )
        .expect("caller disposal must leave exact source custody recoverable by its owner");
    // A saved source and admitted root are not a terminal domain refusal.
    assert!(matches!(
        recovered.finish_unavailable(&host, &request),
        Err(worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
            worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial)
        )) if denial.kind() == worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::SchedulingDeferred
    ));
    let cancellation = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    let stopped =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            Some(std::time::Instant::now() + std::time::Duration::from_secs(30)),
            cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    cancellation.cancel();
    assert!(matches!(
        recovered.finish_unavailable(&host, &runtime.request(&principal, &stopped)),
        Err(worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
            worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial)
        )) if denial.kind() == worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::ExecutionRequest(
            worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(
                worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled
            )
        )
    ));
    let WorthQueryApplicationProgramOutputProgress::Settled(settled) =
        recovered.settle(&host, &request).unwrap()
    else {
        panic!("the existing installed assessment producer must settle")
    };
    assert_eq!(
        request
            .at(settled.observation())
            .query(DocumentRetentionRead {
                identity: DOCUMENT_IDENTITY.into(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        6
    );
    assert_eq!(
        runtime.producer_contacts_on_this_thread_for_test() - before,
        1,
        "the source edit and its recovered delivery execute one real producer in total"
    );
    assert!(matches!(
        recovered.finish_unavailable(&host, &request),
        Err(worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Closed)
    ));
    assert!(settled
        .root_receipt()
        .unwrap()
        .outputs_of::<RetentionAssessmentOutputs>()
        .unwrap()
        .entity::<AssessmentOutput>()
        .is_ok());
}
