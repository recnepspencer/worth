use std::sync::Arc;

use worth_ui_host_contract::{
    UiMountIncarnation, UiMountedAllocationBasis, UiMountedAllocationProjection,
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedContentGeneration,
    UiMountedCoordinateSpace, UiMountedFrameIdentity, UiMountedInstanceIdentity,
    UiMountedMechanicalRole, UiMountedPaintCommand, UiMountedParticipation,
    UiMountedParticipationFact, UiMountedParticipationInput, UiMountedParticipationStatus,
    UiMountedProjectionAudience, UiMountedRgba8, UiMountedTransformProjection,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
    WorthUiHostCapabilityObservationGeneration,
};

use super::mechanic_source::{UiMountedMechanicCompletion, UiMountedMechanicSource};
use super::{
    UiMountedProjectionNodeRecord, UiMountedProjectionSurface, UiMountedSemanticProjection,
};
use crate::mounting::projection::node_receipt::{UiMountedNodeReceipt, UiMountedNodeReceiptInput};
use crate::mounting::projection::semantic_text::{
    lower_semantic_text_seed, UiMountedSemanticTextFormattingSeed, UiMountedSemanticTextSeed,
};
use crate::mounting::projection::{
    hit_test::UiMountedHitTestSeed, static_paint::UiMountedStaticPaintSeed,
};

mod frame_affinity;
mod hit_locality;
mod phase4_locality;
mod phase4_portal_children;
mod semantic_fixture;
use semantic_fixture::{semantic_projection, semantic_projection_with_static_color};

#[test]
pub(crate) fn mechanic_source_routes_paint_only_work_through_current_mounted_authority() {
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
    let (foreign_fonts, _) =
        worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let foreign_fonts = Arc::new(foreign_fonts);
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let graph_node = crate::graph::UiGraphNodeIdentity::new(4_041);
    let initial_seed = UiMountedSemanticTextSeed::scalar_for_test();
    let initial_semantic =
        semantic_projection(graph_node, instance, surface, binding, initial_seed.clone());
    let initial_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let initial_content = UiMountedContentGeneration::mint_unbound().unwrap();
    let initial_receipts = receipt_basis(initial_frame, instance);
    let changed = [instance];
    let mut source = UiMountedMechanicSource::default();

    source
        .apply(completion(
            initial_frame,
            initial_content,
            &initial_receipts,
            &initial_semantic,
            &fonts,
            &changed,
            1,
        ))
        .unwrap();
    let predecessor_layouts = semantic_rows(&source, instance, surface, binding)
        .iter()
        .map(|row| {
            (
                row.slot(),
                Arc::clone(source.qualified_layout_for(instance, row.slot()).unwrap()),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(predecessor_layouts.len(), 2);

    let successor_seed = lower_semantic_text_seed(
        None,
        Some(&initial_seed),
        Some(
            UiMountedSemanticTextFormattingSeed::body_default_with_color_for_test(
                UiMountedRgba8::new(247, 129, 47, 255),
            ),
        ),
    )
    .unwrap()
    .unwrap();
    let successor_semantic = semantic_projection(
        graph_node,
        instance,
        surface,
        binding,
        successor_seed.clone(),
    );
    let successor_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let successor_content = UiMountedContentGeneration::mint_unbound().unwrap();
    let successor_receipts = receipt_basis(successor_frame, instance);
    let mutation = source
        .apply(completion(
            successor_frame,
            successor_content,
            &successor_receipts,
            &successor_semantic,
            &fonts,
            &changed,
            2,
        ))
        .unwrap();

    assert_eq!(mutation.precise_instances, [instance]);
    assert_eq!(mutation.command_changes.len(), 2);
    let successor_rows = semantic_rows(&source, instance, surface, binding);
    assert_eq!(successor_rows.len(), predecessor_layouts.len());
    for row in &successor_rows {
        let predecessor_layout = predecessor_layouts
            .iter()
            .find_map(|(slot, layout)| (*slot == row.slot()).then_some(layout))
            .unwrap();
        assert!(Arc::ptr_eq(
            predecessor_layout,
            source.qualified_layout_for(instance, row.slot()).unwrap()
        ));
        assert_eq!(row.frame(), successor_frame);
        assert_eq!(row.content_generation(), successor_content);
        assert_eq!(
            row.node_receipt(),
            successor_receipts.receipt_for(instance).unwrap()
        );
        assert_eq!(
            row.foregrounds()[0].color(),
            UiMountedRgba8::new(247, 129, 47, 255)
        );
    }

    let retained_successor_layouts = successor_rows
        .iter()
        .map(|row| {
            (
                row.slot(),
                Arc::clone(source.qualified_layout_for(instance, row.slot()).unwrap()),
            )
        })
        .collect::<Vec<_>>();
    let foreign_seed = lower_semantic_text_seed(
        None,
        Some(&successor_seed),
        Some(
            UiMountedSemanticTextFormattingSeed::body_default_with_color_for_test(
                UiMountedRgba8::new(47, 129, 247, 255),
            ),
        ),
    )
    .unwrap()
    .unwrap();
    let foreign_semantic =
        semantic_projection(graph_node, instance, surface, binding, foreign_seed);
    let foreign_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let foreign_content = UiMountedContentGeneration::mint_unbound().unwrap();
    let foreign_receipts = receipt_basis(foreign_frame, instance);
    source
        .apply(completion(
            foreign_frame,
            foreign_content,
            &foreign_receipts,
            &foreign_semantic,
            &foreign_fonts,
            &changed,
            3,
        ))
        .unwrap();
    for row in semantic_rows(&source, instance, surface, binding) {
        let predecessor = retained_successor_layouts
            .iter()
            .find_map(|(slot, layout)| (*slot == row.slot()).then_some(layout))
            .unwrap();
        assert!(
            !Arc::ptr_eq(
                predecessor,
                source.qualified_layout_for(instance, row.slot()).unwrap()
            ),
            "same-numbered foreign collection must force exact requalification"
        );
    }
}

#[test]
fn sparse_text_does_not_hide_a_simultaneous_static_paint_change() {
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let graph_node = crate::graph::UiGraphNodeIdentity::new(4_042);
    let initial_seed = UiMountedSemanticTextSeed::scalar_for_test();
    let initial = semantic_projection(graph_node, instance, surface, binding, initial_seed.clone());
    let initial_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let initial_receipts = receipt_basis(initial_frame, instance);
    let changed = [instance];
    let mut source = UiMountedMechanicSource::default();
    source
        .apply(completion(
            initial_frame,
            UiMountedContentGeneration::mint_unbound().unwrap(),
            &initial_receipts,
            &initial,
            &fonts,
            &changed,
            1,
        ))
        .unwrap();

    let successor_seed = lower_semantic_text_seed(
        None,
        Some(&initial_seed),
        Some(
            UiMountedSemanticTextFormattingSeed::body_default_with_color_for_test(
                UiMountedRgba8::new(247, 129, 47, 255),
            ),
        ),
    )
    .unwrap()
    .unwrap();
    let successor = semantic_projection_with_static_color(
        graph_node,
        instance,
        surface,
        binding,
        successor_seed,
        UiMountedRgba8::new(48, 129, 247, 255),
    );
    let successor_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let successor_receipts = receipt_basis(successor_frame, instance);
    let mutation = source
        .apply(completion(
            successor_frame,
            UiMountedContentGeneration::mint_unbound().unwrap(),
            &successor_receipts,
            &successor,
            &fonts,
            &changed,
            2,
        ))
        .unwrap();

    assert!(mutation.precise_instances.is_empty());
    assert_eq!(mutation.command_changes.len(), 2);
}

fn semantic_rows(
    source: &UiMountedMechanicSource,
    instance: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
) -> Vec<worth_ui_host_contract::UiMountedSemanticTextMechanic> {
    source
        .commands_for_instance(instance, surface, binding)
        .iter()
        .filter_map(|command| match command {
            UiMountedPaintCommand::SemanticText { mechanic, .. } => Some(mechanic.clone()),
            UiMountedPaintCommand::FilledRect { .. }
            | UiMountedPaintCommand::PortalOverlay { .. } => None,
        })
        .collect()
}

fn completion<'a>(
    frame: UiMountedFrameIdentity,
    content: UiMountedContentGeneration,
    receipts: &'a crate::mounting::UiMountedNodeReceiptBasis,
    semantic: &'a UiMountedSemanticProjection,
    fonts: &'a Arc<worth_ui_text::UiGlobalFontCollection>,
    changed: &'a [UiMountedInstanceIdentity],
    capability_generation: u64,
) -> UiMountedMechanicCompletion<'a> {
    UiMountedMechanicCompletion {
        frame,
        content,
        receipts,
        semantic,
        changed,
        capability_generation: WorthUiHostCapabilityObservationGeneration::new(
            capability_generation,
        ),
        capability_profile_digest: capability_generation,
        font_collection: fonts,
    }
}

fn receipt_basis(
    frame: UiMountedFrameIdentity,
    instance: UiMountedInstanceIdentity,
) -> crate::mounting::UiMountedNodeReceiptBasis {
    let mut instances = crate::runtime::persistent_index::UiPersistentOrdSet::default();
    instances.insert(instance);
    crate::mounting::UiMountedNodeReceiptBasis::mint(frame, instances).unwrap()
}

fn admitted_participation() -> UiMountedParticipation {
    let admitted = UiMountedParticipationFact::new(UiMountedParticipationStatus::Admitted);
    let withheld = UiMountedParticipationFact::new(UiMountedParticipationStatus::Withheld);
    UiMountedParticipation::new(UiMountedParticipationInput {
        paint: admitted,
        clip: admitted,
        input: withheld,
        focus: withheld,
        hit_test: admitted,
        accessibility: withheld,
        motion: withheld,
        diagnostic: withheld,
    })
}

fn canonical_bounds() -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 0.0,
        y: 0.0,
        width: 160.0,
        height: 96.0,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap()
}
