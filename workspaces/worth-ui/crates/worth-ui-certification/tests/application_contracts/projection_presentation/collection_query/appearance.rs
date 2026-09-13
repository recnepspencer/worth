use worth_ui::facade::declaration::{ComponentSemanticTextContract, ThemeTokenId};
use worth_ui_dsl::WorthUiRustAuthoredArtifactInput;
use worth_ui_host_contract::UiSemanticTextSlot;
use worth_ui_host_headless::{
    UiHeadlessAppearanceMechanic, UiHeadlessRecorderCapacity, UiHeadlessUnperformedEffect,
    WorthUiHeadlessRecorder,
};
use worth_ui_query_binding::{UiCollectionProjectionRegistration, WorthUiQueryWorkspaceExt};

use super::{
    collection_module, component_descriptor, status_region_descriptor, text_token_descriptor,
    ACTIVE_COMPONENT, TEXT_COLOR,
};

pub(crate) fn application(
    registration: UiCollectionProjectionRegistration,
    recorder: WorthUiHeadlessRecorder,
) -> worth_ui::facade::app::WorthUiApp {
    let role = super::super::query_text_appearance::white_role();
    let component = component_descriptor(ACTIVE_COMPONENT)
        .with_semantic_text(
            ComponentSemanticTextContract::body_default(ThemeTokenId::new(TEXT_COLOR).unwrap(), 1)
                .with_appearance_foreground(),
        )
        .with_appearance_aspect_contract(role.aspect_contract().clone())
        .unwrap();
    let module =
        super::super::query_text_appearance::attach(collection_module(), ACTIVE_COMPONENT, false);
    worth_ui::facade::app::WorthUi::app()
        .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(component)
        .register_mosaic_region_kind(status_region_descriptor())
        .register_theme_token(text_token_descriptor())
        .register_collection_projection(registration)
        .expect("product collection projection registers")
        .register_appearance_role(role)
        .unwrap()
        .register_appearance_role(super::super::query_text_appearance::green_role())
        .unwrap()
        .register_appearance_theme_bundle(super::super::query_text_appearance::theme())
        .unwrap()
        .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([module]))
        .freeze()
        .map(|application| {
            worth_ui_runtime::facade::entry::WorthUiCertificationApplicationTransition::activate_recorder(
                application,
                recorder,
            )
        })
        .expect("appearance collection content application freezes")
}

#[test]
fn empty_collection_first_publication_paints_its_current_posture() {
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::production_default(),
        worth_ui::facade::measurement_exchange::UiViewportExtentObservation {
            width: 320.0,
            height: 128.0,
        },
    );
    let (mut workspace, entities) =
        worth_ui_query_binding::certification::seeded_collection_projection_workspace(
            Vec::new(),
            worth_ui_query_binding::certification::WorthUiCollectionProjectionSeedPosture::Complete,
        );
    assert!(entities.is_empty());
    let domain = workspace.worth_ui().expect("Worth UI domain installed");
    let registration = super::collection_registration(&domain);
    let mut session = application(registration.clone(), recorder.clone())
        .launch()
        .expect("empty collection appearance application launches");
    let mounted_instances = super::mount_and_allocate(&mut session);
    let opened = super::open_live_collection(registration, &mut workspace);
    let (live, snapshot) = opened.into_parts();

    super::publish_collection_fact(&mut session, snapshot.into_observation(), 3136);

    let transcripts = recorder.observed_transcripts();
    assert_eq!(transcripts.len(), 1);
    let transcript = &transcripts[0];
    assert_eq!(transcript.semantic_text().len(), 1);
    let posture = &transcript.semantic_text()[0];
    assert_eq!(posture.slot(), UiSemanticTextSlot::Posture);
    assert_eq!(posture.text(), "CURRENT · COMPLETE");
    assert!(mounted_instances.contains(&posture.mounted_instance()));
    assert_eq!(posture.foregrounds().len(), 1);
    let appearance = transcript
        .appearance_work()
        .expect("the empty collection posture publishes resolved appearance");
    let foregrounds = appearance
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.work().successor().mechanics())
        .map(|mechanic| match mechanic {
            UiHeadlessAppearanceMechanic::TextForeground(foreground) => foreground,
            _ => panic!("the collection role declares only text foreground"),
        })
        .collect::<Vec<_>>();
    assert_eq!(foregrounds.len(), 1);
    let foreground = foregrounds[0];
    assert_eq!(foreground.command(), posture.command_identity());
    assert_eq!(foreground.paint_span(), posture.foregrounds()[0].identity());
    assert_eq!(foreground.node_receipt(), posture.node_receipt());
    assert_eq!(
        foreground.foreground().straight_srgba(),
        [255, 255, 255, 255]
    );
    assert_eq!(foreground.opacity().units(), 65_535);
    assert_eq!(
        transcript.unperformed_effects(),
        &[UiHeadlessUnperformedEffect::NativePaint {
            appearance_mechanic_count: 1,
            portal_overlay_count: 0,
            semantic_text_count: 1,
            preview_node_count: 0,
        }]
    );

    super::close_collection(live, &mut workspace);
    let shutdown = session.shutdown();
    assert!(shutdown.rebind().is_empty());
    assert!(shutdown.mounted_presentation().is_empty());
}
