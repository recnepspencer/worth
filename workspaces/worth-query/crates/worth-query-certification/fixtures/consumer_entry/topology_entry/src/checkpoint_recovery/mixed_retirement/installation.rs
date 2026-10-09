//! An isolated schema admits the replacement conditional without changing other programs.
use super::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc,
};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus;
pub(super) fn install(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
) -> application_installation::WorthQueryProgramApplicationRuntime<MixedSchema, MixedProgram> {
    let configuration = (TopologyConfiguration {
        setup_calls: Arc::new(AtomicUsize::new(0)),
        invariant_calls: Arc::new(AtomicUsize::new(0)),
        invariant_probe: Arc::new(AtomicUsize::new(0)),
        producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
        producer_domain_denial: Arc::new(AtomicBool::new(false)),
    },);
    let program = ApplicationProgramAuthoring::<MixedSchema, MixedProgram>::begin()
        .validated_program()
        .expect("the checkpoint program is complete");
    let declaration = MixedSchema::declaration().expect("the checkpoint schema is valid");
    match checkpoint {
        Some(checkpoint) => application_installation::in_memory_program_from_checkpoint(
            program,
            declaration,
            configuration,
            support::limits(
                32,
                // Window <= history is enforced by primary_graph/bootstrap/preparation.rs:91.
                support::invalidation(128 * 1_024 * 1_024, 1_000_000, 32),
            ),
            checkpoint,
        )
        .expect("the checkpoint restores"),
        None => application_installation::in_memory_program(
            program,
            declaration,
            configuration,
            support::limits(
                32,
                // Window <= history is enforced by primary_graph/bootstrap/preparation.rs:91.
                support::invalidation(128 * 1_024 * 1_024, 1_000_000, 32),
            ),
            |graph, installed| {
                let principal = installed
                    .principal_binding(ConsumerPrincipalBinding::reference::<MixedSchema>())
                    .expect("the principal mapping is installed");
                graph.bind_principal(
                    &principal,
                    primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                    1_u64,
                    support::external_identity(),
                    WorthQueryPrincipalMappingStatus::Enabled,
                )?;
                support::seed_cycle(graph);
                Ok(())
            },
        )
        .expect("the checkpoint source installs"),
    }
}
