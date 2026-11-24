//! Real Directory Home open, close, and reopen for selected consumer programs.
use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationProgramDefinition, ValidatedApplicationProgram,
};
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;

pub(in crate::application_invariant_acceptance) fn unique_directory(
    label: &str,
) -> std::path::PathBuf {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "worth-consumer-root-{label}-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
    ))
}

fn directory_policy() -> worth_foundational::ExecutionRequestPolicy {
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            Some(256 << 20),
            None,
        ),
    )
}

pub(in crate::application_invariant_acceptance) fn open_directory_selected_program<P>(
    directory: &std::path::Path,
    foreign: &domain::WorthQueryInstalledApplicationSchema<ConsumerSchema>,
    program: ValidatedApplicationProgram<ConsumerSchema, P>,
) -> worth_query_host::facade::WorthQueryApplicationHome<ConsumerSchema, P>
where
    P: ApplicationProgramDefinition<
        ConsumerSchema,
        Contributions = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions,
    >,
    P::Outputs: installation::WorthQueryApplicationProgramRoots<ConsumerSchema>,
{
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
    let deployment_scope = *program.revision();
    worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        directory_policy(),
        |phase| {
            worth_query_host::facade::WorthQueryApplicationHomeRequest::new(
                worth_query_host::facade::WorthQueryApplicationHomeLocation::Directory(
                    directory.to_owned(),
                ),
                program,
                installation::WorthQueryApplicationProgramRoster::new(),
                <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().unwrap(),
                configuration,
                ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
                resources::application_limits(),
                |bootstrap_phase, graph, installed| {
                    reject_foreign_invariant_factory(installed, foreign);
                    let principal = installed
                        .principal_binding(ConsumerPrincipalBinding::reference::<ConsumerSchema>())
                        .expect("the contributed principal mapping is installed");
                    graph.bind_principal(
                        &principal,
                        primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner")
                            .unwrap(),
                        1_u64,
                        authentication::external_identity(),
                        WorthQueryPrincipalMappingStatus::Enabled,
                    )?;
                    crate::program_deployment::bind_scope(graph, deployment_scope)?;
                    graph.bind_principal(
                        &principal,
                        primary_graph::WorthQueryApplicationPrincipalKey::new("sibling-owner")
                            .unwrap(),
                        2_u64,
                        worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
                            "https://consumer.invalid/local",
                            "sibling-owner",
                        )
                        .unwrap(),
                        WorthQueryPrincipalMappingStatus::Enabled,
                    )?;
                    seed::seed_cycles_with(bootstrap_phase, graph, |graph| {
                        worth_query_topology_entry::bind_retention_grants(
                            graph,
                            "model-owner",
                            "anchor-a",
                        )
                        .unwrap();
                        worth_query_topology_entry::bind_navigation_grant(
                            graph,
                            "model-owner",
                            "anchor-a",
                        )
                        .unwrap();
                        worth_query_topology_entry::bind_navigation_grant(
                            graph,
                            "model-owner",
                            "remote-a",
                        )
                        .unwrap();
                        worth_query_topology_entry::bind_navigation_grant(
                            graph,
                            "sibling-owner",
                            "anchor-a",
                        )
                        .unwrap();
                    });
                    Ok(())
                },
            )
            .open_in_advancement(&phase)
        },
    )
    .expect("directory Home opening request remains live")
    .unwrap_or_else(|refused| panic!("directory Home open refused: {refused:?}"))
}

pub(in crate::application_invariant_acceptance) fn close_directory_program<P: 'static>(
    home: worth_query_host::facade::WorthQueryApplicationHome<ConsumerSchema, P>,
) {
    worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        directory_policy(),
        |phase| {
            home.close_in_advancement(&phase)
                .unwrap_or_else(|refused| panic!("directory Home close refused: {refused:?}"))
                .reclaim();
        },
    )
    .expect("directory Home closing request remains live");
}
