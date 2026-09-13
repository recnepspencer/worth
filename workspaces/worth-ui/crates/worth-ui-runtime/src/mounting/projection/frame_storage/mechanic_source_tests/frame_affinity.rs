use super::*;

#[test]
fn unchanged_successor_views_refresh_affinity_without_mutating_retained_mechanics() {
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let semantic = semantic_projection(
        crate::graph::UiGraphNodeIdentity::new(4_042),
        instance,
        surface,
        binding,
        UiMountedSemanticTextSeed::scalar_for_test(),
    );
    let predecessor_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let predecessor_content = UiMountedContentGeneration::mint_unbound().unwrap();
    let predecessor_receipts = receipt_basis(predecessor_frame, instance);
    let changed = [instance];
    let mut source = UiMountedMechanicSource::default();
    source
        .apply(completion(
            predecessor_frame,
            predecessor_content,
            &predecessor_receipts,
            &semantic,
            &fonts,
            &changed,
            1,
        ))
        .unwrap();
    let predecessor_hit = source
        .hit_tests_for(
            &semantic,
            surface,
            binding,
            predecessor_frame,
            &predecessor_receipts,
        )
        .unwrap()[0];
    let predecessor_text = source
        .semantic_text_for(
            &semantic,
            surface,
            binding,
            predecessor_content,
            predecessor_frame,
            &predecessor_receipts,
        )
        .unwrap();

    let successor_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let successor_content = UiMountedContentGeneration::mint_unbound().unwrap();
    let successor_receipts = receipt_basis(successor_frame, instance);
    let mutation = source
        .apply(completion(
            successor_frame,
            successor_content,
            &successor_receipts,
            &semantic,
            &fonts,
            &[],
            2,
        ))
        .unwrap();
    assert_eq!((mutation.semantic_text, mutation.hit_tests), (0, 0));
    assert!(mutation.command_changes.is_empty());

    let successor_receipt = successor_receipts.receipt_for(instance).unwrap();
    let successor_hits = source
        .hit_tests_for(
            &semantic,
            surface,
            binding,
            successor_frame,
            &successor_receipts,
        )
        .unwrap();
    let successor_text = source
        .semantic_text_for(
            &semantic,
            surface,
            binding,
            successor_content,
            successor_frame,
            &successor_receipts,
        )
        .unwrap();
    assert_eq!(successor_hits[0].node_receipt(), successor_receipt);
    assert!(successor_text.iter().all(|row| {
        row.frame() == successor_frame
            && row.content_generation() == successor_content
            && row.node_receipt() == successor_receipt
    }));
    assert_eq!(predecessor_hit.frame(), predecessor_frame);
    assert!(predecessor_text
        .iter()
        .all(|row| row.frame() == predecessor_frame));
    assert!(source
        .commands_for_instance(instance, surface, binding)
        .iter()
        .all(|command| match command {
            UiMountedPaintCommand::PortalOverlay { mechanic, .. } => {
                mechanic.frame() == predecessor_frame
            }
            UiMountedPaintCommand::SemanticText { mechanic, .. } => {
                mechanic.frame() == predecessor_frame
            }
        }));
}
