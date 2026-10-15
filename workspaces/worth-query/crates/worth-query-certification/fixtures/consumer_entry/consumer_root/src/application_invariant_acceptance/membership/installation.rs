//! Native installation of only the membership fixture's declared meaning.
use super::super::{authentication, resources, seed};
use super::{MembershipProgram, MembershipSchema as Schema};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc,
};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::{
    application_installation::{in_memory_program, WorthQueryProgramApplicationRuntime},
    declaration::authentication::WorthQueryPrincipalMappingStatus,
    primary_graph::WorthQueryApplicationPrincipalKey,
};
use worth_query_topology_entry::{ConsumerPrincipalBinding, TopologyConfiguration};
pub(in crate::application_invariant_acceptance) struct MembershipWorld {
    pub(in crate::application_invariant_acceptance) application:
        WorthQueryProgramApplicationRuntime<Schema, MembershipProgram>,
}
pub(in crate::application_invariant_acceptance) fn install() -> MembershipWorld {
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
    let program = ApplicationProgramAuthoring::<Schema, MembershipProgram>::begin()
        .validated_program()
        .expect("the separate membership graph and operation authorization are complete");
    let application = in_memory_program(
        program,
        Schema::declaration().unwrap(),
        configuration,
        resources::application_limits(8192, 4096),
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
            Ok(())
        },
    )
    .expect("the separate membership contribution installs through the native application host");
    MembershipWorld { application }
}
