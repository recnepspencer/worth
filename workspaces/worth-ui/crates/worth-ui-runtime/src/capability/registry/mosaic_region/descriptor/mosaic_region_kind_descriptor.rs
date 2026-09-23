use crate::capability::{MosaicRegionKindId, SurfacePlacementClass};

use super::{
    MosaicChildRule, MosaicClippingPosture, MosaicFocusScopeKind, MosaicHitTestPosture,
    MosaicRegionPersistence, MosaicRegionRole, MosaicScrollOwnership, MosaicSizingBehavior,
    UiScrollChromeContract, UiScrollLineExtent,
};

/// Declarative mosaic-owned structural region kind supplied by an application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MosaicRegionKindDescriptor {
    id: MosaicRegionKindId,
    role: MosaicRegionRole,
    sizing_behavior: Option<MosaicSizingBehavior>,
    scroll_ownership: Option<MosaicScrollOwnership>,
    focus_scope: Option<MosaicFocusScopeKind>,
    child_rule: Option<MosaicChildRule>,
    allowed_surface_classes: Vec<SurfacePlacementClass>,
    persistence: Option<MosaicRegionPersistence>,
    clipping: Option<MosaicClippingPosture>,
    hit_test: Option<MosaicHitTestPosture>,
    scroll_chrome: Option<UiScrollChromeContract>,
    scroll_line_extent: Option<UiScrollLineExtent>,
    label: Option<String>,
}

impl MosaicRegionKindDescriptor {
    pub fn new(id: MosaicRegionKindId, role: MosaicRegionRole) -> Self {
        Self {
            id,
            role,
            sizing_behavior: None,
            scroll_ownership: None,
            focus_scope: None,
            child_rule: None,
            allowed_surface_classes: Vec::new(),
            persistence: None,
            clipping: None,
            hit_test: None,
            scroll_chrome: None,
            scroll_line_extent: None,
            label: None,
        }
    }

    pub fn with_sizing_behavior(mut self, sizing_behavior: MosaicSizingBehavior) -> Self {
        self.sizing_behavior = Some(sizing_behavior);
        self
    }

    pub fn with_scroll_ownership(mut self, scroll_ownership: MosaicScrollOwnership) -> Self {
        self.scroll_ownership = Some(scroll_ownership);
        self
    }

    pub fn with_focus_scope(mut self, focus_scope: MosaicFocusScopeKind) -> Self {
        self.focus_scope = Some(focus_scope);
        self
    }

    pub fn with_child_rule(mut self, child_rule: MosaicChildRule) -> Self {
        self.child_rule = Some(child_rule);
        self
    }

    pub fn with_allowed_surface_class(mut self, surface_class: SurfacePlacementClass) -> Self {
        self.allowed_surface_classes.push(surface_class);
        self
    }

    pub fn with_persistence(mut self, persistence: MosaicRegionPersistence) -> Self {
        self.persistence = Some(persistence);
        self
    }

    pub fn with_clipping(mut self, clipping: MosaicClippingPosture) -> Self {
        self.clipping = Some(clipping);
        self
    }

    pub fn with_hit_test(mut self, hit_test: MosaicHitTestPosture) -> Self {
        self.hit_test = Some(hit_test);
        self
    }

    pub fn with_scroll_chrome(mut self, contract: UiScrollChromeContract) -> Self {
        self.scroll_chrome = Some(contract);
        self
    }

    pub const fn with_scroll_line_extent(mut self, extent: UiScrollLineExtent) -> Self {
        self.scroll_line_extent = Some(extent);
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn id(&self) -> &MosaicRegionKindId {
        &self.id
    }

    pub fn role(&self) -> &MosaicRegionRole {
        &self.role
    }

    pub fn sizing_behavior(&self) -> Option<&MosaicSizingBehavior> {
        self.sizing_behavior.as_ref()
    }

    pub fn scroll_ownership(&self) -> Option<&MosaicScrollOwnership> {
        self.scroll_ownership.as_ref()
    }

    pub fn focus_scope(&self) -> Option<&MosaicFocusScopeKind> {
        self.focus_scope.as_ref()
    }

    pub fn child_rule(&self) -> Option<&MosaicChildRule> {
        self.child_rule.as_ref()
    }

    pub fn allowed_surface_classes(&self) -> &[SurfacePlacementClass] {
        &self.allowed_surface_classes
    }

    pub fn persistence(&self) -> Option<&MosaicRegionPersistence> {
        self.persistence.as_ref()
    }

    pub fn clipping(&self) -> Option<&MosaicClippingPosture> {
        self.clipping.as_ref()
    }

    pub fn hit_test(&self) -> Option<&MosaicHitTestPosture> {
        self.hit_test.as_ref()
    }

    pub fn scroll_chrome(&self) -> Option<&UiScrollChromeContract> {
        self.scroll_chrome.as_ref()
    }

    pub const fn scroll_line_extent(&self) -> Option<UiScrollLineExtent> {
        self.scroll_line_extent
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}
