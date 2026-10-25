//! Cost-specific bootstrap population, under the unchanged finite consumer profiles.
use super::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc,
};
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::{
    application_installation::in_memory_program,
    declaration::authentication::WorthQueryPrincipalMappingStatus,
    primary_graph::WorthQueryApplicationPrincipalKey,
};
pub(super) fn install(fanout: usize, unrelated: usize) -> Runtime {
    let configuration = (
        TopologyConfiguration {
            final_feature: worth_query_topology_entry::planar_final_feature_runtime(),
            setup_calls: Arc::new(AtomicUsize::new(0)),
            invariant_calls: Arc::new(AtomicUsize::new(0)),
            invariant_probe: Arc::new(AtomicUsize::new(0)),
            invariant_read_observer: None,
            producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
            producer_domain_denial: Arc::new(AtomicBool::new(false)),
        },
        Arc::new(AtomicUsize::new(0)),
        (),
    );
    let program = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .with_schema(&Schema::declaration().expect("the completed schema is valid"))
        .validated_program()
        .unwrap();
    in_memory_program(
        program,
        Schema::declaration().unwrap(),
        configuration,
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        resources::application_limits(),
        |phase, graph, installed| {
            let binding = installed
                .principal_binding(ConsumerPrincipalBinding::reference::<Schema>())
                .unwrap();
            graph.bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                1u64,
                authentication::external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            crate::application_invariant_acceptance::seed::seed_cycles(phase, graph);
            population::seed(phase, graph, fanout, unrelated);
            Ok(())
        },
    )
    .expect("cost meaning installs through the real native host")
}
