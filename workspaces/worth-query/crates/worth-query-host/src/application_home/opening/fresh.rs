//! Fresh Home consumes its one real initialization opportunity.
use super::*;
pub(super) fn finish_new_home<Schema, Program>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    home: UnsharedReadmittedApplicationHome,
    query: PreparedHomeNativeMeaning<Schema, Program>,
    initial_state: impl FnOnce(
        &WorthQueryBootstrapAdvancementPhase<'_>,
        &mut worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphBootstrap<Schema>,
        &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<
        (),
        worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    >,
    checkpoint_limits: ApplicationHomeCheckpointLimits,
    observation: ApplicationHomeOpeningObservation,
) -> Result<
    WorthQueryApplicationHome<Schema, Program>,
    WorthQueryApplicationHomeOpenRefused<Schema, Program>,
>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
{
    use WorthQueryApplicationHomeOpenRefused as Refused;
    use WorthQueryApplicationHomeOpenRefusedPhase as Phase;
    let (mut query, home) = prepare_source(phase, home, query, &observation)?;
    // Only the user initializer is caught by this borrowed Query operation;
    // the actual Home and Query bootstrap remain outside its unwind boundary.
    if let Err(cause) = query.initialize_home_query_state(phase, initial_state) {
        return Err(Refused::from_phase(Phase::InitialState {
            query,
            home,
            cause,
        }));
    }
    if let Err(cause) = query.publish_home_query_seed(phase) {
        return Err(Refused::from_phase(Phase::SeedPublication {
            query,
            home,
            cause,
        }));
    }
    finish_world(phase, query, home, checkpoint_limits, observation)
}

#[inline(never)]
fn prepare_source<Schema, Program>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    home: UnsharedReadmittedApplicationHome,
    query: PreparedHomeNativeMeaning<Schema, Program>,
    observation: &ApplicationHomeOpeningObservation,
) -> Result<
    (PreparedHomeQueryRuntime<Schema, Program>, DirectoryHome),
    WorthQueryApplicationHomeOpenRefused<Schema, Program>,
>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
{
    let (query, source, builder, bindings) = prepare_native_and_signal(phase, home, query)?;
    readmit_world_and_attach_signal(phase, query, source, builder, bindings, observation)
}

#[inline(never)]
fn prepare_native_and_signal<Schema, Program>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    mut home: UnsharedReadmittedApplicationHome,
    query: PreparedHomeNativeMeaning<Schema, Program>,
) -> Result<
    (
        PreparedHomeSignalProgram<Schema, Program>,
        PreparedQueryHomeSource,
        Box<worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder>,
        QueryHomeAuthoredBindings,
    ),
    WorthQueryApplicationHomeOpenRefused<Schema, Program>,
>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
{
    use WorthQueryApplicationHomeOpenRefused as Refused;
    use WorthQueryApplicationHomeOpenRefusedPhase as Phase;
    let request = phase.execution_request();
    let (native, admission) = match home.fresh_query_finalization_parts() {
        Ok(parts) => parts,
        Err(cause) => {
            return Err(Refused::from_phase(Phase::NativeFinalizationInput {
                query,
                home,
                cause: ApplicationHomeOpenRefusal::Relational(cause),
            }))
        }
    };
    let query = match query.finish_fresh_home_native_readmission(phase, native, admission) {
        Ok(query) => query,
        Err(query) => {
            return Err(Refused::from_phase(Phase::NativeFinalization {
                query,
                home,
            }))
        }
    };
    let bridge = match query.home_bridge_definition() {
        Ok(bridge) => bridge,
        Err(cause) => {
            return Err(Refused::from_phase(Phase::BridgeDefinition {
                query,
                home,
                cause,
            }))
        }
    };
    let (budgets, clock, policy, execution, _conditionals) = query.home_world_resources();
    let definition =
        ApplicationHomeWorldDefinition::new(bridge, "primary", budgets, clock, policy, execution);
    let source = match prepare_query_home_source(home.share(), definition, &request) {
        Ok(source) => source,
        Err(home) => {
            return Err(Refused::from_phase(Phase::WorldSourcePreparation {
                query,
                home,
            }))
        }
    };
    let (registered, ordinary) = source.query_source_parts();
    let (query, builder, bindings) =
        match query.prepare_home_signal_program(phase, registered, ordinary) {
            Ok(parts) => parts,
            Err(query) => {
                return Err(Refused::from_phase(Phase::SignalProgramPreparation {
                    home: source,
                    query: Box::new(query),
                }))
            }
        };
    Ok((query, source, builder, bindings))
}

#[inline(never)]
fn readmit_world_and_attach_signal<Schema, Program>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    query: PreparedHomeSignalProgram<Schema, Program>,
    source: PreparedQueryHomeSource,
    builder: Box<worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder>,
    mut bindings: QueryHomeAuthoredBindings,
    observation: &ApplicationHomeOpeningObservation,
) -> Result<
    (PreparedHomeQueryRuntime<Schema, Program>, DirectoryHome),
    WorthQueryApplicationHomeOpenRefused<Schema, Program>,
>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
{
    use WorthQueryApplicationHomeOpenRefused as Refused;
    use WorthQueryApplicationHomeOpenRefusedPhase as Phase;
    let request = phase.execution_request();
    let signal = match readmit_query_signal(source, builder, &mut bindings, &request) {
        Ok(signal) => signal,
        Err(home) => {
            return Err(Refused::from_phase(Phase::WorldReadmission {
                query,
                home,
                _bindings: bindings,
            }))
        }
    };
    let home = match readmit_query_world(signal, &request) {
        Ok(home) => home.consume(|home| home),
        Err(home) => {
            return Err(Refused::from_phase(Phase::WorldReadmission {
                query,
                home,
                _bindings: bindings,
            }))
        }
    };
    let (_, _, conditional) = match home.query_composition_parts() {
        Ok(parts) => parts,
        Err(cause) => {
            return Err(Refused::from_phase(Phase::Composition {
                query,
                home,
                cause,
            }))
        }
    };
    observation.arrive_for_test(
        ApplicationHomeOpeningStage::QueryReadmission,
        ApplicationHomeOpeningEdge::Before,
    );
    let query = match query.attach_home_authored_signal(conditional) {
        Ok(query) => query,
        Err(query) => return Err(Refused::from_phase(Phase::SourceAttachment { query, home })),
    };
    Ok((query, home))
}

#[inline(never)]
fn finish_world<Schema, Program>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    query: PreparedHomeQueryRuntime<Schema, Program>,
    mut home: DirectoryHome,
    checkpoint_limits: ApplicationHomeCheckpointLimits,
    observation: ApplicationHomeOpeningObservation,
) -> Result<
    WorthQueryApplicationHome<Schema, Program>,
    WorthQueryApplicationHomeOpenRefused<Schema, Program>,
>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
{
    use WorthQueryApplicationHomeOpenRefused as Refused;
    use WorthQueryApplicationHomeOpenRefusedPhase as Phase;
    let request = phase.execution_request();
    let maintenance = match super::super::maintenance::prepare(&home, checkpoint_limits, &request) {
        Ok(operation) => operation,
        Err(cause) => {
            return Err(Refused::from_phase(Phase::CheckpointMaintenance {
                query,
                home,
                cause,
            }))
        }
    };
    let root = match home.select_query_root(&request) {
        Ok(root) => root,
        Err(cause) => return Err(Refused::from_phase(Phase::WorldRoot { query, home, cause })),
    };
    let ports = home.world();
    let query = match query.attach_home_world(
        phase,
        ports.publication_port(),
        ports.branch_port(),
        ports.observation_port(),
        ports.inspection_port(),
        ports.recovery_port(),
        root,
        maintenance,
        home.branch_program_source_owner(),
    ) {
        Ok(query) => query.consume(|query| query),
        Err(query) => return Err(Refused::from_phase(Phase::QueryAttachment { query, home })),
    };
    observation.arrive_for_test(
        ApplicationHomeOpeningStage::QueryReadmission,
        ApplicationHomeOpeningEdge::After,
    );
    super::service::finish_service(
        phase,
        query,
        home,
        checkpoint_limits,
        WorthQueryHomeOpenDisposition::Created,
        observation,
    )
}
