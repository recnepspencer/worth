use super::*;

type Application = application_installation::WorthQueryProgramApplicationRuntime<
    TemporalHostSchema,
    TemporalInstallationProgram,
>;

#[test]
fn explicit_publication_limit_survives_checkpoint_installation_and_enforces_commit_boundary() {
    let source = application_installation::in_memory_program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        (configuration(),),
        limits(),
        |graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "original", 0, 1, true);
            Ok(())
        },
    )
    .expect("ordinary source installation");
    let checkpoint = source.capture_application_checkpoint().unwrap();
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
        runtime::WorthQueryApplicationCandidateResourceProfile::bounded(5_120, 2_048, 5_120)
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
        .on_branch(application.current_world())
        .select()
        .unwrap();
    let principal = selected
        .resolve_authenticated_principal(
            &binding,
            &external,
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = selected
        .resolve_entity(
            IntentIdentityField::reference(),
            "intent-1".to_owned(),
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = schema
        .certification_query(TemporalIntentQuery::reference())
        .unwrap();
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = selected
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            primary_graph::WorthQueryProductQueryControls::new(
                std::num::NonZeroUsize::new(8).unwrap(),
                std::num::NonZeroUsize::new(64).unwrap(),
                &request,
            ),
        )
        .unwrap();
    let result = application
        .execute_application_query_one_shot(plan)
        .expect("authoritative fields remain readable");
    assert_eq!(result.rows().len(), 1);
    result.rows()[0].clone()
}
