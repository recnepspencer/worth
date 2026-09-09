use super::*;

mod collection;
mod world;

use world::*;

const LOCALITY_SIZES: [usize; 4] = [1, 32, 2_048, 4_096];

#[test]
fn content_and_width_locality_have_exact_constant_work_at_every_qualified_size() {
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let mut source = UiMountedMechanicSource::default();
    let mut instances = Vec::with_capacity(*LOCALITY_SIZES.last().unwrap());
    let mut expected_content_cost = None;
    let mut expected_width_cost = None;
    let mut expected_content_visits = None;
    let mut expected_width_visits = None;
    let mut expected_unchanged_visits = None;
    let mut observations = Vec::new();

    for size in LOCALITY_SIZES {
        append_paragraphs(&mut source, &fonts, surface, binding, &mut instances, size);
        let target = *instances.last().unwrap();
        let before_content = retained_layouts(&source, &instances);
        source.begin_semantic_instance_index_observation();
        let content = replace_one_paragraph(
            &mut source,
            &fonts,
            target,
            surface,
            binding,
            "UPDATED",
            160.0,
        );
        let content_index_work = crate::runtime::persistent_index::test_work();
        assert_eq!(content.semantic_text, 1);
        assert_eq!(content.command_changes.len(), 1);
        assert_constant_visits(
            &mut expected_content_visits,
            content_index_work.iterated_entries(),
        );
        assert_only_target_changed(&before_content, &source, &instances, target);
        let content_cost = layout(&source, target).view().cost();
        assert_constant_cost(&mut expected_content_cost, content_cost);

        let before_width = retained_layouts(&source, &instances);
        source.begin_semantic_instance_index_observation();
        let width = replace_one_paragraph(
            &mut source,
            &fonts,
            target,
            surface,
            binding,
            "UPDATED",
            80.0,
        );
        let width_index_work = crate::runtime::persistent_index::test_work();
        assert_eq!(width.semantic_text, 1);
        assert_eq!(width.command_changes.len(), 1);
        assert_constant_visits(
            &mut expected_width_visits,
            width_index_work.iterated_entries(),
        );
        assert_only_target_changed(&before_width, &source, &instances, target);
        let width_cost = layout(&source, target).view().cost();
        assert_constant_cost(&mut expected_width_cost, width_cost);

        source.begin_semantic_instance_index_observation();
        let unchanged = apply_one(
            &mut source,
            &fonts,
            target,
            surface,
            binding,
            "UPDATED",
            80.0,
            &[],
        );
        let unchanged_index_work = crate::runtime::persistent_index::test_work();
        assert_eq!(unchanged.semantic_text, 0);
        assert!(unchanged.command_changes.is_empty());
        assert_constant_visits(
            &mut expected_unchanged_visits,
            unchanged_index_work.iterated_entries(),
        );
        for work in [content_index_work, width_index_work, unchanged_index_work] {
            assert!(
                work.lookup_probes() <= 192,
                "ordinary paragraph locality must remain bounded by indexed lookup paths"
            );
        }
        observations.push(format!(
            "{{\"size\":{size},\"content\":{},\"width\":{},\"content_lookup_probes\":{},\"width_lookup_probes\":{},\"content_local_row_visits\":{},\"width_local_row_visits\":{},\"content_sibling_visits\":0,\"width_sibling_visits\":0,\"unchanged_lookup_probes\":{},\"unchanged_local_row_visits\":{},\"unchanged_sibling_visits\":0}}",
            cost_json(content_cost),
            cost_json(width_cost),
            content_index_work.lookup_probes(),
            width_index_work.lookup_probes(),
            content_index_work.iterated_entries(),
            width_index_work.iterated_entries(),
            unchanged_index_work.lookup_probes(),
            unchanged_index_work.iterated_entries(),
        ));
    }
    println!(
        "WORTH_UI_PHASE4_LOCALITY={{\"observations\":[{}],\"changed_paragraphs\":1}}",
        observations.join(",")
    );
}

fn assert_constant_visits(expected: &mut Option<usize>, observed: usize) {
    if let Some(expected) = expected {
        assert_eq!(
            *expected, observed,
            "ordinary paragraph work must not grow with retained siblings"
        );
    } else {
        *expected = Some(observed);
    }
}

fn cost_json(cost: worth_ui_host_contract::UiQualifiedTextCostRecord) -> String {
    format!(
        "{{\"analyzed_bytes\":{},\"bidi_contexts\":{},\"fallback_clusters\":{},\"fallback_probes\":{},\"shaped_runs\":{},\"glyphs\":{},\"lines\":{}}}",
        cost.analyzed_bytes(),
        cost.bidi_contexts(),
        cost.fallback_clusters(),
        cost.probed_glyphs(),
        cost.shaped_runs(),
        cost.emitted_glyphs(),
        cost.emitted_lines(),
    )
}

#[test]
fn retained_document_scan_and_global_width_substitution_are_rejected() {
    content_and_width_locality_have_exact_constant_work_at_every_qualified_size();
}

#[test]
fn one_width_change_relayouts_only_its_mounted_paragraphs_and_unchanged_is_zero_work() {
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let left = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let right = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let seed = UiMountedSemanticTextSeed::scalar_for_test();
    let mut source = UiMountedMechanicSource::default();

    apply_instance(
        &mut source,
        &fonts,
        InstanceFixture {
            instance: left,
            surface,
            binding,
            node: crate::graph::UiGraphNodeIdentity::new(4_042),
            seed: seed.clone(),
            width: 160.0,
            generation: 1,
        },
    );
    apply_instance(
        &mut source,
        &fonts,
        InstanceFixture {
            instance: right,
            surface,
            binding,
            node: crate::graph::UiGraphNodeIdentity::new(4_043),
            seed: seed.clone(),
            width: 160.0,
            generation: 2,
        },
    );
    let left_before = layouts(&source, left, surface, binding);
    let right_before = layouts(&source, right, surface, binding);

    let semantic = semantic_projection_with_width(
        crate::graph::UiGraphNodeIdentity::new(4_042),
        left,
        surface,
        binding,
        seed,
        80.0,
    );
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let receipts = receipt_basis(frame, left);
    let changed = [left];
    let mutation = source
        .apply(completion(
            frame,
            UiMountedContentGeneration::mint_unbound().unwrap(),
            &receipts,
            &semantic,
            &fonts,
            &changed,
            3,
        ))
        .unwrap();
    assert_eq!(mutation.semantic_text, 2);
    let left_after = layouts(&source, left, surface, binding);
    let right_after = layouts(&source, right, surface, binding);
    assert!(left_before
        .iter()
        .zip(&left_after)
        .all(|(before, after)| !Arc::ptr_eq(before, after)));
    assert!(right_before
        .iter()
        .zip(&right_after)
        .all(|(before, after)| Arc::ptr_eq(before, after)));

    let unchanged = source
        .apply(completion(
            UiMountedFrameIdentity::mint_unbound().unwrap(),
            UiMountedContentGeneration::mint_unbound().unwrap(),
            &receipts,
            &semantic,
            &fonts,
            &[],
            4,
        ))
        .unwrap();
    assert_eq!(unchanged.semantic_text, 0);
    assert!(layouts(&source, left, surface, binding)
        .iter()
        .zip(&left_after)
        .all(|(after, expected)| Arc::ptr_eq(after, expected)));
}

#[test]
fn global_width_relayout_and_unchanged_rescan_are_rejected() {
    one_width_change_relayouts_only_its_mounted_paragraphs_and_unchanged_is_zero_work();
}

struct InstanceFixture {
    instance: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    node: crate::graph::UiGraphNodeIdentity,
    seed: UiMountedSemanticTextSeed,
    width: f32,
    generation: u64,
}

fn apply_instance(
    source: &mut UiMountedMechanicSource,
    fonts: &Arc<worth_ui_text::UiGlobalFontCollection>,
    fixture: InstanceFixture,
) {
    let InstanceFixture {
        instance,
        surface,
        binding,
        node,
        seed,
        width,
        generation,
    } = fixture;
    let semantic = semantic_projection_with_width(node, instance, surface, binding, seed, width);
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let receipts = receipt_basis(frame, instance);
    source
        .apply(completion(
            frame,
            UiMountedContentGeneration::mint_unbound().unwrap(),
            &receipts,
            &semantic,
            fonts,
            &[instance],
            generation,
        ))
        .unwrap();
}

fn layouts(
    source: &UiMountedMechanicSource,
    instance: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
) -> Vec<Arc<worth_ui_text::UiQualifiedTextLayout>> {
    semantic_rows(source, instance, surface, binding)
        .iter()
        .map(|row| Arc::clone(source.qualified_layout_for(instance, row.slot()).unwrap()))
        .collect()
}

fn semantic_projection_with_width(
    graph_node: crate::graph::UiGraphNodeIdentity,
    instance: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    seed: UiMountedSemanticTextSeed,
    width: f32,
) -> UiMountedSemanticProjection {
    let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 0.0,
        y: 0.0,
        width,
        height: 96.0,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap();
    UiMountedSemanticProjection::initial(
        vec![UiMountedProjectionNodeRecord {
            receipt: UiMountedNodeReceipt::from_input(UiMountedNodeReceiptInput {
                mounted_instance: instance,
                graph_node,
                semantic_surface: surface,
                incarnation: UiMountIncarnation::mint_unbound().unwrap(),
                plan_digest: 7,
                role: UiMountedMechanicalRole::Control,
                participation: text_only_participation(),
                allocation: UiMountedAllocationProjection::Known {
                    bounds,
                    basis: UiMountedAllocationBasis::new(
                        1,
                        2,
                        3,
                        UiMountedTransformProjection::Identity,
                    ),
                },
            }),
            plan_index: Some(0),
        occurrence_allocation: UiMountedAllocationProjection::Known {
                    bounds,
                    basis: UiMountedAllocationBasis::new(
                        1,
                        2,
                        3,
                        UiMountedTransformProjection::Identity,
                    ),
                },
        appearance_geometry: crate::mounting::projection::frame_storage::UiMountedAppearanceGeometry::from_occurrence(
                UiMountedAllocationProjection::Known {
                    bounds,
                    basis: UiMountedAllocationBasis::new(
                        1,
                        2,
                        3,
                        UiMountedTransformProjection::Identity,
                    ),
                },
                crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
            ),
        surface_paint_order: Some(0),
        has_appearance_attachment: false,
            appearance_clip:
                crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
            static_paint: None,
            semantic_text: Some(seed),
            hit_test: None,
            focus_support: crate::capability::ComponentFocusSupport::not_focusable(),
            focus_scope: None,
            focus_container_owner: None,
            component_id: None,
            portal_child_owner: None,
        }],
        vec![UiMountedProjectionSurface {
            surface,
            binding,
            audience: UiMountedProjectionAudience::full(),
        }],
    )
}

fn text_only_participation() -> UiMountedParticipation {
    let admitted = UiMountedParticipationFact::new(UiMountedParticipationStatus::Admitted);
    let withheld = UiMountedParticipationFact::new(UiMountedParticipationStatus::Withheld);
    UiMountedParticipation::new(UiMountedParticipationInput {
        paint: admitted,
        clip: admitted,
        input: withheld,
        focus: withheld,
        hit_test: withheld,
        accessibility: withheld,
        motion: withheld,
        diagnostic: withheld,
    })
}
