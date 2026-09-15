use super::*;

pub(crate) struct UnavailableWorld {
    pub(crate) application: installation::WorthQueryProgramApplicationRuntime<
        ConsumerSchema,
        crate::UnavailableConsumerProgram,
    >,
}

pub(crate) fn install_unavailable() -> UnavailableWorld {
    let configuration = (
        TopologyConfiguration {
            setup_calls: Arc::new(AtomicUsize::new(0)),
            invariant_calls: Arc::new(AtomicUsize::new(0)),
            invariant_probe: Arc::new(AtomicUsize::new(0)),
            producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
        },
        Arc::new(AtomicUsize::new(0)),
    );
    let limits = WorthQueryInMemoryApplicationLimits::new(
        resources::world_resources(),
        runtime::WorthQueryApplicationCandidateResourceProfile::bounded(4096, 8192, 4096).unwrap(),
        runtime::WorthQueryApplicationQueryResourceProfile::bounded(4096, 4096, 4096, 32).unwrap(),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    );
    let application =
        installation::in_memory_program::<ConsumerSchema, crate::UnavailableConsumerProgram>(
            crate::application_program::validated_unavailable_program()
                .expect("the unavailable program declaration is complete"),
            ConsumerSchema::declaration().expect("the contributed declaration is valid"),
            configuration,
            limits,
            |graph, installed| {
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
            },
        )
        .expect("an unavailable program still installs its declared graph");
    UnavailableWorld { application }
}
