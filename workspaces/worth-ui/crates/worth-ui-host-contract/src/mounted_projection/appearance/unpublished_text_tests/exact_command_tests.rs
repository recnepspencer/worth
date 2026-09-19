use super::super::super::{context, node_affinity, surface_mechanic};
use super::{candidate_with_slot, foreground};
use crate::*;

#[test]
fn one_adopted_span_preserves_distinct_candidate_commands() {
    let context = context();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let receipt = surface_mechanic(&context, instance).node_receipt();
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
    let work = exact_command_work(&context, receipt, span, [&value, &posture]);
    let result = UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: None,
            successor: Some(receipt),
        },
        work,
        [value.clone(), posture.clone()],
        context.requirement,
        node_affinity(&context, receipt),
    );

    assert_eq!(result.unwrap().text_candidates(), &[value, posture]);
}

#[test]
fn foreground_command_cannot_borrow_a_sibling_candidate_span() {
    let context = context();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let receipt = surface_mechanic(&context, instance).node_receipt();
    let span = UiMountedTextPaintSpanIdentity::from_runtime_mounting([82; 32]);
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
    let work = exact_command_work(&context, receipt, span, [&posture]);
    assert_eq!(
        UiUnpublishedAppearanceFragment::from_runtime_mounting(
            UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                predecessor: None,
                successor: Some(receipt),
            },
            work,
            [value],
            context.requirement,
            node_affinity(&context, receipt),
        ),
        Err(UiUnpublishedAppearanceFrameProjectionDenial::TextCandidateSpanMismatch)
    );
}

fn exact_command_work<'a>(
    context: &super::super::super::Context,
    receipt: UiMountedNodeReceiptIdentity,
    span: UiMountedTextPaintSpanIdentity,
    candidates: impl IntoIterator<Item = &'a UiMountedSemanticTextMechanic>,
) -> UiMountedAppearanceWork {
    let mechanics = candidates
        .into_iter()
        .map(|candidate| {
            UiMountedAppearanceMechanic::TextForeground(foreground(
                context,
                receipt,
                UiMountedPaintCommandIdentity::semantic_text(candidate),
                span,
            ))
        })
        .collect::<Vec<_>>();
    let manifest = UiMountedAppearancePredecessorManifest::from_runtime_mounting(
        mechanics.iter().map(UiMountedAppearanceMechanic::identity),
        [],
    )
    .unwrap();
    let frame = UiMountedAppearanceFrame::from_runtime_mounting(
        context.frame,
        context.surface,
        mechanics,
        super::super::super::order(context),
    )
    .unwrap();
    UiMountedAppearanceWork::from_runtime_mounting(
        UiMountedAppearanceWorkPosture::Unchanged,
        Some(context.predecessor),
        Some(manifest),
        frame,
        [],
        [],
        false,
    )
    .unwrap()
}
