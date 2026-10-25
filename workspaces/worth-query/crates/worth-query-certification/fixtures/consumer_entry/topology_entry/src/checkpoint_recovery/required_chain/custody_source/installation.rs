//! An isolated schema admits the replacement conditional without changing other programs.
use super::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc,
};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus;
pub(in crate::checkpoint_recovery::required_chain) fn install(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
) -> application_installation::WorthQueryProgramApplicationRuntime<
    ValueSourceSchema,
    ValueSourceProgram,
> {
    let configuration = (TopologyConfiguration {
        final_feature: crate::planar_final_feature_runtime(),
        setup_calls: Arc::new(AtomicUsize::new(0)),
        invariant_calls: Arc::new(AtomicUsize::new(0)),
        invariant_probe: Arc::new(AtomicUsize::new(0)),
        invariant_read_observer: None,
        producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
        producer_domain_denial: Arc::new(AtomicBool::new(false)),
    },);
    let declaration = ValueSourceSchema::declaration().expect("the checkpoint schema is valid");
    let program = ApplicationProgramAuthoring::<ValueSourceSchema, ValueSourceProgram>::begin()
        .with_schema(&declaration)
        .validated_program()
        .expect("the checkpoint program is complete");
    match checkpoint {
        Some(checkpoint) => application_installation::in_memory_program_from_checkpoint(
            program,
            declaration,
            configuration,
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            support::limits(
                32,
                // Window <= history is enforced by primary_graph/bootstrap/preparation.rs:91.
                support::invalidation(),
            ),
            checkpoint,
        )
        .expect("the checkpoint restores"),
        None => application_installation::in_memory_program(
            program,
            declaration,
            configuration,
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            support::limits(
                32,
                // Window <= history is enforced by primary_graph/bootstrap/preparation.rs:91.
                support::invalidation(),
            ),
            |_phase, graph, installed| {
                let principal = installed
                    .principal_binding(ConsumerPrincipalBinding::reference::<ValueSourceSchema>())
                    .expect("the principal mapping is installed");
                graph.bind_principal(
                    &principal,
                    primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                    1_u64,
                    support::external_identity(),
                    WorthQueryPrincipalMappingStatus::Enabled,
                )?;
                super::super::source_world::seed(graph);
                Ok(())
            },
        )
        .expect("the checkpoint source installs"),
    }
}
