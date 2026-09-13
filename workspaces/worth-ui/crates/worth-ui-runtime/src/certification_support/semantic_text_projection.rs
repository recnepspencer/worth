//! SUPPORT AUTHORITY for host-adapter semantic-text translation tests.

use std::sync::Arc;

use worth_ui_host_contract::{
    UiMountedAccessibilityProjection, UiMountedAllocationBasis, UiMountedAllocationProjection,
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedContentGeneration,
    UiMountedCoordinateSpace, UiMountedDiagnosticProjection, UiMountedFrameIdentity,
    UiMountedInstanceIdentity, UiMountedMechanicalRole, UiMountedMotionProjection,
    UiMountedNodeProjectionView, UiMountedNodeProjectionViewInput, UiMountedNodeReceiptIdentity,
    UiMountedNodeReceiptIssuer, UiMountedOmissionReason, UiMountedPaintBatchTable,
    UiMountedPaintProjection, UiMountedParticipation, UiMountedParticipationFact,
    UiMountedParticipationInput, UiMountedParticipationStatus, UiMountedPreviewProjection,
    UiMountedProjectionView, UiMountedProjectionViewInput, UiMountedRealtimeBatchTable,
    UiMountedResourceTable, UiMountedRgba8, UiMountedSemanticTextCompletionInput,
    UiMountedSemanticTextMechanic, UiMountedSemanticTextReference, UiMountedSemanticTextTable,
    UiMountedSpatialBatchTable, UiMountedTextForegroundSpan, UiMountedTextPaintSpanIdentity,
    UiMountedTransformProjection, UiSemanticSurfaceIdentity, UiSemanticTextProfile,
    UiSemanticTextSlot, UiSurfaceBindingGeneration, WorthUiHostCapabilityObservationGeneration,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiSemanticTextProjectionCertificationMutation {
    Exact,
    ForeignFrame,
    ForeignSurface,
    ForeignBinding,
    ForeignContentGeneration,
    ForeignInstance,
    ForeignNodeReceipt,
    ForeignAllocation,
    ForeignCapabilityGeneration,
    ForeignCapabilityProfile,
    WithheldPaint,
    MissingReference,
    DuplicateReference,
    UnreferencedRow,
}

struct SemanticTextRowBasis {
    frame: UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    content_generation: UiMountedContentGeneration,
    instance: UiMountedInstanceIdentity,
    receipt: UiMountedNodeReceiptIdentity,
    allocation: UiMountedAllocationBasis,
    bounds: UiMountedCanonicalBox,
    capability_generation: WorthUiHostCapabilityObservationGeneration,
    capability_profile_digest: u64,
    mutation: UiSemanticTextProjectionCertificationMutation,
    text: Arc<str>,
}

pub fn semantic_text_projection_for_certification(
    mutation: UiSemanticTextProjectionCertificationMutation,
) -> UiMountedProjectionView {
    semantic_text_projection(
        mutation,
        WorthUiHostCapabilityObservationGeneration::new(7),
        11,
        Arc::from("ONLINE"),
    )
}

pub fn semantic_text_projection_for_certification_with_text(text: &str) -> UiMountedProjectionView {
    semantic_text_projection(
        UiSemanticTextProjectionCertificationMutation::Exact,
        WorthUiHostCapabilityObservationGeneration::new(7),
        11,
        Arc::from(text),
    )
}

pub fn semantic_text_projection_for_certification_with_capability(
    capability_generation: WorthUiHostCapabilityObservationGeneration,
    capability_profile_digest: u64,
) -> UiMountedProjectionView {
    semantic_text_projection(
        UiSemanticTextProjectionCertificationMutation::Exact,
        capability_generation,
        capability_profile_digest,
        Arc::from("ONLINE"),
    )
}

pub fn empty_projection_for_certification() -> UiMountedProjectionView {
    UiMountedProjectionView::new(UiMountedProjectionViewInput {
        frame: UiMountedFrameIdentity::mint_unbound().expect("frame identity"),
        surface: UiSemanticSurfaceIdentity::mint_unbound().expect("surface identity"),
        binding: UiSurfaceBindingGeneration::mint_unbound().expect("binding generation"),
        content_generation: UiMountedContentGeneration::mint_unbound().expect("content generation"),
        nodes: Vec::new(),
        clips: worth_ui_host_contract::UiMountedClipTable::produced(Vec::new()),
        layers: worth_ui_host_contract::UiMountedLayerTable::produced(Vec::new()),
        portal_overlays: worth_ui_host_contract::UiMountedPortalOverlayTable::empty(),
        semantic_text: UiMountedSemanticTextTable::empty(),
        hit_tests: worth_ui_host_contract::UiMountedHitTestTable::empty(),
        paint_batches: UiMountedPaintBatchTable::new(Vec::new()),
        spatial_batches: UiMountedSpatialBatchTable::new(Vec::new()),
        realtime_batches: UiMountedRealtimeBatchTable::new(Vec::new()),
        resources: UiMountedResourceTable::new(Vec::new()),
        authored_paint_commands: Vec::new(),
        authored_paint_order: Vec::new(),
    })
}

fn semantic_text_projection(
    mutation: UiSemanticTextProjectionCertificationMutation,
    capability_generation: WorthUiHostCapabilityObservationGeneration,
    capability_profile_digest: u64,
    text: Arc<str>,
) -> UiMountedProjectionView {
    let frame = UiMountedFrameIdentity::mint_unbound().expect("frame identity");
    let surface = UiSemanticSurfaceIdentity::mint_unbound().expect("surface identity");
    let binding = UiSurfaceBindingGeneration::mint_unbound().expect("binding generation");
    let content_generation =
        UiMountedContentGeneration::mint_unbound().expect("content generation");
    let instance = UiMountedInstanceIdentity::mint_unbound().expect("mounted instance");
    let bounds = canonical_bounds();
    let allocation = allocation_basis(2);
    let row_frame = mutate_identity(
        frame,
        mutation,
        UiSemanticTextProjectionCertificationMutation::ForeignFrame,
    );
    let row_instance = mutate_identity(
        instance,
        mutation,
        UiSemanticTextProjectionCertificationMutation::ForeignInstance,
    );
    let row_receipt = node_receipt(row_frame, row_instance);
    let row = semantic_row(SemanticTextRowBasis {
        frame: row_frame,
        surface: mutate_identity(
            surface,
            mutation,
            UiSemanticTextProjectionCertificationMutation::ForeignSurface,
        ),
        binding: mutate_identity(
            binding,
            mutation,
            UiSemanticTextProjectionCertificationMutation::ForeignBinding,
        ),
        content_generation: mutate_identity(
            content_generation,
            mutation,
            UiSemanticTextProjectionCertificationMutation::ForeignContentGeneration,
        ),
        instance: row_instance,
        receipt: row_receipt,
        allocation,
        bounds,
        capability_generation,
        capability_profile_digest,
        mutation,
        text,
    });
    let node_receipt = if matches!(
        mutation,
        UiSemanticTextProjectionCertificationMutation::ForeignFrame
            | UiSemanticTextProjectionCertificationMutation::ForeignInstance
            | UiSemanticTextProjectionCertificationMutation::ForeignNodeReceipt
    ) {
        node_receipt(frame, instance)
    } else {
        row_receipt
    };
    let references = match mutation {
        UiSemanticTextProjectionCertificationMutation::MissingReference => {
            vec![UiMountedSemanticTextReference::from_runtime_mounting(1)]
        }
        UiSemanticTextProjectionCertificationMutation::UnreferencedRow => Vec::new(),
        UiSemanticTextProjectionCertificationMutation::DuplicateReference => {
            vec![text_reference(), text_reference()]
        }
        _ => vec![text_reference()],
    };
    projection(SemanticTextProjectionBasis {
        frame,
        surface,
        binding,
        content_generation,
        instance,
        node_receipt,
        allocation: if mutation == UiSemanticTextProjectionCertificationMutation::ForeignAllocation
        {
            allocation_basis(3)
        } else {
            allocation
        },
        bounds,
        references,
        row,
        paint_admitted: mutation != UiSemanticTextProjectionCertificationMutation::WithheldPaint,
    })
}

struct SemanticTextProjectionBasis {
    frame: UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    content_generation: UiMountedContentGeneration,
    instance: UiMountedInstanceIdentity,
    node_receipt: UiMountedNodeReceiptIdentity,
    allocation: UiMountedAllocationBasis,
    bounds: UiMountedCanonicalBox,
    references: Vec<UiMountedSemanticTextReference>,
    row: UiMountedSemanticTextMechanic,
    paint_admitted: bool,
}

fn projection(basis: SemanticTextProjectionBasis) -> UiMountedProjectionView {
    let nodes = vec![node(&basis)];
    let rows = vec![basis.row];
    let (authored_paint_commands, authored_paint_order) =
        crate::mounting::compile_presentation_sources(&nodes, &[], &rows);
    UiMountedProjectionView::new(UiMountedProjectionViewInput {
        frame: basis.frame,
        surface: basis.surface,
        binding: basis.binding,
        content_generation: basis.content_generation,
        nodes,
        clips: worth_ui_host_contract::UiMountedClipTable::produced(Vec::new()),
        layers: worth_ui_host_contract::UiMountedLayerTable::produced(Vec::new()),
        portal_overlays: worth_ui_host_contract::UiMountedPortalOverlayTable::empty(),
        semantic_text: UiMountedSemanticTextTable::from_runtime_mounting(rows)
            .expect("one semantic row fits the mounted table"),
        hit_tests: worth_ui_host_contract::UiMountedHitTestTable::empty(),
        paint_batches: UiMountedPaintBatchTable::new(Vec::new()),
        spatial_batches: UiMountedSpatialBatchTable::new(Vec::new()),
        realtime_batches: UiMountedRealtimeBatchTable::new(Vec::new()),
        resources: UiMountedResourceTable::new(Vec::new()),
        authored_paint_commands,
        authored_paint_order,
    })
}

fn semantic_row(input: SemanticTextRowBasis) -> UiMountedSemanticTextMechanic {
    let text_len = u32::try_from(input.text.len()).expect("certification text fits u32");
    UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
        UiMountedSemanticTextCompletionInput {
            content_generation: input.content_generation,
            frame: input.frame,
            surface: input.surface,
            binding: input.binding,
            mounted_instance: input.instance,
            portal_group: None,
            node_receipt: input.receipt,
            allocation_basis: input.allocation,
            bounds: input.bounds,
            clip_bounds: input.bounds,
            origin_x: 8.0,
            origin_y: 12.0,
            text: Arc::clone(&input.text),
            layout: crate::mounting::qualified_text_test_support::inert_qualified_layout(
                &input.text,
            )
            .view(),
            slot: UiSemanticTextSlot::Value,
            collection_row: None,
            foregrounds: Arc::from([UiMountedTextForegroundSpan::from_runtime_mounting(
                worth_ui_host_contract::UiTextOriginalRange::from_text_mechanics(0, text_len)
                    .unwrap(),
                UiMountedRgba8::new(255, 255, 255, 255),
                UiMountedTextPaintSpanIdentity::from_runtime_mounting([1; 32]),
            )]),
            profile: UiSemanticTextProfile::BodyDefault,
            layer_semantic_order: 1,
            capability_generation: if input.mutation
                == UiSemanticTextProjectionCertificationMutation::ForeignCapabilityGeneration
            {
                WorthUiHostCapabilityObservationGeneration::new(
                    input.capability_generation.as_u64().wrapping_add(1),
                )
            } else {
                input.capability_generation
            },
            capability_profile_digest: if input.mutation
                == UiSemanticTextProjectionCertificationMutation::ForeignCapabilityProfile
            {
                input.capability_profile_digest.wrapping_add(1)
            } else {
                input.capability_profile_digest
            },
        },
    )
    .expect("certification semantic row is structurally valid")
}

fn node(basis: &SemanticTextProjectionBasis) -> UiMountedNodeProjectionView {
    let admitted = UiMountedParticipationFact::new(UiMountedParticipationStatus::Admitted);
    let withheld = UiMountedParticipationFact::new(UiMountedParticipationStatus::Withheld);
    let omitted = UiMountedOmissionReason::NotDefinedByCurrentRuntime;
    UiMountedNodeProjectionView::new(UiMountedNodeProjectionViewInput {
        mounted_instance: basis.instance,
        node_receipt: basis.node_receipt,
        authored_position: 0,
        role: UiMountedMechanicalRole::Control,
        participation: UiMountedParticipation::new(UiMountedParticipationInput {
            paint: if basis.paint_admitted {
                admitted
            } else {
                withheld
            },
            clip: admitted,
            input: withheld,
            focus: withheld,
            hit_test: withheld,
            accessibility: withheld,
            motion: withheld,
            diagnostic: withheld,
        }),
        allocation: UiMountedAllocationProjection::Known {
            bounds: basis.bounds,
            basis: basis.allocation,
        },
        preview: UiMountedPreviewProjection::Omitted(omitted),
        paint: UiMountedPaintProjection::Omitted(omitted),
        hit_test: worth_ui_host_contract::UiMountedHitTestProjection::Omitted(omitted),
        accessibility: UiMountedAccessibilityProjection::Omitted(omitted),
        motion: UiMountedMotionProjection::Omitted(omitted),
        diagnostic: UiMountedDiagnosticProjection::Omitted(omitted),
        drawables: basis
            .references
            .iter()
            .copied()
            .map(worth_ui_host_contract::UiMountedDrawableReference::SemanticText)
            .collect(),
        semantic_text: basis.references.clone(),
        portal_presentation: None,
    })
}

fn canonical_bounds() -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 0.0,
        y: 0.0,
        width: 160.0,
        height: 96.0,
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .expect("certification bounds")
}

fn allocation_basis(generation: u64) -> UiMountedAllocationBasis {
    UiMountedAllocationBasis::new(1, generation, 3, UiMountedTransformProjection::Identity)
}

fn node_receipt(
    frame: UiMountedFrameIdentity,
    instance: UiMountedInstanceIdentity,
) -> UiMountedNodeReceiptIdentity {
    UiMountedNodeReceiptIssuer::mint_for(frame)
        .expect("receipt issuer")
        .receipt_for(instance)
}

fn text_reference() -> UiMountedSemanticTextReference {
    UiMountedSemanticTextReference::from_runtime_mounting(0)
}

fn mutate_identity<T: MintIdentity>(
    value: T,
    mutation: UiSemanticTextProjectionCertificationMutation,
    target: UiSemanticTextProjectionCertificationMutation,
) -> T {
    if mutation == target {
        T::mint()
    } else {
        value
    }
}

trait MintIdentity: Copy {
    fn mint() -> Self;
}

macro_rules! mint_identity {
    ($($identity:ty),+ $(,)?) => {
        $(impl MintIdentity for $identity {
            fn mint() -> Self {
                Self::mint_unbound().expect("foreign certification identity")
            }
        })+
    };
}

mint_identity!(
    UiMountedFrameIdentity,
    UiSemanticSurfaceIdentity,
    UiSurfaceBindingGeneration,
    UiMountedContentGeneration,
    UiMountedInstanceIdentity,
);
