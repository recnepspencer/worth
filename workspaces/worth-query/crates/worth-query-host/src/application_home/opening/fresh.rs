//! Fresh Home consumes its one real initialization opportunity.
use super::*;
pub(super) fn finish_new_home<Schema, Program>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    mut home: UnsharedReadmittedApplicationHome,
    query: PreparedHomeNativeMeaning<Schema, Program>,
    initial_state: impl FnOnce(
        &WorthQueryBootstrapAdvancementPhase<'_>,
        &mut worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphBootstrap<Schema>,
        &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<
        (),
        worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    >,
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
    let request = phase.execution_request();
    let (native, authority, outcome) = home.query_finalization_parts();
    let query = match query.finish_fresh_home_native_readmission(phase, native, authority, outcome)
    {
        Ok(query) => query,
        Err(query) => return Err(Refused::NativeFinalization { query, home }),
    };
    let bridge = match query.home_bridge_definition() {
        Ok(bridge) => bridge,
        Err(cause) => return Err(Refused::BridgeDefinition { query, home, cause }),
    };
    let (budgets, clock, policy, execution, conditionals) = query.home_world_resources();
    let definition = ApplicationHomeWorldDefinition::new(
        bridge,
        "primary-application-source",
        budgets,
        clock,
        policy,
        execution,
        conditionals,
    );
    let home = match readmit_query_world(home.share(), definition, &request) {
        Ok(home) => home.consume(|home| home),
        Err(home) => return Err(Refused::WorldReadmission { query, home }),
    };
    let (source, bridge, conditional) = match home.query_composition_parts() {
        Ok(parts) => parts,
        Err(cause) => return Err(Refused::Composition { query, home, cause }),
    };
    let mut query = match query.attach_home_registered_source(phase, source, bridge, conditional) {
        Ok(query) => query,
        Err(query) => return Err(Refused::SourceAttachment { query, home }),
    };
    // Only the user initializer is caught by this borrowed Query operation;
    // the actual Home and Query bootstrap remain outside its unwind boundary.
    if let Err(cause) = query.initialize_home_query_state(phase, initial_state) {
        return Err(Refused::InitialState { query, home, cause });
    }
    if let Err(cause) = query.publish_home_query_seed(phase) {
        return Err(Refused::SeedPublication { query, home, cause });
    }
    let mut home = home;
    let root = match home.select_query_root(&request) {
        Ok(root) => root,
        Err(cause) => return Err(Refused::WorldRoot { query, home, cause }),
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
        directory_requirement_refusal,
    ) {
        Ok(query) => query.consume(|query| query),
        Err(query) => return Err(Refused::QueryAttachment { query, home }),
    };
    super::service::finish_service(phase, query, home)
}
