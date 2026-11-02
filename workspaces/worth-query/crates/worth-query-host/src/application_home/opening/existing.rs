//! Existing artifacts readmit their actual Query publication, never initialization.
use super::*;
pub(super) fn finish_existing_home<Schema, Program>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    mut home: UnsharedReadmittedApplicationHome,
    query: PreparedHomeNativeMeaning<Schema, Program>,
    checkpoint_limits: ApplicationHomeCheckpointLimits,
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
    let (native, authority, outcome) = match home.recovered_query_finalization_parts() {
        Ok(parts) => parts,
        Err(cause) => {
            return Err(Refused::NativeFinalizationInput {
                query,
                home,
                cause: ApplicationHomeOpenRefusal::Relational(cause),
            })
        }
    };
    let query =
        match query.finish_existing_home_native_readmission(phase, native, authority, outcome) {
            Ok(query) => query,
            Err(query) => return Err(Refused::ExistingNativeFinalization { query, home }),
        };
    let bridge = match query.home_bridge_definition() {
        Ok(bridge) => bridge,
        Err(cause) => return Err(Refused::ExistingBridgeDefinition { query, home, cause }),
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
        Err(home) => return Err(Refused::ExistingWorldReadmission { query, home }),
    };
    let (source, bridge, conditional) = match home.query_composition_parts() {
        Ok(parts) => parts,
        Err(cause) => return Err(Refused::ExistingComposition { query, home, cause }),
    };
    let query =
        match query.attach_existing_home_registered_source(phase, source, bridge, conditional) {
            Ok(query) => query,
            Err(query) => return Err(Refused::ExistingSourceAttachment { query, home }),
        };
    let mut home = home;
    let maintenance = match super::super::maintenance::prepare(&home, checkpoint_limits, &request) {
        Ok(operation) => operation,
        Err(cause) => return Err(Refused::ExistingCheckpointMaintenance { query, home, cause }),
    };
    let root = match home.select_query_root(&request) {
        Ok(root) => root,
        Err(cause) => return Err(Refused::ExistingWorldRoot { query, home, cause }),
    };
    let ports = home.world();
    let query = match query.attach_existing_home_world(
        phase,
        ports.publication_port(),
        ports.branch_port(),
        ports.observation_port(),
        ports.inspection_port(),
        ports.recovery_port(),
        root,
        directory_requirement_refusal,
        maintenance,
    ) {
        Ok(query) => query.consume(|query| query),
        Err(query) => return Err(Refused::ExistingQueryAttachment { query, home }),
    };
    super::service::finish_service(phase, query, home, checkpoint_limits)
}
