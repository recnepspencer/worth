//! Native installation of only the membership fixture's declared meaning.
use super::super::{authentication, resources, seed};
use super::{MembershipProgram, MembershipSchema as Schema};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc,
};
use worth_query_decl::facade::application_program::{
    ApplicationProgramAuthoring, ApplicationProgramDefinition,
};
use worth_query_host::facade::{
    application_installation::{in_memory_program, WorthQueryProgramApplicationRuntime},
    declaration::authentication::WorthQueryPrincipalMappingStatus,
    primary_graph::WorthQueryApplicationPrincipalKey,
};
use worth_query_topology_entry::{ConsumerPrincipalBinding, TopologyConfiguration};
pub(in crate::application_invariant_acceptance) struct MembershipWorld<P: worth_query_decl::facade::application_program::ApplicationProgramDefinition<Schema> = MembershipProgram> {
    pub(in crate::application_invariant_acceptance) application:
        WorthQueryProgramApplicationRuntime<Schema, P>,
}
pub(in crate::application_invariant_acceptance) fn install() -> MembershipWorld {
    install_program::<MembershipProgram>()
}
pub(in crate::application_invariant_acceptance) fn install_program<
    P: ApplicationProgramDefinition<
        Schema,
        Contributions = <MembershipProgram as ApplicationProgramDefinition<Schema>>::Contributions,
        Outputs = <MembershipProgram as ApplicationProgramDefinition<Schema>>::Outputs,
    >,
>() -> MembershipWorld<P> {
    install_program_seeded::<P>(false)
}
pub(in crate::application_invariant_acceptance) fn install_with_isolated_cycle() -> MembershipWorld
{
    install_program_seeded::<MembershipProgram>(true)
}
fn install_program_seeded<
    P: ApplicationProgramDefinition<
        Schema,
        Contributions = <MembershipProgram as ApplicationProgramDefinition<Schema>>::Contributions,
        Outputs = <MembershipProgram as ApplicationProgramDefinition<Schema>>::Outputs,
    >,
>(
    isolated_cycle: bool,
) -> MembershipWorld<P> {
    let configuration = (
        TopologyConfiguration {
            setup_calls: Arc::new(AtomicUsize::new(0)),
            invariant_calls: Arc::new(AtomicUsize::new(0)),
            invariant_probe: Arc::new(AtomicUsize::new(0)),
            producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
            producer_domain_denial: Arc::new(AtomicBool::new(false)),
        },
        Arc::new(AtomicUsize::new(0)),
        (),
    );
    let program = ApplicationProgramAuthoring::<Schema, P>::begin()
        .validated_program()
        .expect("the separate membership graph and operation authorization are complete");
    let application = in_memory_program(
        program,
        Schema::declaration().unwrap(),
        configuration,
        resources::application_limits(),
        |phase, graph, installed| {
            let principal = installed
                .principal_binding(ConsumerPrincipalBinding::reference::<Schema>())
                .expect("the topology contribution declares its principal binding");
            graph.bind_principal(
                &principal,
                WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                1u64,
                authentication::external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            seed::seed_cycles(phase, graph);
            if isolated_cycle {
                seed::seed_isolated_cycle(phase, graph);
            }
            Ok(())
        },
    )
    .expect("the separate membership contribution installs through the native application host");
    MembershipWorld { application }
}
