use super::{AdmissionTarget, AdmissionWorld, TARGET_POINT};
use crate::filesystem_mounted_world::{component_graph_nodes, establish_allocation};
use crate::intent::operability::{
    build_scoped, build_scoped_with_provider, build_scoped_with_provider_observation,
    OccupancyLayout, OperabilityFacts, PrimaryIntent,
};
use crate::mounted_application_lifecycle::known_empty_surface_world::profile;
use crate::mounted_application_lifecycle::published_mounted_world::presented_epoch;
use worth_ui::facade::app::{WorthUiActiveApplicationSession, WorthUiApp};
use worth_ui::facade::observation_report::UiHostObservationPresentationBasis;
use worth_ui_dsl::WorthUiRustAuthoredArtifactInput;
use worth_ui_runtime::facade::{
    entry::UiNativeComponentSemanticTextChange,
    mounted::{
        UiHostSurfacePresentationMode, UiMountedFrameOutcome, UiMountedFrameRequest,
        UiMountedInstanceIdentity, UiPresentationDeadline, UiSurfaceBindingGeneration,
    },
};
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiMountedIdentityCertificationExt,
    WorthUiMountedPublicationCertificationExt,
};

enum AdmissionSurfaces<'declaration> {
    IndependentPages(usize),
    DeclaredPage(&'declaration str),
}

impl AdmissionWorld {
    pub(in crate::intent) fn launch(target_count: usize) -> Self {
        assert!(target_count > 0, "an admission world needs one real target");
        let (application, facts) = build_scoped(OccupancyLayout::TargetRoute);
        Self::launch_application(application, facts, target_count)
    }

    pub(in crate::intent) fn launch_with_provider_observation(
        target_count: usize,
    ) -> (
        Self,
        worth_ui_certification::WorthUiCertificationProviderObservation,
    ) {
        assert!(target_count > 0, "an admission world needs one real target");
        let (application, facts, observation) =
            build_scoped_with_provider_observation(OccupancyLayout::TargetRoute);
        (
            Self::launch_application(application, facts, target_count),
            observation,
        )
    }

    pub(in crate::intent) fn launch_with_provider<P>(target_count: usize, provider: P) -> Self
    where
        P: worth_ui::facade::intent::UiIntentExecutionProvider<PrimaryIntent>,
    {
        assert!(target_count > 0, "an admission world needs one real target");
        let (application, facts) =
            build_scoped_with_provider(OccupancyLayout::TargetRoute, provider);
        Self::launch_application(application, facts, target_count)
    }

    pub(in crate::intent) fn launch_application(
        application: worth_ui::facade::app::WorthUiApp,
        facts: OperabilityFacts,
        target_count: usize,
    ) -> Self {
        Self::launch_application_with_routed_component(application, facts, target_count, 1)
    }

    pub(in crate::intent) fn launch_application_with_routed_component(
        application: worth_ui::facade::app::WorthUiApp,
        facts: OperabilityFacts,
        target_count: usize,
        routed_component_index: usize,
    ) -> Self {
        Self::launch_application_with_target(
            application,
            facts,
            target_count,
            routed_component_index,
            TARGET_POINT,
        )
    }

    pub(in crate::intent) fn launch_application_with_target(
        application: worth_ui::facade::app::WorthUiApp,
        facts: OperabilityFacts,
        target_count: usize,
        routed_component_index: usize,
        target_point: [i64; 2],
    ) -> Self {
        let replacement_input = crate::intent::operability::replacement_input(&facts);
        Self::launch_pages(
            application,
            facts,
            AdmissionSurfaces::IndependentPages(target_count),
            routed_component_index,
            target_point,
            replacement_input,
            &[],
        )
    }

    pub(in crate::intent) fn launch_application_on_declared_surface(
        application: WorthUiApp,
        facts: OperabilityFacts,
        declared_surface: &str,
        routed_component_index: usize,
        target_point: [i64; 2],
    ) -> Self {
        let replacement_input = crate::intent::operability::replacement_input(&facts);
        Self::launch_pages(
            application,
            facts,
            AdmissionSurfaces::DeclaredPage(declared_surface),
            routed_component_index,
            target_point,
            replacement_input,
            &[],
        )
    }

    pub(in crate::intent) fn launch_application_on_declared_surface_with_replacement(
        application: WorthUiApp,
        facts: OperabilityFacts,
        declared_surface: &str,
        routed_component_index: usize,
        target_point: [i64; 2],
        replacement_input: WorthUiRustAuthoredArtifactInput,
    ) -> Self {
        Self::launch_pages(
            application,
            facts,
            AdmissionSurfaces::DeclaredPage(declared_surface),
            routed_component_index,
            target_point,
            replacement_input,
            &[],
        )
    }

    pub(in crate::intent) fn launch_application_on_declared_surface_with_semantic_text(
        application: WorthUiApp,
        facts: OperabilityFacts,
        declared_surface: &str,
        routed_component_index: usize,
        target_point: [i64; 2],
        changes: &[UiNativeComponentSemanticTextChange],
    ) -> Self {
        let replacement_input = crate::intent::operability::replacement_input(&facts);
        Self::launch_pages(
            application,
            facts,
            AdmissionSurfaces::DeclaredPage(declared_surface),
            routed_component_index,
            target_point,
            replacement_input,
            changes,
        )
    }

    fn launch_pages(
        application: WorthUiApp,
        facts: OperabilityFacts,
        surfaces: AdmissionSurfaces<'_>,
        routed_component_index: usize,
        target_point: [i64; 2],
        replacement_input: WorthUiRustAuthoredArtifactInput,
        changes: &[UiNativeComponentSemanticTextChange],
    ) -> Self {
        let nodes = component_graph_nodes(&application);
        assert!(routed_component_index < nodes.len());
        let mut session = application
            .launch()
            .expect("admission application launches");
        if !changes.is_empty() {
            session
                .register_and_apply_component_semantic_text(changes)
                .expect("semantic text is admitted before initial publication");
        }
        let surfaces = match surfaces {
            AdmissionSurfaces::IndependentPages(count) => (0..count)
                .map(|_| {
                    session
                        .create_semantic_surface()
                        .expect("an ordinary page creates its surface")
                })
                .collect::<Vec<_>>(),
            AdmissionSurfaces::DeclaredPage(name) => vec![session
                .create_declared_semantic_surface(name)
                .expect("the declared page binds its authored surface")],
        };
        let target_count = surfaces.len();
        let mounted = mount_complete_pages(&mut session, &nodes, surfaces, routed_component_index);
        establish_allocation(&mut session, 3);
        crate::mounted_geometry_fixture::install_current_occurrence_geometry(&mut session);
        let prepared = session
            .prepare_application_presentation_frame(UiMountedFrameRequest::all_bound_surfaces())
            .expect("application presentation prepares the admission frame");
        assert_eq!(prepared.surfaces().len(), target_count);
        let publication = match session.present_prepared_mounted_frame(
            prepared,
            UiPresentationDeadline::at_tick(1_000),
            0,
        ) {
            UiMountedFrameOutcome::Published(publication) => publication,
            _ => panic!("admission frame must publish"),
        };
        let targets = mounted
            .into_iter()
            .map(|(binding, mounted_instance)| AdmissionTarget {
                presentation: presentation(&session, publication.frame(), binding),
                mounted_instance,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self {
            session,
            facts,
            targets,
            next_pointer: 1,
            next_sequence: 1,
            target_point,
            replacement_input,
        }
    }
}

fn mount_complete_pages(
    session: &mut WorthUiActiveApplicationSession,
    nodes: &[worth_ui::facade::graph::UiGraphNodeIdentity],
    surfaces: Vec<worth_ui_host_contract::UiSemanticSurfaceIdentity>,
    routed_component_index: usize,
) -> Vec<(UiSurfaceBindingGeneration, UiMountedInstanceIdentity)> {
    surfaces
        .into_iter()
        .enumerate()
        .map(|(index, surface)| {
            let binding = session
                .register_host_surface(
                    surface,
                    UiHostSurfacePresentationMode::RecordOnly,
                    profile(index as u64 + 1),
                )
                .unwrap()
                .binding_generation();
            let mut routed_instance = None;
            for (node_index, graph_node) in nodes.iter().copied().enumerate() {
                let handle = session.mounted_graph_node(graph_node).unwrap();
                let mounted = session.mount_instance(handle, surface).unwrap();
                if node_index == routed_component_index {
                    routed_instance = Some(mounted);
                }
            }
            let routed_instance =
                routed_instance.expect("the complete page includes its routed hit-only component");
            (binding, routed_instance)
        })
        .collect()
}

fn presentation(
    session: &WorthUiActiveApplicationSession,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    binding: UiSurfaceBindingGeneration,
) -> UiHostObservationPresentationBasis {
    let host_surface = session
        .inspect_mounted_identity()
        .surface_bindings()
        .iter()
        .find(|candidate| candidate.binding_generation() == binding)
        .expect("the target binding remains current")
        .host_surface_identity();
    UiHostObservationPresentationBasis::new(
        host_surface,
        frame,
        binding,
        presented_epoch(session, frame, binding),
    )
}
