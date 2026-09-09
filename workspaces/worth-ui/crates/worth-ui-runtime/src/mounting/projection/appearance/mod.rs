// Gate 1 deliberately keeps this authority unpublished until a later cutover.

mod backdrop;
mod clip;
mod counters;
mod damage;
mod delta;
mod fact;
mod geometry;
mod geometry_input;
pub(crate) use geometry_input::UiMountedAppearanceGeometryInput;
mod geometry_scope;
mod input;
mod lowering;
mod mechanic_equivalence;
mod outline;
mod overlay_input;
mod overlay_order;
mod portal_geometry;
mod reconstruction;
mod resolved_node_source;
mod style;
mod surface;
mod text_foreground;
mod text_geometry;
pub(crate) use text_geometry::{UiMountedAppearanceTextGeometry, UiMountedAppearanceTextSpanInput};

pub(crate) use clip::{
    derive_unbound_ancestry, UiMountedAppearanceClip, UiMountedAppearanceClipDenial,
};
pub(crate) use delta::UiMountedAppearanceDeltaSummary;
pub(crate) use fact::{
    UiMountedAppearanceFacts, UiMountedAppearanceLoweringInput, UiMountedAppearanceNodeInput,
    UiMountedAppearanceSurfaceOverlayInput,
};
pub(crate) use geometry::UiMountedAppearanceGeometryDenial;
pub(crate) use geometry_scope::UiMountedAppearanceGeometryScope;
pub(in crate::mounting::projection) use portal_geometry::{
    portal_ancestor_clip, portal_presented_allocation,
};
pub(crate) use resolved_node_source::UiResolvedAppearanceNodeSource;
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedAppearanceLoweringDenial {
    NodeSessionMismatch,
    HostGeometryProfileUnavailable,
    HostGeometrySurfaceUnavailable,
    HostGeometryScale(worth_ui_host_contract::UiHostAppearanceScaleDenial),
    NodeAllocationUnavailable,
    BorderWidthInvalid,
    RadiusInvalid,
    OutlineWidthInvalid,
    OutlineOffsetInvalid,
    OutlineGeometry(worth_ui_host_contract::UiAppearanceOutlineGeometryDenial),
    SurfacePaintOrderUnavailable,
    AncestorClip(UiMountedAppearanceClipDenial),
    Geometry(UiMountedAppearanceGeometryDenial),
    NodeProjectionUnavailable,
    NodeReceiptFrameMismatch,
    NodeProjectionIssuerMismatch,
    NodeSurfaceMismatch,
    PortalSurfaceMissing,
    PortalTargetMismatch,
    OutlineAllocationMismatch,
    BackdropPlacementMismatch,
    OverlayRevisionMissing,
    OrderParticipantMissing,
    Surface(worth_ui_host_contract::UiMountedSurfaceAppearanceCompletionDenial),
    PortalSurface(worth_ui_host_contract::UiMountedPortalSurfaceAppearanceCompletionDenial),
    Outline(worth_ui_host_contract::UiMountedOutlineAppearanceCompletionDenial),
    TextForeground(worth_ui_host_contract::UiMountedTextForegroundAppearanceCompletionDenial),
    Backdrop(worth_ui_host_contract::UiMountedBackdropCompletionDenial),
    OverlayOrder(worth_ui_host_contract::UiMountedOverlayOrderMechanicDenial),
    Frame(worth_ui_host_contract::UiMountedAppearanceFrameDenial),
    WorkConstruction,
    AmbiguousMotionOpacity,
}

impl UiMountedAppearanceLoweringInput {
    pub(crate) fn empty(
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> Self {
        Self {
            frame,
            semantic_surface,
            presentation,
            nodes: Vec::new(),
            backdrops: Vec::new(),
            overlay: fact::UiMountedAppearanceOverlayInput {
                semantic_surface,
                presentation,
                portal_revision: 0,
                backdrop_revision: 0,
                bottom_to_top: Box::new([]),
            },
        }
    }

    pub(crate) fn for_node(
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        node: Option<UiMountedAppearanceNodeInput>,
    ) -> Self {
        let mut input = Self::empty(frame, semantic_surface, presentation);
        input.nodes.extend(node);
        input
    }

    pub(in crate::mounting::projection) fn into_nodes(self) -> Vec<UiMountedAppearanceNodeInput> {
        self.nodes
    }
}

impl UiMountedAppearanceSurfaceOverlayInput {
    pub(in crate::mounting::projection) fn lowering_input(
        &self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        nodes: Vec<UiMountedAppearanceNodeInput>,
    ) -> UiMountedAppearanceLoweringInput {
        UiMountedAppearanceLoweringInput {
            frame,
            semantic_surface: self.semantic_surface,
            presentation,
            nodes,
            backdrops: self.backdrops.to_vec(),
            overlay: fact::UiMountedAppearanceOverlayInput {
                semantic_surface: self.semantic_surface,
                presentation,
                portal_revision: self.portal_revision,
                backdrop_revision: self.backdrop_revision,
                bottom_to_top: self.bottom_to_top.clone(),
            },
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct UiMountedAppearanceSidecar {
    current: Option<UiMountedAppearanceFacts>,
    counters: counters::UiMountedAppearanceCounters,
    last_delta: Option<UiMountedAppearanceDeltaSummary>,
}

impl UiMountedAppearanceSidecar {
    pub(in crate::mounting::projection) fn matches_geometry_input(
        &self,
        input: &UiMountedAppearanceGeometryInput,
    ) -> bool {
        self.current
            .as_ref()
            .is_some_and(|facts| facts.matches_geometry_input(input))
    }

    pub(crate) fn removal_work(
        &self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> Result<worth_ui_host_contract::UiMountedAppearanceWork, UiMountedAppearanceLoweringDenial>
    {
        let current = self
            .current
            .as_ref()
            .ok_or(UiMountedAppearanceLoweringDenial::WorkConstruction)?;
        let empty = lowering::lower(UiMountedAppearanceLoweringInput::empty(
            frame,
            current.frame().semantic_surface(),
            presentation,
        ))?;
        delta::work(Some(current), &empty).map(|delta| delta.work)
    }

    pub(crate) fn current_node_receipt(
        &self,
    ) -> Option<worth_ui_host_contract::UiMountedNodeReceiptIdentity> {
        self.current
            .as_ref()?
            .records()
            .iter()
            .find_map(|fact| fact.node_receipt())
    }

    pub(crate) const fn has_current(&self) -> bool {
        self.current.is_some()
    }

    pub(crate) fn mount(
        &mut self,
        input: UiMountedAppearanceLoweringInput,
    ) -> Result<worth_ui_host_contract::UiMountedAppearanceWork, UiMountedAppearanceLoweringDenial>
    {
        let successor = lowering::lower(input)?;
        let delta = delta::work(self.current.as_ref(), &successor)?;
        self.counters.observe(&delta.work, delta.summary);
        self.last_delta = Some(delta.summary);
        self.current = Some(successor);
        Ok(delta.work)
    }

    pub(crate) fn reconstruct(
        &mut self,
        input: UiMountedAppearanceLoweringInput,
    ) -> Result<worth_ui_host_contract::UiMountedAppearanceWork, UiMountedAppearanceLoweringDenial>
    {
        reconstruction::rebuild(self.current.as_ref(), input).map(|(delta, facts)| {
            self.counters.observe(&delta.work, delta.summary);
            self.last_delta = Some(delta.summary);
            self.current = Some(facts);
            delta.work
        })
    }

    #[cfg(test)]
    pub(crate) fn current(&self) -> Option<&UiMountedAppearanceFacts> {
        self.current.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn current_frame_identity(
        &self,
    ) -> Option<worth_ui_host_contract::UiMountedFrameIdentity> {
        self.current.as_ref().map(|facts| facts.frame().frame())
    }

    #[cfg(test)]
    pub(crate) fn current_overlay_order(
        &self,
    ) -> Option<Box<[worth_ui_host_contract::UiOverlayParticipantIdentity]>> {
        self.current.as_ref().map(|facts| {
            facts
                .frame()
                .overlay_order()
                .bottom_to_top()
                .to_vec()
                .into_boxed_slice()
        })
    }

    #[cfg(test)]
    pub(crate) fn current_node_receipts(
        &self,
    ) -> Box<[worth_ui_host_contract::UiMountedNodeReceiptIdentity]> {
        self.current
            .as_ref()
            .map(|facts| {
                facts
                    .records()
                    .iter()
                    .filter_map(|record| record.node_receipt())
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            })
            .unwrap_or_default()
    }

    #[cfg(test)]
    #[allow(
        dead_code,
        reason = "Gate 1 retains appearance counter evidence for later mounting proofs"
    )]
    pub(crate) const fn counters(&self) -> counters::UiMountedAppearanceCounters {
        self.counters
    }

    #[cfg(test)]
    pub(crate) const fn last_delta(&self) -> Option<UiMountedAppearanceDeltaSummary> {
        self.last_delta
    }

    #[cfg(test)]
    pub(crate) fn last_delta_mechanics_changed(&self) -> Option<bool> {
        self.last_delta.map(|delta| delta.mechanics_changed())
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) use tests::mounted_sidecar_with_retained_facts_for_test;
#[cfg(test)]
pub(crate) use tests::MountedAppearanceReconstructionTestFixture;
