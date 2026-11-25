#![allow(
    clippy::result_large_err,
    reason = "The fixture drives the real installation and Home owners, whose refusals hand back the original owner"
)]
//! An isolated schema admits the replacement conditional without changing other programs.
use super::*;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc,
};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

pub(super) fn install(
    directory: Option<PathBuf>,
) -> support::DirectoryApp<MixedSchema, MixedProgram> {
    let configuration = (TopologyConfiguration {
        final_feature: crate::planar_final_feature_runtime(),
        setup_calls: Arc::new(AtomicUsize::new(0)),
        invariant_calls: Arc::new(AtomicUsize::new(0)),
        invariant_probe: Arc::new(AtomicUsize::new(0)),
        invariant_read_observer: None,
        producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
        producer_domain_denial: Arc::new(AtomicBool::new(false)),
        producer_contacts: None,
    },);
    let declaration =
        <MixedSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration()
            .expect("the checkpoint schema is valid");
    let program = ApplicationProgramAuthoring::<MixedSchema, MixedProgram>::begin()
        .with_schema(&declaration)
        .validated_program()
        .expect("the checkpoint program is complete");
    let dir = directory.unwrap_or_else(|| support::directory::unique_directory("mixed-retire"));
    let home = support::directory::with_directory_bootstrap(|phase| {
        WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(dir.clone()),
            program,
            application_installation::WorthQueryApplicationProgramRoster::new(),
            declaration,
            configuration,
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            support::limits(32, support::invalidation()),
            |_phase, graph, installed| {
                let principal = installed
                    .principal_binding(ConsumerPrincipalBinding::reference::<MixedSchema>())
                    .expect("the principal mapping is installed");
                graph.bind_principal(
                    &principal,
                    primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner")
                        .unwrap(),
                    1_u64,
                        support::external_identity(),
                        WorthQueryPrincipalMappingStatus::Enabled,
                    )?;
                    support::seed_cycle(graph);
                    Ok(())
                },
            )
            .open_in_advancement(phase)
    })
    .unwrap_or_else(|refused| panic!("mixed directory open refused: {refused:?}"));
    support::DirectoryApp::wrap(home, dir)
}
