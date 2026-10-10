use super::*;
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

type Application = application_installation::WorthQueryProgramApplicationRuntime<
    TemporalHostSchema,
    TemporalInstallationProgram,
>;

#[test]
fn explicit_publication_limit_survives_checkpoint_installation_and_enforces_commit_boundary() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = application_installation::in_memory_program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        (configuration(),),
        limits(),
        |_phase, graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "original", 0, 1, true);
            Ok(())
        },
    )
    .expect("ordinary source installation");
    let checkpoint = source
        .with_host_advancement(|host_phase| {
            source
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .expect("installed host call opens");
    let original = read_intent(&source);
    drop(source);
    for maximum in [1, 65_536] {
        let application = application_installation::in_memory_program_from_checkpoint(
            validated_program(),
            TemporalHostSchema::declaration().unwrap(),
            (configuration(),),
            limits()
                .with_maximum_publication_records(std::num::NonZeroUsize::new(maximum).unwrap()),
            checkpoint.clone(),
        )
        .expect("a record budget governs new commits, not valid checkpoint recovery");
        assert_eq!(read_intent(&application), original);
        let installed = application.conditional::<TemporalConditional>().unwrap();
        let installed = installed.as_ref().as_ref().unwrap();
        let outcome = change_input(
            request_execution,
            &application,
            &installed.invariant,
            application.current_world(),
            "changed",
        );
        if maximum == 1 {
            assert!(
                matches!(
                    outcome,
                    primary_graph::WorthQueryApplicationCommitOutcome::Aborted
                ),
                "{outcome:?}"
            );
            assert_eq!(
                read_intent(&application),
                original,
                "rejected publication leaves authoritative fields unchanged"
            );
        } else {
            outcome
                .require_committed()
                .expect("same real command commits within its explicit allowance");
            let changed = read_intent(&application);
            assert_eq!(changed.input, "changed");
            assert_eq!(changed.revision, 2);
        }
    }
}

fn configuration() -> TemporalContributionConfiguration {
    let contacts = ContactCounters::default();
    let (installation_predicate, _) = Predicate::controlled(contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    TemporalContributionConfiguration {
        installation_predicate,
        definition_predicate: Arc::new(definition_predicate),
        clock_source,
        clock_control,
        contacts,
        install_route: true,
    }
}

fn limits() -> WorthQueryInMemoryApplicationLimits {
    WorthQueryInMemoryApplicationLimits::new(
        product_world_resources(1_024),
        runtime::WorthQueryApplicationCandidateResourceProfile::physical_resources(5_120, 2_048)
            .unwrap(),
        runtime::WorthQueryApplicationQueryResourceProfile::bounded(5_120, 2_048, usize::MAX, 128)
            .unwrap(),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}

fn read_intent(application: &Application) -> IntentQueryResult {
    let request = request_scope();
    let schema = application.installed_schema();
    let binding = schema
        .principal_binding(TemporalPrincipalBinding::reference())
        .unwrap();
    let authentication = admit_identity_adapter(schema);
    let external = block_on(authentication.authenticate((), &request)).unwrap();
    let selected = application
        .with_host_advancement(|host_phase| {
            application
                .on_branch(application.current_world())
                .select(&host_phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let principal = application
        .with_host_advancement(|host_phase| {
            selected
                .resolve_authenticated_principal(
                    &host_phase,
                    &binding,
                    &external,
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let scope = application
        .with_host_advancement(|host_phase| {
            selected
                .resolve_entity(
                    &host_phase,
                    IntentIdentityField::reference(),
                    "intent-1".to_owned(),
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let query = schema
        .certification_query(TemporalIntentQuery::reference())
        .unwrap();
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = application
        .with_host_advancement(|host_phase| {
            selected
                .admit_application_query(
                    &host_phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    primary_graph::WorthQueryProductQueryControls::new(
                        std::num::NonZeroUsize::new(8).unwrap(),
                        std::num::NonZeroUsize::new(64).unwrap(),
                        &request,
                    ),
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let result = application
        .with_host_advancement(|phase| {
            application
                .execute_application_query_one_shot(&phase, plan)
                .expect("authoritative fields remain readable")
        })
        .expect("installed host call opens");
    assert_eq!(result.rows().len(), 1);
    result.rows()[0].clone()
}
