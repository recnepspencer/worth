use super::*;
use worth_ui_host_contract::UiSemanticTextSlot;

use crate::mounting::projection::frame_storage::mechanic_source::UiMountedMechanicMutation;

pub(super) fn append_paragraphs(
    source: &mut UiMountedMechanicSource,
    fonts: &Arc<worth_ui_text::UiGlobalFontCollection>,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    instances: &mut Vec<UiMountedInstanceIdentity>,
    target_size: usize,
) {
    let start = instances.len();
    let added = (start..target_size)
        .map(|index| {
            let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
            instances.push(instance);
            locality_node(
                crate::graph::UiGraphNodeIdentity::new(50_000 + index as u64),
                instance,
                surface,
                "CURRENT",
                160.0,
            )
        })
        .collect::<Vec<_>>();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let receipts = receipt_basis_for(frame, &instances[start..]);
    let projection = UiMountedSemanticProjection::initial(
        added,
        vec![UiMountedProjectionSurface {
            surface,
            binding,
            audience: UiMountedProjectionAudience::full(),
        }],
    );
    let mutation = source
        .apply(completion(
            frame,
            UiMountedContentGeneration::mint_unbound().unwrap(),
            &receipts,
            &projection,
            fonts,
            &instances[start..],
            10 + target_size as u64,
        ))
        .unwrap();
    assert_eq!(mutation.semantic_text, target_size - start);
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_one(
    source: &mut UiMountedMechanicSource,
    fonts: &Arc<worth_ui_text::UiGlobalFontCollection>,
    instance: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    text: &'static str,
    width: f32,
    changed: &[UiMountedInstanceIdentity],
) -> UiMountedMechanicMutation {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let receipts = receipt_basis_for(frame, &[instance]);
    let projection = UiMountedSemanticProjection::initial(
        vec![locality_node(
            crate::graph::UiGraphNodeIdentity::new(90_000),
            instance,
            surface,
            text,
            width,
        )],
        vec![UiMountedProjectionSurface {
            surface,
            binding,
            audience: UiMountedProjectionAudience::full(),
        }],
    );
    source
        .apply(completion(
            frame,
            UiMountedContentGeneration::mint_unbound().unwrap(),
            &receipts,
            &projection,
            fonts,
            changed,
            20 + u64::from(width.to_bits()),
        ))
        .unwrap()
}

pub(super) fn replace_one_paragraph(
    source: &mut UiMountedMechanicSource,
    fonts: &Arc<worth_ui_text::UiGlobalFontCollection>,
    instance: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    text: &'static str,
    width: f32,
) -> UiMountedMechanicMutation {
    apply_one(
        source,
        fonts,
        instance,
        surface,
        binding,
        text,
        width,
        &[instance],
    )
}

fn locality_node(
    graph_node: crate::graph::UiGraphNodeIdentity,
    instance: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    text: &'static str,
    width: f32,
) -> UiMountedProjectionNodeRecord {
    let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 0.0,
        y: 0.0,
        width,
        height: 96.0,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap();
    UiMountedProjectionNodeRecord {
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
        plan_index: None,
        occurrence_allocation: UiMountedAllocationProjection::Known {
            bounds,
            basis: UiMountedAllocationBasis::new(1, 2, 3, UiMountedTransformProjection::Identity),
        },
        appearance_geometry:
            crate::mounting::projection::frame_storage::UiMountedAppearanceGeometry::from_occurrence(
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
        semantic_text: Some(UiMountedSemanticTextSeed::posture_only_for_test(text)),
        hit_test: None,
        focus_support: crate::capability::ComponentFocusSupport::not_focusable(),
        focus_scope: None,
        focus_container_owner: None,
        component_id: None,
        portal_child_owner: None,
    }
}

fn receipt_basis_for(
    frame: UiMountedFrameIdentity,
    instances: &[UiMountedInstanceIdentity],
) -> crate::mounting::UiMountedNodeReceiptBasis {
    let mut presented = crate::runtime::persistent_index::UiPersistentOrdSet::default();
    for instance in instances {
        presented.insert(*instance);
    }
    crate::mounting::UiMountedNodeReceiptBasis::mint(frame, presented).unwrap()
}

pub(super) fn retained_layouts(
    source: &UiMountedMechanicSource,
    instances: &[UiMountedInstanceIdentity],
) -> Vec<Arc<worth_ui_text::UiQualifiedTextLayout>> {
    instances
        .iter()
        .map(|instance| Arc::clone(layout(source, *instance)))
        .collect()
}

pub(super) fn layout(
    source: &UiMountedMechanicSource,
    instance: UiMountedInstanceIdentity,
) -> &Arc<worth_ui_text::UiQualifiedTextLayout> {
    source
        .qualified_layout_for(instance, UiSemanticTextSlot::Posture)
        .unwrap()
}

pub(super) fn assert_only_target_changed(
    before: &[Arc<worth_ui_text::UiQualifiedTextLayout>],
    source: &UiMountedMechanicSource,
    instances: &[UiMountedInstanceIdentity],
    target: UiMountedInstanceIdentity,
) {
    for (instance, predecessor) in instances.iter().zip(before) {
        assert_eq!(
            Arc::ptr_eq(predecessor, layout(source, *instance)),
            *instance != target,
            "only the named paragraph may receive a new layout owner"
        );
    }
}

pub(super) fn assert_constant_cost(
    expected: &mut Option<worth_ui_host_contract::UiQualifiedTextCostRecord>,
    observed: worth_ui_host_contract::UiQualifiedTextCostRecord,
) {
    if let Some(expected) = expected {
        assert_eq!(*expected, observed);
    } else {
        *expected = Some(observed);
    }
    assert_eq!(observed.analyzed_bytes(), 7);
    assert_eq!(observed.bidi_contexts(), 1);
    assert!(observed.fallback_clusters() > 0);
    assert!(observed.probed_glyphs() > 0);
    assert!(observed.shaped_runs() > 0);
    assert!(observed.emitted_glyphs() > 0);
    assert!(observed.emitted_lines() > 0);
}
