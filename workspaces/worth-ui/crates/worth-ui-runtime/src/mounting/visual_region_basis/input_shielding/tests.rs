use super::super::UiMountedVisualRegionBasis;
use crate::runtime::portal::UiPortalStackOrdinal;
use worth_ui_host_contract::*;

struct Stack {
    frame: UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    host: UiHostSurfaceIdentity,
}

impl Stack {
    fn new() -> Self {
        Self {
            frame: UiMountedFrameIdentity::mint_unbound().unwrap(),
            surface: UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            binding: UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            host: UiHostSurfaceIdentity::mint_unbound().unwrap(),
        }
    }

    fn portal(
        &self,
        owner: UiMountedInstanceIdentity,
        lifecycle: UiMountedPortalOverlayLifecyclePosture,
        shielding: UiMountedPortalInputShielding,
    ) -> UiMountedPortalOverlayMechanic {
        let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 24.0,
            coordinate_space: UiMountedCoordinateSpace::Viewport,
        })
        .unwrap();
        UiMountedPortalOverlayMechanic::complete_from_runtime_mounting(
            UiMountedPortalOverlayCompletionInput {
                frame: self.frame,
                surface: self.surface,
                binding: self.binding,
                owner,
                owner_receipt: UiMountedNodeReceiptIssuer::mint_for(self.frame)
                    .unwrap()
                    .receipt_for(owner),
                portal_identity: owner.diagnostic_value(),
                anchor_presentation: UiHostObservationPresentationBasis::new(
                    self.host,
                    self.frame,
                    self.binding,
                    UiHostPresentationEpoch::issued_by_host(1),
                ),
                anchor_bounds: bounds,
                bounds,
                paint_bounds: bounds,
                clip_bounds: bounds,
                color: UiMountedRgba8::new(255, 255, 255, 255),
                layer_semantic_order: 0,
                layer_depth: 0,
                lifecycle,
                shielding,
            },
        )
        .unwrap()
    }
}

/// A menu opened above a modal closes. The modal still sets the same floor,
/// but the menu no longer admits input, so the admission differs.
#[test]
fn a_portal_above_a_modal_that_starts_closing_changes_what_the_modal_admits() {
    use UiMountedPortalInputShielding::{ContentBounds, ModalSurface};
    use UiMountedPortalOverlayLifecyclePosture::{Closing, Visible};
    let stack = Stack::new();
    let modal = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let menu = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let order = std::collections::BTreeMap::from([
        (
            modal.diagnostic_value(),
            UiPortalStackOrdinal::minted_for_test(1),
        ),
        (
            menu.diagnostic_value(),
            UiPortalStackOrdinal::minted_for_test(2),
        ),
    ]);
    let basis = |menu_lifecycle| {
        UiMountedVisualRegionBasis::new(Box::new([]), Box::new([])).with_portal_overlays(
            vec![
                stack.portal(modal, Visible, ModalSurface),
                stack.portal(menu, menu_lifecycle, ContentBounds),
            ],
            order.clone(),
        )
    };
    let (open, closing) = (basis(Visible), basis(Closing));
    let mut work = Default::default();
    assert_eq!(
        open.modal_input_floor(stack.binding, &mut work),
        closing.modal_input_floor(stack.binding, &mut work)
    );
    assert!(open.modal_input_admission(stack.binding).is_some());
    assert_ne!(
        open.modal_input_admission(stack.binding),
        closing.modal_input_admission(stack.binding)
    );
    assert_eq!(
        open.modal_input_admission(stack.binding),
        basis(Visible).modal_input_admission(stack.binding)
    );
}

#[test]
fn no_modal_admits_every_row() {
    let stack = Stack::new();
    let menu = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let basis = UiMountedVisualRegionBasis::new(Box::new([]), Box::new([])).with_portal_overlays(
        vec![stack.portal(
            menu,
            UiMountedPortalOverlayLifecyclePosture::Visible,
            UiMountedPortalInputShielding::ContentBounds,
        )],
        std::collections::BTreeMap::from([(
            menu.diagnostic_value(),
            UiPortalStackOrdinal::minted_for_test(1),
        )]),
    );
    assert_eq!(basis.modal_input_admission(stack.binding), None);
}
