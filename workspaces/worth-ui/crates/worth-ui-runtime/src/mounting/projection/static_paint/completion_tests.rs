use worth_ui_host_contract::{
    UiMountIncarnation, UiMountedAllocationBasis, UiMountedAllocationProjection,
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedMechanicalRole,
    UiMountedOmissionReason, UiMountedParticipation, UiMountedParticipationFact,
    UiMountedParticipationInput, UiMountedParticipationStatus, UiMountedProjectionAudience,
    UiMountedRgba8, UiMountedTransformProjection, UiSemanticSurfaceIdentity,
    UiSurfaceBindingGeneration,
};

use super::super::frame_storage::{
    UiMountedProjectionNodeRecord, UiMountedProjectionSurface, UiMountedSemanticProjection,
};
use super::super::node_receipt::{UiMountedNodeReceipt, UiMountedNodeReceiptInput};
use super::super::UiMountedProjectionDenial;
use super::{complete_static_filled_rects, UiMountedStaticPaintSeed};

#[test]
fn missing_and_unsupported_allocation_deny_with_the_exact_graph_node() {
    let node = crate::graph::UiGraphNodeIdentity::new(41);
    for (allocation, expected) in [
        (
            UiMountedAllocationProjection::Omitted(UiMountedOmissionReason::NoCommittedAllocation),
            UiMountedProjectionDenial::MissingStaticPaintAllocation(node),
        ),
        (
            UiMountedAllocationProjection::PortalAnchorObservation {
                bounds: canonical_bounds(),
                basis: allocation_basis(),
            },
            UiMountedProjectionDenial::UnsupportedStaticPaintAllocation(node),
        ),
    ] {
        let fixture =
            CompletionFixture::new(node, allocation, admitted_participation(), true, false);
        assert_eq!(fixture.complete(), Err(expected));
    }
}

#[test]
fn withheld_participation_and_foreign_receipt_basis_deny_before_completion() {
    let node = crate::graph::UiGraphNodeIdentity::new(42);
    let fixture = CompletionFixture::new(
        node,
        UiMountedAllocationProjection::Known {
            bounds: canonical_bounds(),
            basis: allocation_basis(),
        },
        withheld_paint_participation(),
        true,
        false,
    );
    assert_eq!(
        fixture.complete(),
        Err(UiMountedProjectionDenial::StaticPaintParticipationWithheld(
            node
        ))
    );

    let fixture = CompletionFixture::new(
        node,
        UiMountedAllocationProjection::Known {
            bounds: canonical_bounds(),
            basis: allocation_basis(),
        },
        admitted_participation(),
        false,
        false,
    );
    assert_eq!(
        fixture.complete(),
        Err(UiMountedProjectionDenial::StaticPaintNodeReceiptMismatch)
    );
}

#[test]
fn semantic_text_does_not_suppress_a_declared_static_fill() {
    let fixture = CompletionFixture::new(
        crate::graph::UiGraphNodeIdentity::new(43),
        UiMountedAllocationProjection::Known {
            bounds: canonical_bounds(),
            basis: allocation_basis(),
        },
        admitted_participation(),
        true,
        true,
    );

    let rows = fixture
        .complete()
        .expect("paint and text complete independently");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].color(), UiMountedRgba8::new(47, 129, 247, 255));
}

struct CompletionFixture {
    frame: UiMountedFrameIdentity,
    receipt_basis: super::super::super::UiMountedNodeReceiptBasis,
    semantic: UiMountedSemanticProjection,
}

impl CompletionFixture {
    fn new(
        graph_node: crate::graph::UiGraphNodeIdentity,
        allocation: UiMountedAllocationProjection,
        participation: UiMountedParticipation,
        include_instance_in_receipt_basis: bool,
        include_semantic_text: bool,
    ) -> Self {
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let mounted_instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let receipt = UiMountedNodeReceipt::from_input(UiMountedNodeReceiptInput {
            mounted_instance,
            graph_node,
            semantic_surface: surface,
            incarnation: UiMountIncarnation::mint_unbound().unwrap(),
            plan_digest: 7,
            role: UiMountedMechanicalRole::Control,
            participation,
            allocation,
        });
        let semantic = UiMountedSemanticProjection::initial(
            vec![UiMountedProjectionNodeRecord {
                receipt,
                plan_index: Some(0),
        occurrence_allocation: allocation,
        appearance_geometry:
                    crate::mounting::projection::frame_storage::UiMountedAppearanceGeometry::from_occurrence(
                        allocation,
                        crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
                    ),
        surface_paint_order: Some(0),
        has_appearance_attachment: false,
                appearance_clip:
                    crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
                static_paint: Some(UiMountedStaticPaintSeed::for_test(UiMountedRgba8::new(
                    47, 129, 247, 255,
                ))),
                semantic_text: include_semantic_text
                    .then(super::super::semantic_text::UiMountedSemanticTextSeed::scalar_for_test),
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
        );
        let mut presented = crate::runtime::persistent_index::UiPersistentOrdSet::default();
        if include_instance_in_receipt_basis {
            presented.insert(mounted_instance);
        }
        let receipt_basis =
            super::super::super::UiMountedNodeReceiptBasis::mint(frame, presented).unwrap();
        Self {
            frame,
            receipt_basis,
            semantic,
        }
    }

    fn complete(
        &self,
    ) -> Result<Vec<worth_ui_host_contract::UiMountedFilledRectMechanic>, UiMountedProjectionDenial>
    {
        complete_static_filled_rects(self.frame, &self.receipt_basis, &self.semantic)
    }
}

fn admitted_participation() -> UiMountedParticipation {
    participation(UiMountedParticipationStatus::Admitted)
}

fn withheld_paint_participation() -> UiMountedParticipation {
    participation(UiMountedParticipationStatus::Withheld)
}

fn participation(paint: UiMountedParticipationStatus) -> UiMountedParticipation {
    let admitted = UiMountedParticipationFact::new(UiMountedParticipationStatus::Admitted);
    let withheld = UiMountedParticipationFact::new(UiMountedParticipationStatus::Withheld);
    UiMountedParticipation::new(UiMountedParticipationInput {
        paint: UiMountedParticipationFact::new(paint),
        clip: admitted,
        input: withheld,
        focus: withheld,
        hit_test: withheld,
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

fn allocation_basis() -> UiMountedAllocationBasis {
    UiMountedAllocationBasis::new(1, 2, 3, UiMountedTransformProjection::Identity)
}
