pub use super::*;
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

pub type Application = application_installation::WorthQueryProgramApplicationRuntime<
    TemporalHostSchema,
    TemporalInstallationProgram,
>;

pub fn assert_patch_budget_denial(outcome: primary_graph::WorthQueryApplicationCommitOutcome) {
    let primary_graph::WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("expected a pre-effect publication denial, got {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryApplicationCommitDenialKind::ProviderRejected
    );
    assert_eq!(
        denial.stage(),
        primary_graph::WorthQueryApplicationCommitDenialStage::ProviderCommit
    );
    let Some(worth_relational::facade::mvcc::TransactionCommitError::Publication { error, .. }) =
        denial.native_preparation_error()
    else {
        panic!("expected the native publication cause, got {denial:?}");
    };
    // The native owner has no typed patch-budget subcause.
    assert_eq!(error.detail, "patch record budget exceeded");
}

pub fn assert_source_change_key_absent(application: &Application) {
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
    let operation = schema
        .installed_operation(AmendTemporal::reference())
        .unwrap();
    let admission = application
        .with_host_advancement(|host_phase| {
            selected
                .authorize_operation(
                    &host_phase,
                    &principal,
                    &scope,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let key = primary_graph::WorthQueryApplicationIdempotencyBinding::for_host_commit::<
        TemporalHostSchema,
        AmendTemporal,
        _,
        _,
    >("source-change", "source-change")
    .unwrap();
    let read = application
        .with_host_advancement(|host_phase| {
            application
                .resolve_admitted_application_idempotency(&host_phase, &admission, key)
                .unwrap()
        })
        .expect("installed host call opens");
    assert_eq!(
        read.resolution(),
        &primary_graph::WorthQueryApplicationIdempotencyResolution::Unseen,
        "a pre-effect refusal records no idempotency commitment"
    );
}

pub fn configuration() -> TemporalContributionConfiguration {
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

pub fn limits() -> WorthQueryInMemoryApplicationLimits {
    WorthQueryInMemoryApplicationLimits::new(
        product_world_resources(1_024),
        runtime::WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(128).expect("concurrent graph work is non-zero"),
        ),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}

pub fn read_intent(application: &Application) -> IntentQueryResult {
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
