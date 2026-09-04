use worth_ui_host_contract::{
    UiMountedFrameCanonicalCore, UiMountedFrameIntegrity, UiMountedFrameManifest,
    UiMountedProjectionView, UiMountedSurfaceBindingRequirement, UiSemanticSurfaceIdentity,
};

use crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity;

#[path = "assembly/prepared_frame.rs"]
mod prepared_frame;

pub(crate) use prepared_frame::binding_requirement;

#[derive(Clone, Debug)]
pub struct UiMountedFrameRequest {
    surfaces: UiMountedSurfaceSelection,
    virtualized_range: Option<crate::runtime::WorthUiVisibleRange>,
    visual_overlay_revision: u64,
    visual_overlay: Option<super::UiMountedVisualOverlayProjectionInput>,
    portal_overlay_revision: u64,
    portal_overlays: std::rc::Rc<[super::UiMountedPortalOverlayProjectionInput]>,
    reuse_identity: UiMountedFrameRequestIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum UiMountedSurfaceSelection {
    AllBound,
    Exact(std::rc::Rc<[UiSemanticSurfaceIdentity]>),
}

#[derive(Clone)]
pub(crate) struct UiMountedFrameRequestIdentity(std::rc::Rc<()>);

#[derive(Debug, PartialEq)]
pub enum UiMountedFramePreparationDenial {
    DuplicateSurfaceRequirement,
    MissingSurfaceBinding(UiSemanticSurfaceIdentity),
    SurfaceRebindRequired(UiSemanticSurfaceIdentity),
    LaneWorkUnavailable(worth_ui_host_contract::UiMountedLaneParticipation),
    Lane(crate::facade::WorthUiMountedLaneProjectionDenial),
    Projection(super::UiMountedProjectionDenial),
    TraceSourceGenerationMismatch,
    IncompleteManifest,
    IntegrityMismatch,
    AppearanceStateCapacityExceeded(super::projection::UiAppearanceStateCapacityExceeded),
    AppearanceStateIdentityMismatch,
}

#[derive(Clone)]
pub struct UiMountedSurfaceReceipt {
    requirement: UiMountedSurfaceBindingRequirement,
    projection_frame: std::rc::Rc<super::UiMountedProjectionFrame>,
    projection: std::cell::OnceCell<UiMountedProjectionView>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedFrameReceipt {
    canonical_core: UiMountedFrameCanonicalCore,
    integrity: UiMountedFrameIntegrity,
    surface_count: usize,
    cost: super::UiMountCostReport,
}

pub struct UiPreparedMountedFrame {
    candidate: super::UiProjectedMountedFrameCandidate,
    generation: WorthUiPreparedApplicationGenerationIdentity,
    manifest: UiMountedFrameManifest,
    canonical_core: UiMountedFrameCanonicalCore,
    integrity: UiMountedFrameIntegrity,
    surfaces: Box<[UiMountedSurfaceReceipt]>,
    identity_trace_basis: super::UiMountedIdentityTraceBasis,
    cost: super::UiMountCostReport,
    reuse_contract: super::UiMountedFrameReuseContract,
}

pub(crate) struct UiPreparedMountedFrameAdmission {
    pub candidate: super::UiProjectedMountedFrameCandidate,
    pub generation: WorthUiPreparedApplicationGenerationIdentity,
    pub manifest: UiMountedFrameManifest,
    pub graph_world: u64,
    pub allocation_truth_revision: u64,
    pub trace_source:
        crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource,
    pub reuse_contract: super::UiMountedFrameReuseContract,
}

impl UiMountedFrameRequest {
    pub fn all_bound_surfaces() -> Self {
        Self {
            surfaces: UiMountedSurfaceSelection::AllBound,
            virtualized_range: None,
            visual_overlay_revision: 0,
            visual_overlay: None,
            portal_overlay_revision: 0,
            portal_overlays: std::rc::Rc::from([]),
            reuse_identity: UiMountedFrameRequestIdentity(std::rc::Rc::new(())),
        }
    }

    pub fn exact_surfaces(surfaces: Vec<UiSemanticSurfaceIdentity>) -> Self {
        Self {
            surfaces: UiMountedSurfaceSelection::Exact(surfaces.into()),
            virtualized_range: None,
            visual_overlay_revision: 0,
            visual_overlay: None,
            portal_overlay_revision: 0,
            portal_overlays: std::rc::Rc::from([]),
            reuse_identity: UiMountedFrameRequestIdentity(std::rc::Rc::new(())),
        }
    }

    pub fn with_virtualized_range(mut self, range: crate::runtime::WorthUiVisibleRange) -> Self {
        self.virtualized_range = Some(range);
        self.reuse_identity = UiMountedFrameRequestIdentity(std::rc::Rc::new(()));
        self
    }

    pub fn virtualized_range(&self) -> Option<crate::runtime::WorthUiVisibleRange> {
        self.virtualized_range
    }

    pub(crate) fn with_visual_overlay(
        mut self,
        revision: u64,
        visual_overlay: Option<super::UiMountedVisualOverlayProjectionInput>,
    ) -> Self {
        self.visual_overlay_revision = revision;
        self.visual_overlay = visual_overlay;
        self.reuse_identity = UiMountedFrameRequestIdentity(std::rc::Rc::new(()));
        self
    }

    pub(crate) const fn visual_overlay_revision(&self) -> u64 {
        self.visual_overlay_revision
    }

    pub(crate) const fn visual_overlay(
        &self,
    ) -> Option<super::UiMountedVisualOverlayProjectionInput> {
        self.visual_overlay
    }

    pub(crate) fn with_portal_overlays(
        mut self,
        revision: u64,
        portal_overlays: Vec<super::UiMountedPortalOverlayProjectionInput>,
    ) -> Self {
        self.portal_overlay_revision = revision;
        self.portal_overlays = portal_overlays.into();
        self.reuse_identity = UiMountedFrameRequestIdentity(std::rc::Rc::new(()));
        self
    }

    pub(crate) fn portal_overlays(
        &self,
    ) -> std::rc::Rc<[super::UiMountedPortalOverlayProjectionInput]> {
        std::rc::Rc::clone(&self.portal_overlays)
    }

    pub(crate) fn reuse_identity(&self) -> UiMountedFrameRequestIdentity {
        self.reuse_identity.clone()
    }

    pub(crate) fn resolve_requirements(
        &self,
        bindings: &[super::UiSurfaceBindingIdentityView],
    ) -> Result<Vec<super::UiSurfaceBindingIdentityView>, UiMountedFramePreparationDenial> {
        match &self.surfaces {
            UiMountedSurfaceSelection::AllBound => Ok(bindings.to_vec()),
            UiMountedSurfaceSelection::Exact(surfaces) => {
                let mut ordered = surfaces.to_vec();
                ordered.sort();
                if ordered.windows(2).any(|pair| pair[0] == pair[1]) {
                    return Err(UiMountedFramePreparationDenial::DuplicateSurfaceRequirement);
                }
                ordered
                    .into_iter()
                    .map(|surface| {
                        bindings
                            .iter()
                            .find(|binding| binding.semantic_surface_identity() == surface)
                            .copied()
                            .ok_or(UiMountedFramePreparationDenial::MissingSurfaceBinding(
                                surface,
                            ))
                    })
                    .collect()
            }
        }
    }
}

impl PartialEq for UiMountedFrameRequest {
    fn eq(&self, other: &Self) -> bool {
        self.surfaces == other.surfaces
            && self.virtualized_range == other.virtualized_range
            && self.visual_overlay_revision == other.visual_overlay_revision
            && self.visual_overlay == other.visual_overlay
            && self.portal_overlay_revision == other.portal_overlay_revision
            && self.portal_overlays == other.portal_overlays
    }
}

impl Eq for UiMountedFrameRequest {}

impl PartialEq for UiMountedFrameRequestIdentity {
    fn eq(&self, other: &Self) -> bool {
        std::rc::Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for UiMountedFrameRequestIdentity {}

impl std::fmt::Debug for UiMountedFrameRequestIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("UiMountedFrameRequestIdentity")
    }
}

impl UiMountedSurfaceReceipt {
    pub fn requirement(&self) -> UiMountedSurfaceBindingRequirement {
        self.requirement
    }

    pub fn projection(&self) -> &UiMountedProjectionView {
        self.projection.get_or_init(|| {
            self.projection_frame
                .view_for(self.requirement.binding())
                .expect("admitted surface binding remains present in mounted authority")
        })
    }

    pub(crate) fn projection_owner(&self) -> std::rc::Rc<super::UiMountedProjectionFrame> {
        std::rc::Rc::clone(&self.projection_frame)
    }

    pub(crate) fn presentation_effects(
        &self,
    ) -> Box<[worth_ui_host_contract::UiMountedEffectFamily]> {
        self.projection_frame.presentation_effects(
            self.requirement.presentation_mode(),
            self.requirement.binding(),
        )
    }
}

impl std::fmt::Debug for UiMountedSurfaceReceipt {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UiMountedSurfaceReceipt")
            .field("requirement", &self.requirement)
            .field("projection_materialized", &self.projection.get().is_some())
            .finish()
    }
}

impl PartialEq for UiMountedSurfaceReceipt {
    fn eq(&self, other: &Self) -> bool {
        self.requirement == other.requirement && self.projection() == other.projection()
    }
}

impl UiMountedFrameReceipt {
    pub fn canonical_core(&self) -> UiMountedFrameCanonicalCore {
        self.canonical_core
    }

    pub fn integrity(&self) -> UiMountedFrameIntegrity {
        self.integrity
    }

    pub fn surface_count(&self) -> usize {
        self.surface_count
    }

    pub fn cost_report(&self) -> super::UiMountCostReport {
        self.cost
    }

    pub fn delta(&self) -> super::UiMountedFrameDelta {
        super::UiMountedFrameDelta::from_cost(self.cost)
    }
}

#[cfg(test)]
#[path = "assembly_visual_overlay_tests.rs"]
mod visual_overlay_tests;
