//! The real existing-home Signal phase, before serving Query attachment.
use super::*;
pub(super) fn existing_signal_program(
    query: NativePreparation,
    mut home: UnsharedReadmittedApplicationHome,
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
) -> (
    PreparedExistingHomeSignalProgram<HomeIdentitySchema, HomeProgram>,
    DirectoryHome,
    worth_runtime_bridge::facade::RuntimeBridgeRelationalSource,
) {
    let execution = phase.execution_request();
    let query = finalize_native(query, &mut home, phase).unwrap_or_else(|refused| {
        panic!(
            "actual existing Native finalization: {:?}",
            refused.reason()
        )
    });
    let bridge = query.home_bridge_definition().unwrap();
    let (budgets, clock, policy, execution_authority, _conditionals) = query.home_world_resources();
    let source = prepare_query_home_source(
        home.share(),
        ApplicationHomeWorldDefinition::new(
            bridge,
            "primary-application-source",
            budgets,
            clock,
            policy,
            execution_authority,
        ),
        &execution,
    )
    .unwrap_or_else(|_| panic!("actual source admission"));
    let (registered, ordinary) = source.query_source_parts();
    let witness = registered.clone();
    let (query, builder, mut bindings) = query
        .prepare_existing_home_signal_program(phase, registered, ordinary)
        .unwrap_or_else(|refused| panic!("actual existing programme: {:?}", refused.reason()));
    let signal = readmit_query_signal(source, builder, &mut bindings, &execution)
        .unwrap_or_else(|refused| panic!("actual existing Signal: {:?}", refused.reason()));
    let home = readmit_query_world(signal, &execution)
        .unwrap_or_else(|refused| panic!("actual existing World: {:?}", refused.reason()))
        .consume(|home| home);
    (query, home, witness)
}
