use crate::capability::UiRuntimeServiceFamily;

#[test]
fn each_private_declaration_contract_names_exactly_one_service_owner() {
    assert_eq!(
        [
            super::portal::UiDeclaredPortalSurfaceContract::MountedOverlay.family(),
            super::focus::UiDeclaredFocusOwnershipContract::SemanticKeyboardFocus.family(),
            super::motion::UiDeclaredMotionPolicyContract::ReducedMotionAware.family(),
            super::command_routing::UiDeclaredCommandRoutingContract::TypedInvocation.family(),
            super::scroll::UiDeclaredScrollOwnershipContract::RuntimeOwnedOffset.family(),
            super::selection::UiDeclaredSelectionIdentityContract::StableItemKey.family(),
        ],
        UiRuntimeServiceFamily::ALL
    );
}

#[test]
fn declared_usage_classification_is_complete_without_new_dsl_tokens() {
    use super::super::UiDeclaredServiceUsagePosture as Posture;

    assert_eq!(
        [
            Posture::Portal.family(),
            Posture::Focus.family(),
            Posture::Motion.family(),
            Posture::CommandRouting.family(),
            Posture::Scroll.family(),
            Posture::Selection.family(),
        ],
        UiRuntimeServiceFamily::ALL
    );
}

#[test]
fn declaration_geometry_is_a_checked_portal_preference_not_runtime_geometry() {
    let geometry = super::portal::UiDeclaredPortalPlacementGeometry::checked(280, 320, 8, 16)
        .expect("valid declaration placement preference");

    assert_eq!(geometry.preferred_width(), 280);
    assert_eq!(geometry.maximum_height(), 320);
    assert_eq!(geometry.anchor_gap(), 8);
    assert_eq!(geometry.viewport_margin(), 16);
    let modal = super::portal::UiDeclaredPortalPlacementGeometry::modal_dialog();
    assert_eq!(modal.preferred_width(), 280);
    assert_eq!(modal.maximum_height(), 320);
    assert_eq!(modal.anchor_gap(), 8);
    assert_eq!(modal.viewport_margin(), 24);
    assert_eq!(
        super::portal::UiDeclaredPortalPlacementGeometry::checked(0, 320, 8, 16),
        Err(super::portal::UiDeclaredPortalPlacementGeometryDenial::EmptyExtent)
    );
}
