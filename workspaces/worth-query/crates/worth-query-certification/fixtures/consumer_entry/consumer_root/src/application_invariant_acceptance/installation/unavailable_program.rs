use super::*;
use worth_query_decl::facade::application_program::ApplicationProgramRulePosture;
use worth_query_host::facade::domain::WorthQueryApplicationProgramInstallationDenialKind;

pub(crate) struct UnavailableWorld {
    pub(crate) application: installation::WorthQueryProgramApplicationRuntime<
        ConsumerSchema,
        crate::UnavailableConsumerProgram,
    >,
}

pub(crate) fn install_unavailable() -> UnavailableWorld {
    let application =
        installation::in_memory_program::<ConsumerSchema, crate::UnavailableConsumerProgram>(
            crate::application_program::validated_unavailable_program()
                .expect("the unavailable program declaration is complete"),
            ConsumerSchema::declaration().expect("the contributed declaration is valid"),
            configuration(),
            limits(),
            seed_graph,
        )
        .expect("an unavailable program still installs its declared graph");
    let rules = application.installed_program().rules();
    assert_eq!(rules.len(), 4);
    assert_eq!(
        rules
            .iter()
            .filter(|rule| rule.posture() == ApplicationProgramRulePosture::Unavailable)
            .map(|rule| (rule.identity(), rule.major()))
            .collect::<Vec<_>>(),
        [("PlannedFutureRule", 1), ("PositivePlanarTurn", 2)],
    );
    UnavailableWorld { application }
}

pub(crate) fn assert_rule_provider_postures() {
    let missing = installation::in_memory_program::<
        ConsumerSchema,
        crate::application_program::MissingRuleProviderProgram,
    >(
        crate::application_program::validated_missing_rule_provider_program().unwrap(),
        ConsumerSchema::declaration().unwrap(),
        configuration(),
        limits(),
        seed_graph,
    );
    let missing = match missing {
        Err(installation::WorthQueryInMemoryApplicationDenial::Program(denial)) => denial,
        Err(other) => panic!("wrong missing-rule denial: {other:?}"),
        Ok(_) => panic!("an available rule without a provider must be denied"),
    };
    assert_eq!(
        missing.kind(),
        WorthQueryApplicationProgramInstallationDenialKind::MissingInstalledRule,
    );
    assert_eq!(missing.subject(), "PlannedFutureRule");

    let stale = installation::in_memory_program::<
        ConsumerSchema,
        crate::application_program::StaleUnavailableRuleProgram,
    >(
        crate::application_program::validated_stale_unavailable_rule_program().unwrap(),
        ConsumerSchema::declaration().unwrap(),
        configuration(),
        limits(),
        seed_graph,
    );
    let stale = match stale {
        Err(installation::WorthQueryInMemoryApplicationDenial::Program(denial)) => denial,
        Err(other) => panic!("wrong stale-rule denial: {other:?}"),
        Ok(_) => panic!("an installed provider cannot remain declared unavailable"),
    };
    assert_eq!(
        stale.kind(),
        WorthQueryApplicationProgramInstallationDenialKind::UnavailableRuleInstalled,
    );
    assert_eq!(stale.subject(), "PositivePlanarTurn");
}

fn configuration() -> (TopologyConfiguration, Arc<AtomicUsize>) {
    (
        TopologyConfiguration {
            setup_calls: Arc::new(AtomicUsize::new(0)),
            invariant_calls: Arc::new(AtomicUsize::new(0)),
            invariant_probe: Arc::new(AtomicUsize::new(0)),
            producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
        },
        Arc::new(AtomicUsize::new(0)),
    )
}

fn limits() -> WorthQueryInMemoryApplicationLimits {
    WorthQueryInMemoryApplicationLimits::new(
        resources::world_resources(),
        runtime::WorthQueryApplicationCandidateResourceProfile::bounded(4096, 8192, 4096).unwrap(),
        runtime::WorthQueryApplicationQueryResourceProfile::bounded(4096, 4096, 4096, 32).unwrap(),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}

fn seed_graph(
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<ConsumerSchema>,
    installed: &domain::WorthQueryInstalledApplicationSchema<ConsumerSchema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    let principal = installed
        .principal_binding(ConsumerPrincipalBinding::reference::<ConsumerSchema>())
        .expect("the principal mapping is installed");
    graph.bind_principal(
        &principal,
        primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
        1_u64,
        authentication::external_identity(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    seed::seed_cycles(graph);
    Ok(())
}
