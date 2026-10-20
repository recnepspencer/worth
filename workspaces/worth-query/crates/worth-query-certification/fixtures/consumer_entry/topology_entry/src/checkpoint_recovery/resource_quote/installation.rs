//! Install only the counted schema, with the ordinary principal and host owners.
use super::*;
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus;

pub(super) fn install_program(
    seed: impl FnOnce(&mut primary_graph::WorthQueryPrimaryGraphBootstrap<QuoteSchema>),
) -> application_installation::WorthQueryProgramApplicationRuntime<QuoteSchema, program::Program> {
    let program = ApplicationProgramAuthoring::<QuoteSchema, program::Program>::begin()
        .validated_program()
        .expect("the counted program is complete");
    let declaration = QuoteSchema::declaration().expect("the counted schema is valid");
    application_installation::in_memory_program(
        program,
        declaration,
        ((),),
        support::limits(32, support::invalidation(128 * 1024 * 1024, None, 32)),
        |_phase, graph, installed| {
            let principal = installed
                .principal_binding(ConsumerPrincipalBinding::reference::<QuoteSchema>())
                .expect("the original principal mapping is installed");
            graph.bind_principal(
                &principal,
                primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                1_u64,
                support::external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            seed(graph);
            Ok(())
        },
    )
    .expect("the isolated counted source installs")
}

/// Contribution-owned execution deliberately has no selected application program.
pub(super) fn install_ordinary(
    seed: impl FnOnce(&mut primary_graph::WorthQueryPrimaryGraphBootstrap<QuoteSchema>),
) -> primary_graph::WorthQueryPrimaryGraphApplicationRuntime<QuoteSchema> {
    application_installation::in_memory(
        QuoteSchema::declaration().expect("the counted schema is valid"),
        ((),),
        support::limits(32, support::invalidation(128 * 1024 * 1024, None, 32)),
        |_phase, graph, installed| {
            let principal = installed
                .principal_binding(ConsumerPrincipalBinding::reference::<QuoteSchema>())
                .expect("the original principal mapping is installed");
            graph.bind_principal(
                &principal,
                primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                1_u64,
                support::external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            seed(graph);
            Ok(())
        },
    )
    .expect("the ordinary counted source installs")
}
