//! Native checkpoint-fixture installation with caller-owned limits and seed.
use super::*;

pub(super) fn install_program_with_domain_denial<Program>(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    limits: WorthQueryInMemoryApplicationLimits,
    seed: fn(&mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>),
    domain_denial: Arc<AtomicBool>,
) -> application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, Program>
where
    Program: ApplicationProgramDefinition<
        CheckpointSchema,
        Contributions = <CheckpointSchema as ApplicationSchemaComposition>::Contributions,
    >,
    Program::Outputs: application_installation::WorthQueryApplicationProgramRoots<CheckpointSchema>,
{
    let configuration = (TopologyConfiguration {
        setup_calls: Arc::new(AtomicUsize::new(0)),
        invariant_calls: Arc::new(AtomicUsize::new(0)),
        invariant_probe: Arc::new(AtomicUsize::new(0)),
        producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
        producer_domain_denial: domain_denial,
    },);
    let program = ApplicationProgramAuthoring::<CheckpointSchema, Program>::begin()
        .validated_program()
        .expect("the checkpoint program is complete");
    let declaration = CheckpointSchema::declaration().expect("the checkpoint schema is valid");
    match checkpoint {
        Some(checkpoint) => application_installation::in_memory_program_from_checkpoint(
            program,
            declaration,
            configuration,
            limits.with_output_demand_resources(profile),
            checkpoint,
        )
        .expect("the checkpoint restores"),
        None => application_installation::in_memory_program(
            program,
            declaration,
            configuration,
            limits.with_output_demand_resources(profile),
            |graph, installed| {
                let principal = installed
                    .principal_binding(ConsumerPrincipalBinding::reference::<CheckpointSchema>())
                    .expect("the principal mapping is installed");
                graph.bind_principal(
                    &principal,
                    primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                    1_u64,
                    external_identity(),
                    WorthQueryPrincipalMappingStatus::Enabled,
                )?;
                seed(graph);
                Ok(())
            },
        )
        .expect("the checkpoint source installs"),
    }
}
