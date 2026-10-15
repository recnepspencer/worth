//! Install one actual selected program with explicit source-revision deployment grants.
use super::*;

pub(in crate::application_invariant_acceptance) fn install_selected_program<Program>(
    foreign: &domain::WorthQueryInstalledApplicationSchema<ConsumerSchema>, candidate_bytes: u64, query_bytes: usize,
    roster: installation::WorthQueryApplicationProgramRoster<ConsumerSchema>,
    deployment_grants: &[crate::program_deployment::ConsumerDeploymentGrantSeed],
    program: worth_query_decl::facade::application_program::ValidatedApplicationProgram<ConsumerSchema, Program>,
) -> Result<ConsumerWorld<Program>, installation::WorthQueryInMemoryApplicationDenial>
where Program: worth_query_decl::facade::application_program::ApplicationProgramDefinition<ConsumerSchema,
    Contributions = <ConsumerProgram as worth_query_decl::facade::application_program::ApplicationProgramDefinition<ConsumerSchema>>::Contributions>,
Program::Outputs: installation::WorthQueryApplicationProgramRoots<ConsumerSchema>{
    install_selected_program_with_isolated_cycle(
        foreign,
        candidate_bytes,
        query_bytes,
        roster,
        deployment_grants,
        false,
        program,
    )
}


pub(in crate::application_invariant_acceptance) fn install_selected_program_with_isolated_cycle<Program>(
    foreign: &domain::WorthQueryInstalledApplicationSchema<ConsumerSchema>, candidate_bytes: u64, query_bytes: usize,
    roster: installation::WorthQueryApplicationProgramRoster<ConsumerSchema>,
    deployment_grants: &[crate::program_deployment::ConsumerDeploymentGrantSeed],
    isolated_cycle: bool,
    program: worth_query_decl::facade::application_program::ValidatedApplicationProgram<ConsumerSchema, Program>,
) -> Result<ConsumerWorld<Program>, installation::WorthQueryInMemoryApplicationDenial>
where Program: worth_query_decl::facade::application_program::ApplicationProgramDefinition<ConsumerSchema,
    Contributions = <ConsumerProgram as worth_query_decl::facade::application_program::ApplicationProgramDefinition<ConsumerSchema>>::Contributions>,
Program::Outputs: installation::WorthQueryApplicationProgramRoots<ConsumerSchema>{
    let topology_calls = Arc::new(AtomicUsize::new(0));
    let parameter_calls = Arc::new(AtomicUsize::new(0));
    let invariant_calls = Arc::new(AtomicUsize::new(0));
    let invariant_probe = Arc::new(AtomicUsize::new(0));
    let producer_authorization_denials = Arc::new(AtomicUsize::new(0));
    let configuration = (
        TopologyConfiguration {
            setup_calls: Arc::clone(&topology_calls),
            invariant_calls: Arc::clone(&invariant_calls),
            invariant_probe: Arc::clone(&invariant_probe),
            producer_authorization_denials: Arc::clone(&producer_authorization_denials),
            producer_domain_denial: Arc::new(AtomicBool::new(false)),
        },
        Arc::clone(&parameter_calls),
        (),
    );
    let limits = resources::application_limits(candidate_bytes, query_bytes);
    let deployment_scopes: std::collections::BTreeSet<_> = std::iter::once(*program.revision())
        .chain(deployment_grants.iter().map(|grant| grant.source()))
        .collect();
    let application = installation::in_memory_rostered_program::<ConsumerSchema, Program>(
        program,
        roster,
        ConsumerSchema::declaration().expect("the contributed declaration is valid"),
        configuration,
        limits,
        |bootstrap_phase, graph, installed| {
            reject_foreign_invariant_factory(installed, foreign);
            let principal = installed
                .principal_binding(ConsumerPrincipalBinding::reference::<ConsumerSchema>())
                .expect("the contributed principal mapping is installed");
            graph.bind_principal(
                &principal,
                primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                1_u64,
                authentication::external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            for source in &deployment_scopes {
                crate::program_deployment::bind_scope(graph, *source)?;
            }
            for grant in deployment_grants {
                grant.bind(graph)?;
            }
            seed::seed_cycles(bootstrap_phase, graph);
            if isolated_cycle {
                seed::seed_isolated_cycle(bootstrap_phase, graph);
            }
            graph.bind_principal(
                &principal,
                primary_graph::WorthQueryApplicationPrincipalKey::new("sibling-owner").unwrap(),
                2_u64,
                worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
                    "https://consumer.invalid/local",
                    "sibling-owner",
                )
                .unwrap(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            Ok(())
        },
    )?;
    assert_eq!(topology_calls.load(Ordering::SeqCst), 1);
    assert_eq!(parameter_calls.load(Ordering::SeqCst), 1);
    Ok(ConsumerWorld {
        application,
        invariant_calls,
        invariant_probe,
        producer_authorization_denials,
    })
}
