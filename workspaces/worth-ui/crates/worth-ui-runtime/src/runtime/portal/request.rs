#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiPortalServiceRequest {
    portal: super::UiPortalIdentity,
    idempotency: crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity,
    operation: UiPortalServiceOperation,
    presented_anchor: Option<crate::runtime::interaction::UiPresentedInteractionGeometry>,
    presented_viewport: Option<crate::runtime::interaction::UiPresentedViewportGeometry>,
    placement_geometry: Option<crate::declaration::UiDeclaredPortalPlacementGeometry>,
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    declared_portal: Option<worth_ui_dsl::UiPortalDeclarationId>,
    parent: Option<super::UiPortalIdentity>,
    shielding: super::UiPortalInputShielding,
    shielding_uses_policy_default: bool,
}

// Presented viewport boxes are canonical finite geometry, so equality is reflexive.
impl Eq for UiPortalServiceRequest {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPortalServiceOperation {
    Open,
    Close(super::UiPortalDismissalCause),
}

impl UiPortalServiceRequest {
    pub(crate) const fn open(
        portal: super::UiPortalIdentity,
        idempotency: crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity,
        presented_anchor: crate::runtime::interaction::UiPresentedInteractionGeometry,
        presented_viewport: Option<crate::runtime::interaction::UiPresentedViewportGeometry>,
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Self {
        Self {
            portal,
            idempotency,
            operation: UiPortalServiceOperation::Open,
            presented_anchor: Some(presented_anchor),
            presented_viewport,
            placement_geometry: Some(
                crate::declaration::UiDeclaredPortalPlacementGeometry::dropdown(),
            ),
            semantic_surface,
            declared_portal: None,
            parent: None,
            shielding: super::UiPortalInputShielding::ContentBounds,
            shielding_uses_policy_default: true,
        }
    }

    pub(crate) const fn open_nested(
        portal: super::UiPortalIdentity,
        idempotency: crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity,
        presented_anchor: crate::runtime::interaction::UiPresentedInteractionGeometry,
        presented_viewport: crate::runtime::interaction::UiPresentedViewportGeometry,
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        parent: super::UiPortalIdentity,
        shielding: super::UiPortalInputShielding,
    ) -> Self {
        Self {
            portal,
            idempotency,
            operation: UiPortalServiceOperation::Open,
            presented_anchor: Some(presented_anchor),
            presented_viewport: Some(presented_viewport),
            placement_geometry: Some(
                crate::declaration::UiDeclaredPortalPlacementGeometry::dropdown(),
            ),
            semantic_surface,
            declared_portal: None,
            parent: Some(parent),
            shielding,
            shielding_uses_policy_default: false,
        }
    }

    pub(crate) const fn close(
        portal: super::UiPortalIdentity,
        idempotency: crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity,
        cause: super::UiPortalDismissalCause,
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Self {
        Self {
            portal,
            idempotency,
            operation: UiPortalServiceOperation::Close(cause),
            presented_anchor: None,
            presented_viewport: None,
            placement_geometry: None,
            semantic_surface,
            declared_portal: None,
            parent: None,
            shielding: super::UiPortalInputShielding::ContentBounds,
            shielding_uses_policy_default: false,
        }
    }

    pub(crate) const fn portal(self) -> super::UiPortalIdentity {
        self.portal
    }

    pub(crate) const fn idempotency(
        self,
    ) -> crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity {
        self.idempotency
    }

    pub(crate) const fn operation(self) -> UiPortalServiceOperation {
        self.operation
    }

    pub(crate) const fn presented_anchor(
        self,
    ) -> Option<crate::runtime::interaction::UiPresentedInteractionGeometry> {
        self.presented_anchor
    }

    pub(crate) const fn presented_viewport(
        self,
    ) -> Option<crate::runtime::interaction::UiPresentedViewportGeometry> {
        self.presented_viewport
    }

    pub(crate) const fn placement_geometry(
        self,
    ) -> Option<crate::declaration::UiDeclaredPortalPlacementGeometry> {
        self.placement_geometry
    }

    pub(crate) const fn parent(self) -> Option<super::UiPortalIdentity> {
        self.parent
    }

    pub(crate) const fn semantic_surface(
        self,
    ) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.semantic_surface
    }

    pub(crate) const fn declared_portal(self) -> Option<worth_ui_dsl::UiPortalDeclarationId> {
        self.declared_portal
    }

    pub(crate) const fn with_declared_portal(
        mut self,
        declared_portal: Option<worth_ui_dsl::UiPortalDeclarationId>,
    ) -> Self {
        self.declared_portal = declared_portal;
        if declared_portal.is_some() {
            self.shielding_uses_policy_default = true;
        }
        self
    }

    pub(crate) const fn shielding(self) -> super::UiPortalInputShielding {
        self.shielding
    }

    pub(super) const fn with_policy(mut self, policy: crate::declaration::UiPortalPolicy) -> Self {
        if matches!(self.operation, UiPortalServiceOperation::Open)
            && self.shielding_uses_policy_default
        {
            self.shielding = match policy.kind() {
                crate::declaration::UiPortalPolicyKind::ModalDialog => {
                    super::UiPortalInputShielding::ModalSurface
                }
                crate::declaration::UiPortalPolicyKind::Dropdown
                | crate::declaration::UiPortalPolicyKind::Popover => {
                    super::UiPortalInputShielding::ContentBounds
                }
            };
        }
        self
    }
}
