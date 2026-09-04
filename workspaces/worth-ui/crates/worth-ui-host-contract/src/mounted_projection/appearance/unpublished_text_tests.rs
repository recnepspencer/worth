use std::sync::Arc;

use super::super::{
    context, node_affinity, surface_mechanic, surface_work, text_row, text_work, Context,
};
use super::support::mixed_text_row;
use crate::*;

fn candidate_with_slot(
    context: &Context,
    instance: UiMountedInstanceIdentity,
    receipt: UiMountedNodeReceiptIdentity,
    span: UiMountedTextPaintSpanIdentity,
    slot: UiSemanticTextSlot,
    collection_row: Option<UiMountedCollectionRowCorrelation>,
    capability_generation: u64,
    capability_profile_digest: u64,
) -> UiMountedSemanticTextMechanic {
    let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 32.0,
        y: 32.0,
        width: 160.0,
        height: 96.0,
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .unwrap();
    UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
        UiMountedSemanticTextCompletionInput {
            content_generation: context.content,
            frame: context.frame,
            surface: context.surface,
            binding: context.binding,
            mounted_instance: instance,
            node_receipt: receipt,
            allocation_basis: UiMountedAllocationBasis::new(
                1,
                2,
                3,
                UiMountedTransformProjection::Identity,
            ),
            bounds,
            clip_bounds: bounds,
            origin_x: 32.0,
            origin_y: 40.0,
            text: Arc::from("ONLINE"),
            layout: super::super::inert_layout(),
            slot,
            collection_row,
            foregrounds: Arc::from([UiMountedTextForegroundSpan::from_runtime_mounting(
                UiTextOriginalRange::new(0, 6).unwrap(),
                UiMountedRgba8::new(255, 255, 255, 255),
                span,
            )]),
            profile: UiSemanticTextProfile::BodyDefault,
            layer_semantic_order: 7,
            capability_generation: WorthUiHostCapabilityObservationGeneration::new(
                capability_generation,
            ),
            capability_profile_digest,
        },
    )
    .unwrap()
}

fn surface_text_fragment(
    context: &Context,
    mechanic: UiMountedSurfaceAppearanceMechanic,
    candidates: impl IntoIterator<Item = UiMountedSemanticTextMechanic>,
) -> UiUnpublishedAppearanceFragment {
    surface_text_result(context, mechanic, candidates).unwrap()
}

fn surface_text_result(
    context: &Context,
    mechanic: UiMountedSurfaceAppearanceMechanic,
    candidates: impl IntoIterator<Item = UiMountedSemanticTextMechanic>,
) -> Result<UiUnpublishedAppearanceFragment, UiUnpublishedAppearanceFrameProjectionDenial> {
    let receipt = mechanic.node_receipt();
    let (_, work) = surface_work(context, mechanic);
    UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: None,
            successor: Some(receipt),
        },
        work,
        candidates,
        context.requirement,
        node_affinity(context, receipt),
    )
}

fn foreground(
    context: &Context,
    receipt: UiMountedNodeReceiptIdentity,
    span: UiMountedTextPaintSpanIdentity,
) -> UiMountedTextForegroundAppearanceMechanic {
    let issuer = UiMountedNodeReceiptIssuer::mint_for(context.frame).unwrap();
    UiMountedTextForegroundAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedTextForegroundAppearanceCompletionInput {
            issuer,
            node_receipt: receipt,
            paint_span: span,
            foreground: UiMountedAppearanceColor::from_straight_srgba([9, 8, 7, 255]),
            opacity: UiMountedAppearanceOpacity::ONE,
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap()
}

#[test]
fn mixed_candidate_retains_nonadopted_spans_and_joins_adopted_span() {
    let context = context();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mechanic = surface_mechanic(&context, instance);
    let receipt = mechanic.node_receipt();
    let adopted = UiMountedTextPaintSpanIdentity::from_runtime_mounting([51; 32]);
    let retained = UiMountedTextPaintSpanIdentity::from_runtime_mounting([52; 32]);
    let mechanic = foreground(&context, receipt, adopted);
    let (_, work) = text_work(&context, mechanic, receipt);
    let fragment = UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: None,
            successor: Some(receipt),
        },
        work,
        [mixed_text_row(
            &context, instance, receipt, adopted, retained,
        )],
        context.requirement,
        node_affinity(&context, receipt),
    )
    .unwrap();

    assert_eq!(fragment.text_candidates()[0].foregrounds().len(), 2);
}

#[test]
fn candidate_capability_generation_and_profile_digest_must_join_binding() {
    let context = context();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mechanic = surface_mechanic(&context, instance);
    let receipt = mechanic.node_receipt();
    let span = UiMountedTextPaintSpanIdentity::from_runtime_mounting([61; 32]);
    let wrong_generation = candidate_with_slot(
        &context,
        instance,
        receipt,
        span,
        UiSemanticTextSlot::Value,
        None,
        8,
        11,
    );
    let result = surface_text_result(&context, mechanic.clone(), [wrong_generation]);
    assert_eq!(
        result,
        Err(UiUnpublishedAppearanceFrameProjectionDenial::CandidateCapabilityGenerationMismatch)
    );

    let wrong_digest = candidate_with_slot(
        &context,
        instance,
        receipt,
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([62; 32]),
        UiSemanticTextSlot::Value,
        None,
        7,
        12,
    );
    let result = surface_text_result(&context, mechanic, [wrong_digest]);
    assert_eq!(
        result,
        Err(UiUnpublishedAppearanceFrameProjectionDenial::CandidateCapabilityProfileDigestMismatch)
    );
}

#[test]
fn same_receipt_candidates_preserve_value_collection_and_posture_rows() {
    let context = context();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mechanic = surface_mechanic(&context, instance);
    let receipt = mechanic.node_receipt();
    let value = candidate_with_slot(
        &context,
        instance,
        receipt,
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([71; 32]),
        UiSemanticTextSlot::Value,
        None,
        7,
        11,
    );
    let collection = candidate_with_slot(
        &context,
        instance,
        receipt,
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([72; 32]),
        UiSemanticTextSlot::CollectionValue {
            selected_field_ordinal: 2,
        },
        Some(UiMountedCollectionRowCorrelation::from_runtime_mounting(
            [73; 32],
        )),
        7,
        11,
    );
    let posture = candidate_with_slot(
        &context,
        instance,
        receipt,
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([74; 32]),
        UiSemanticTextSlot::Posture,
        None,
        7,
        11,
    );
    let fragment = surface_text_fragment(&context, mechanic, [value, collection, posture]);

    assert_eq!(fragment.text_candidates().len(), 3);
    assert_eq!(
        fragment.text_candidates()[1].slot(),
        UiSemanticTextSlot::CollectionValue {
            selected_field_ordinal: 2
        }
    );
}

#[test]
fn duplicate_receipt_and_paint_span_is_denied_even_for_distinct_slots() {
    let context = context();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mechanic = surface_mechanic(&context, instance);
    let receipt = mechanic.node_receipt();
    let span = UiMountedTextPaintSpanIdentity::from_runtime_mounting([81; 32]);
    let value = candidate_with_slot(
        &context,
        instance,
        receipt,
        span,
        UiSemanticTextSlot::Value,
        None,
        7,
        11,
    );
    let posture = candidate_with_slot(
        &context,
        instance,
        receipt,
        span,
        UiSemanticTextSlot::Posture,
        None,
        7,
        11,
    );
    let (_, work) = surface_work(&context, mechanic);
    let result = UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: None,
            successor: Some(receipt),
        },
        work,
        [value, posture],
        context.requirement,
        node_affinity(&context, receipt),
    );

    assert_eq!(
        result,
        Err(
            UiUnpublishedAppearanceFrameProjectionDenial::ConflictingTextCandidate {
                receipt,
                span,
            }
        )
    );
}

#[test]
fn text_candidate_admission_stops_at_capacity_plus_one() {
    struct BoundedCandidates {
        yielded: usize,
        candidate: UiMountedSemanticTextMechanic,
    }

    impl Iterator for BoundedCandidates {
        type Item = UiMountedSemanticTextMechanic;

        fn next(&mut self) -> Option<Self::Item> {
            if self.yielded == UiMountedSemanticTextTable::MAX_ROWS + 1 {
                panic!("text candidate admission drained beyond capacity plus one");
            }
            self.yielded += 1;
            Some(self.candidate.clone())
        }
    }

    let context = context();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(context.frame).unwrap();
    let receipt = issuer.receipt_for(instance);
    let candidate = text_row(
        &context,
        instance,
        receipt,
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([91; 32]),
    );
    let mechanic = surface_mechanic(&context, instance);
    let (_, work) = surface_work(&context, mechanic);
    assert_eq!(
        UiUnpublishedAppearanceFragment::from_runtime_mounting(
            UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                predecessor: None,
                successor: Some(receipt),
            },
            work,
            BoundedCandidates {
                yielded: 0,
                candidate,
            },
            context.requirement,
            node_affinity(&context, receipt),
        ),
        Err(UiUnpublishedAppearanceFrameProjectionDenial::TextCandidateCapacityExceeded)
    );
}
