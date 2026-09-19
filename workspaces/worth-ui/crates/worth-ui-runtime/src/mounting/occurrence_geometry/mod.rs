mod region;
mod state;
pub use region::UiMountedMosaicRegionGeometry;
mod receipt;
pub use receipt::UiMountedLayoutCompletionReceipt;
mod region_paint;
mod surface_paint_posture;
pub(crate) use surface_paint_posture::{
    UiMountedCanonicalBorderOmission, UiMountedSurfacePaintPosture,
};
mod clip_binding;
pub(crate) use clip_binding::{UiMountedMosaicClipBinding, UiMountedScrollClipBinding};
mod basis;
pub use basis::UiMountedLayoutBasis;

pub(crate) use state::UiMountedOccurrenceGeometryState;

use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedInstanceIdentity, UiSemanticSurfaceIdentity,
};

/// Monotonic revision minted by the owner completing mounted layout.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UiMountedLayoutRevision(std::num::NonZeroU64);

/// Coordinate contract for one completed layout occurrence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiMountedOccurrencePlacement {
    /// A root occurrence in host-surface coordinates.
    Surface(UiMountedCanonicalBox),
    /// A child occurrence in its exact parent's local coordinates.
    ParentRelative {
        parent: UiMountedInstanceIdentity,
        bounds: UiMountedCanonicalBox,
    },
}

/// One completed layout result for an exact mounted occurrence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMountedOccurrenceGeometry {
    instance: UiMountedInstanceIdentity,
    placement: UiMountedOccurrencePlacement,
}

/// Atomic layout-owner publication for one currently bound runtime surface.
#[derive(Clone, Debug, PartialEq)]
pub struct UiMountedSurfaceGeometryBatch {
    basis: UiMountedLayoutBasis,
    surface: UiSemanticSurfaceIdentity,
    layout_revision: UiMountedLayoutRevision,
    viewport: UiMountedCanonicalBox,
    occurrences: Box<[UiMountedOccurrenceGeometry]>,
    regions: Box<[UiMountedMosaicRegionGeometry]>,
    mosaic_clips:
        std::collections::BTreeMap<UiMountedInstanceIdentity, Box<[UiMountedMosaicClipBinding]>>,
    scroll_clips: std::collections::BTreeMap<
        UiMountedInstanceIdentity,
        Result<Box<[UiMountedScrollClipBinding]>, crate::graph::UiGraphNodeIdentity>,
    >,
    seam_paint: Option<UiMountedMosaicSeamPaintInput>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiMountedMosaicSeamPaintInput {
    contract: crate::capability::MosaicSeamPaintContract,
    region_kinds: std::collections::BTreeMap<
        (
            UiMountedInstanceIdentity,
            worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
        ),
        crate::capability::MosaicRegionKindId,
    >,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedOccurrenceGeometryDenial {
    PresentationInFlight,
    StateRevisionExhausted,
    MissingSurfaceBinding,
    EmptyBatch,
    StaleLayoutRevision,
    UnknownMountedInstance,
    ForeignSurface,
    DuplicateMountedInstance,
    MissingMountedInstance,
    UnknownParentOccurrence,
    ParentSurfaceMismatch,
    SurfaceViewportCoordinateSpaceMismatch,
    SurfaceCoordinateSpaceMismatch,
    ParentCoordinateSpaceMismatch,
    ParentCycle,
    MissingOccurrenceGeometry,
    StaleOccurrenceGeometry,
    ForeignRegionDeclaration,
    UnknownExecutedRegion,
    MissingMosaicRegionGeometry,
    AmbiguousMosaicRegionGeometry,
    UndeclaredMosaicSharedEdge,
    MosaicBindingUnavailable,
    DuplicateRegionOccurrence,
    StaleLayoutBasis,
}

impl UiMountedOccurrenceGeometry {
    pub fn surface(instance: UiMountedInstanceIdentity, bounds: UiMountedCanonicalBox) -> Self {
        Self {
            instance,
            placement: UiMountedOccurrencePlacement::Surface(bounds),
        }
    }

    pub fn parent_relative(
        instance: UiMountedInstanceIdentity,
        parent: UiMountedInstanceIdentity,
        bounds: UiMountedCanonicalBox,
    ) -> Self {
        Self {
            instance,
            placement: UiMountedOccurrencePlacement::ParentRelative { parent, bounds },
        }
    }

    pub fn instance(self) -> UiMountedInstanceIdentity {
        self.instance
    }

    pub fn parent(self) -> Option<UiMountedInstanceIdentity> {
        match self.placement {
            UiMountedOccurrencePlacement::Surface(_) => None,
            UiMountedOccurrencePlacement::ParentRelative { parent, .. } => Some(parent),
        }
    }

    pub fn bounds(self) -> UiMountedCanonicalBox {
        match self.placement {
            UiMountedOccurrencePlacement::Surface(bounds)
            | UiMountedOccurrencePlacement::ParentRelative { bounds, .. } => bounds,
        }
    }

    pub fn placement(self) -> UiMountedOccurrencePlacement {
        self.placement
    }
}

impl UiMountedSurfaceGeometryBatch {
    pub fn new(
        basis: UiMountedLayoutBasis,
        layout_revision: UiMountedLayoutRevision,
        viewport: UiMountedCanonicalBox,
        occurrences: impl Into<Box<[UiMountedOccurrenceGeometry]>>,
    ) -> Self {
        Self {
            surface: basis.surface(),
            basis,
            layout_revision,
            viewport,
            occurrences: occurrences.into(),
            regions: Box::default(),
            mosaic_clips: std::collections::BTreeMap::new(),
            scroll_clips: std::collections::BTreeMap::new(),
            seam_paint: None,
        }
    }

    pub fn surface(&self) -> UiSemanticSurfaceIdentity {
        self.surface
    }

    pub fn layout_revision(&self) -> UiMountedLayoutRevision {
        self.layout_revision
    }

    pub fn viewport(&self) -> UiMountedCanonicalBox {
        self.viewport
    }

    pub fn occurrences(&self) -> &[UiMountedOccurrenceGeometry] {
        &self.occurrences
    }

    pub fn with_regions(
        mut self,
        regions: impl Into<Box<[UiMountedMosaicRegionGeometry]>>,
    ) -> Self {
        self.regions = regions.into();
        self
    }

    pub fn regions(&self) -> &[UiMountedMosaicRegionGeometry] {
        &self.regions
    }

    pub(crate) fn with_mosaic_clips(
        mut self,
        clips: std::collections::BTreeMap<
            UiMountedInstanceIdentity,
            Box<[UiMountedMosaicClipBinding]>,
        >,
    ) -> Self {
        self.mosaic_clips = clips;
        self
    }

    pub(crate) fn mosaic_clips(
        &self,
    ) -> &std::collections::BTreeMap<UiMountedInstanceIdentity, Box<[UiMountedMosaicClipBinding]>>
    {
        &self.mosaic_clips
    }

    pub(crate) fn with_scroll_clips(
        mut self,
        clips: std::collections::BTreeMap<
            UiMountedInstanceIdentity,
            Result<Box<[UiMountedScrollClipBinding]>, crate::graph::UiGraphNodeIdentity>,
        >,
    ) -> Self {
        self.scroll_clips = clips;
        self
    }

    pub(crate) fn scroll_clips(
        &self,
    ) -> &std::collections::BTreeMap<
        UiMountedInstanceIdentity,
        Result<Box<[UiMountedScrollClipBinding]>, crate::graph::UiGraphNodeIdentity>,
    > {
        &self.scroll_clips
    }

    pub(crate) fn with_mosaic_seam_paint(
        mut self,
        contract: crate::capability::MosaicSeamPaintContract,
        region_kinds: std::collections::BTreeMap<
            (
                UiMountedInstanceIdentity,
                worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
            ),
            crate::capability::MosaicRegionKindId,
        >,
    ) -> Self {
        self.seam_paint = Some(UiMountedMosaicSeamPaintInput {
            contract,
            region_kinds,
        });
        self
    }

    pub(crate) fn seam_paint(&self) -> Option<&UiMountedMosaicSeamPaintInput> {
        self.seam_paint.as_ref()
    }

    pub(crate) fn basis(&self) -> &UiMountedLayoutBasis {
        &self.basis
    }
}

impl UiMountedMosaicSeamPaintInput {
    pub(super) fn contract(&self) -> &crate::capability::MosaicSeamPaintContract {
        &self.contract
    }

    pub(super) fn region_kind(
        &self,
        owner: UiMountedInstanceIdentity,
        declaration: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    ) -> Option<&crate::capability::MosaicRegionKindId> {
        self.region_kinds.get(&(owner, declaration))
    }
}

impl UiMountedLayoutRevision {
    pub const fn new(revision: u64) -> Option<Self> {
        match std::num::NonZeroU64::new(revision) {
            Some(revision) => Some(Self(revision)),
            None => None,
        }
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}
