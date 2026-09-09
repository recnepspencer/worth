use super::*;
use crate::certification_support::{
    initial_presentation_mechanics_for_certification, semantic_text_projection_for_certification,
    UiSemanticTextProjectionCertificationMutation,
};
use worth_ui_host_contract::*;

#[test]
fn unchanged_successor_carries_empty_demand_while_physical_sample_keeps_current_frame() {
    let projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let requirement = UiMountedSurfaceBindingRequirement::new(
        projection.surface(),
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        projection.binding(),
        WorthUiHostCapabilityObservationGeneration::new(7),
        11,
        UiHostSurfacePresentationMode::NativeDisplay,
    );
    let initial = initial_presentation_mechanics_for_certification(&projection, requirement);
    let affinity = initial.affinity();
    let unchanged =
        UiMountedPresentationUnchanged::from_inert_mechanics(UiMountedPresentationUnchangedInput {
            predecessor: affinity.successor(),
            successor: UiMountedFrameIdentity::mint_unbound().unwrap(),
            surface: affinity.surface(),
            binding: affinity.binding(),
            content: affinity.content(),
            baseline: affinity.baseline(),
            production_cost: Default::default(),
        });
    let dpi = UiMountedEventTimeDpiAuthority::from_requirement(requirement).unwrap();
    let Some(UiNativeTextPresentationPreparation::Prepared(prepared)) =
        prepare_mounted_semantic_text(
            UiMountedPresentationWorkView::Unchanged(&unchanged),
            dpi,
            |_| panic!("unchanged commands must not resolve layout"),
        )
    else {
        panic!("mounted successor must carry currentness through host acceptance")
    };
    assert!(!prepared.pin_set_complete());
    assert!(prepared.pin_commands().is_empty());
    assert!(prepared.pin_removals().is_empty());
    assert!(prepared.demand_batches().is_empty());
    assert!(prepared.glyph_runs().is_empty());
    assert_eq!(prepared.performed_layout_work(), [0; 17]);
    let planning = prepared.planning_inspection().unwrap();
    assert_eq!(
        (
            planning.demand_batches(),
            planning.demand_records(),
            planning.key_checks()
        ),
        (0, 0, 0)
    );

    let sample =
        UiMountedPresentationSample::from_inert_mechanics(UiMountedPresentationSampleInput {
            frame: affinity.successor(),
            surface: affinity.surface(),
            binding: affinity.binding(),
            content: affinity.content(),
            baseline: affinity.baseline(),
            changes: vec![UiMountedPresentationSampleChange::from_runtime_sampling(
                initial.commands()[0].identity(),
                None,
                UiMountedPresentationOpacity::from_runtime_composition(32_768),
            )],
            damage: vec![],
            production_cost: Default::default(),
        })
        .unwrap();
    assert!(prepare_mounted_semantic_text(
        UiMountedPresentationWorkView::Sample(&sample),
        dpi,
        |_| panic!("physical sample must not resolve layout")
    )
    .is_none());
}
