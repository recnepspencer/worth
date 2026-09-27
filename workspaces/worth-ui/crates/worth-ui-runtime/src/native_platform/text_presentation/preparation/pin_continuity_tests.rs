//! Retained receipts vouch for the pins only while every one of them holds
//! the candidate's set, however many of them share one committed set.
use std::sync::Arc;

use super::*;
use crate::mounting::qualified_text_test_support::inert_qualified_layout;
use worth_ui_host_contract::*;

#[test]
fn receipts_sharing_a_set_pass_together_and_a_later_differing_set_fails() {
    use crate::certification_support::{
        initial_presentation_mechanics_for_certification,
        semantic_text_projection_for_certification, UiSemanticTextProjectionCertificationMutation,
    };
    let projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let binding = UiMountedSurfaceBindingRequirement::new(
        projection.surface(),
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        projection.binding(),
        WorthUiHostCapabilityObservationGeneration::new(7),
        11,
        UiHostSurfacePresentationMode::NativeDisplay,
    );
    let initial = initial_presentation_mechanics_for_certification(&projection, binding);
    let layout = inert_qualified_layout("ONLINE");
    let dpi = UiMountedEventTimeDpiAuthority::from_requirement(binding).unwrap();
    let work = UiMountedPresentationWorkView::Initial(&initial);
    let selected = mounted_semantic_text(work);
    let (command, mechanic) = selected.mechanics[0];
    let UiNativeTextPresentationPreparation::Prepared(complete) =
        prepare_mounted_semantic_text(work, dpi, |_| Some(layout.as_ref())).unwrap()
    else {
        panic!("qualified initial text prepares");
    };
    let demand = &complete.demand_batches()[0];
    let pins = demand
        .records()
        .iter()
        .map(|record| {
            UiGlyphRasterPinRequest::from_text_mechanics(demand.layout_identity(), record.key())
        })
        .collect::<Vec<_>>();
    assert!(pins.len() > 1, "the fixture pins more than one glyph");
    let basis = UiMountedTextForegroundPresentationBasis::from_work(
        work,
        binding,
        dpi.dpi_milli(),
        None,
        presentation_damage_digest(work),
    );
    let receipt = |set: &Arc<[UiGlyphRasterPinRequest]>| {
        UiMountedTextForegroundReuseReceipt::from_prepared(command, mechanic, demand, set, basis)
    };
    let committed: Arc<[UiGlyphRasterPinRequest]> = Arc::from(pins.as_slice());
    let reordered: Vec<_> = pins.iter().rev().copied().collect();
    let other: Arc<[UiGlyphRasterPinRequest]> = Arc::from(&pins[1..]);
    let shared = [receipt(&committed), receipt(&committed)];
    let continuous = |receipts: &[&UiMountedTextForegroundReuseReceipt],
                      candidate: &[UiGlyphRasterPinRequest]| {
        UiMountedTextForegroundReuseReceipt::pins_are_continuous(
            receipts.iter().copied(),
            candidate,
        )
    };

    assert!(continuous(&[&shared[0], &shared[1]], &reordered));
    assert!(!continuous(&[&shared[0], &shared[1]], &pins[1..]));
    let later = receipt(&other);
    assert!(
        !continuous(&[&shared[0], &shared[1], &later], &pins),
        "a set that differs after a shared, verified set still fails"
    );
    assert!(!continuous(&[&later, &shared[0]], &pins));
}
