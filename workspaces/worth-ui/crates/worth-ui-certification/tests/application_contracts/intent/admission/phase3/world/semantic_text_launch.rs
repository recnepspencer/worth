use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiMountedPublicationCertificationExt,
};

use super::{AdmissionTarget, AdmissionWorld};

impl AdmissionWorld {
    pub(in crate::intent) fn launch_application_with_target_and_semantic_text(
        application: worth_ui::facade::app::WorthUiApp,
        facts: super::OperabilityFacts,
        target_count: usize,
        routed_component_index: usize,
        target_point: [i64; 2],
        changes: &[worth_ui_runtime::facade::entry::UiNativeComponentSemanticTextChange],
    ) -> Self {
        let nodes = super::component_graph_nodes(&application);
        assert!(routed_component_index < nodes.len());
        let mut session = application
            .launch()
            .expect("admission application launches");
        session
            .register_and_apply_component_semantic_text(changes)
            .expect("authored semantic text is admitted before the initial mounted frame");
        let mounted =
            super::mount_complete_pages(&mut session, &nodes, target_count, routed_component_index);
        super::establish_allocation(&mut session, 3);
        crate::mounted_geometry_fixture::install_current_occurrence_geometry(&mut session);
        let prepared = session
            .prepare_application_presentation_frame(
                worth_ui_runtime::facade::mounted::UiMountedFrameRequest::all_bound_surfaces(),
            )
            .expect("application presentation prepares the admission frame");
        assert_eq!(prepared.surfaces().len(), target_count);
        let publication = match session.present_prepared_mounted_frame(
            prepared,
            worth_ui_runtime::facade::mounted::UiPresentationDeadline::at_tick(1_000),
            0,
        ) {
            worth_ui_runtime::facade::mounted::UiMountedFrameOutcome::Published(publication) => {
                publication
            }
            _ => panic!("admission frame must publish"),
        };
        let targets = mounted
            .into_iter()
            .map(|(binding, mounted_instance)| AdmissionTarget {
                presentation: super::presentation(&session, publication.frame(), binding),
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
        }
    }
}
