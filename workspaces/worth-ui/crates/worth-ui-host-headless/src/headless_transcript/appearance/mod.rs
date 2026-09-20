mod backdrop;
mod outline;
mod overlay_order;
mod pointer_affordance;
pub(crate) mod reference_raster;
mod scroll_chrome;
mod spatial_reference;
mod surface;
mod text_foreground;
pub(crate) mod work;

pub use work::UiHeadlessAppearanceMechanicChange;

use worth_ui_host_contract::{
    UiAppearanceDamageRegion, UiMountedAppearanceFrame, UiMountedAppearanceMechanic,
    UiMountedAppearancePredecessorManifest, UiMountedOverlayOrderMechanic,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiHeadlessAppearanceMechanic {
    Surface(worth_ui_host_contract::UiMountedSurfaceAppearanceMechanic),
    PortalSurface(worth_ui_host_contract::UiMountedPortalSurfaceAppearanceMechanic),
    Outline(worth_ui_host_contract::UiMountedOutlineAppearanceMechanic),
    TextForeground(worth_ui_host_contract::UiMountedTextForegroundAppearanceMechanic),
    Pointer(worth_ui_host_contract::UiMountedPointerAffordanceMechanic),
    Backdrop(worth_ui_host_contract::UiMountedBackdropMechanic),
    ScrollChrome(worth_ui_host_contract::UiMountedScrollChromeMechanic),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiHeadlessAppearanceFrameTranscript {
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    mechanics: Box<[UiHeadlessAppearanceMechanic]>,
    overlay_order: UiMountedOverlayOrderMechanic,
    reference_source_over: worth_ui_host_contract::UiMountedAppearanceColor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiHeadlessAppearanceWorkTranscript {
    posture: worth_ui_host_contract::UiMountedAppearanceWorkPosture,
    predecessor: Option<worth_ui_host_contract::UiMountedFrameIdentity>,
    predecessor_manifest: Option<UiMountedAppearancePredecessorManifest>,
    successor: UiHeadlessAppearanceFrameTranscript,
    changes: Box<[UiHeadlessAppearanceMechanicChange]>,
    damage: Box<[UiAppearanceDamageRegion]>,
    text_damage_requirements: Box<[worth_ui_host_contract::UiAppearanceTextDamageRequirement]>,
    order_changed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiHeadlessAppearanceFragmentTranscript {
    identity: worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity,
    work: UiHeadlessAppearanceWorkTranscript,
    text_candidates: Box<[worth_ui_host_contract::UiMountedSemanticTextMechanic]>,
    surface_binding: worth_ui_host_contract::UiMountedSurfaceBindingRequirement,
    presentation_affinity: worth_ui_host_contract::UiMountedPresentationAffinity,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiHeadlessAppearancePresentationTranscript {
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    requirement: worth_ui_host_contract::UiMountedSurfaceBindingRequirement,
    fragments: Box<[UiHeadlessAppearanceFragmentTranscript]>,
    sample_overrides: Box<[worth_ui_host_contract::UiMountedPresentationSampleChange]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiHeadlessAppearanceProjectionTranscript {
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    fragments: Box<[UiHeadlessAppearanceFragmentTranscript]>,
}

impl UiHeadlessAppearanceMechanic {
    fn from_mounted(mechanic: &UiMountedAppearanceMechanic) -> Self {
        match mechanic {
            UiMountedAppearanceMechanic::Surface(mechanic) => surface::translate(mechanic),
            UiMountedAppearanceMechanic::PortalSurface(mechanic) => {
                surface::translate_portal(mechanic)
            }
            UiMountedAppearanceMechanic::Outline(mechanic) => outline::translate(mechanic),
            UiMountedAppearanceMechanic::TextForeground(mechanic) => {
                text_foreground::translate(mechanic)
            }
            UiMountedAppearanceMechanic::Pointer(mechanic) => {
                pointer_affordance::translate(mechanic)
            }
            UiMountedAppearanceMechanic::Backdrop(mechanic) => backdrop::translate(mechanic),
            UiMountedAppearanceMechanic::ScrollChrome(mechanic) => {
                scroll_chrome::translate(mechanic)
            }
        }
    }

    pub(crate) fn matches_mounted(&self, source: &UiMountedAppearanceMechanic) -> bool {
        match (self, source) {
            (Self::Surface(left), UiMountedAppearanceMechanic::Surface(right)) => left == right,
            (Self::PortalSurface(left), UiMountedAppearanceMechanic::PortalSurface(right)) => {
                left == right
            }
            (Self::Outline(left), UiMountedAppearanceMechanic::Outline(right)) => left == right,
            (Self::TextForeground(left), UiMountedAppearanceMechanic::TextForeground(right)) => {
                left == right
            }
            (Self::Pointer(left), UiMountedAppearanceMechanic::Pointer(right)) => left == right,
            (Self::Backdrop(left), UiMountedAppearanceMechanic::Backdrop(right)) => left == right,
            (Self::ScrollChrome(left), UiMountedAppearanceMechanic::ScrollChrome(right)) => {
                left == right
            }
            _ => false,
        }
    }
}

impl UiHeadlessAppearanceFrameTranscript {
    pub(crate) fn from_mounted(frame: &UiMountedAppearanceFrame) -> Option<Self> {
        Some(Self {
            frame: frame.frame(),
            semantic_surface: frame.semantic_surface(),
            mechanics: frame
                .mechanics()
                .iter()
                .map(UiHeadlessAppearanceMechanic::from_mounted)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            overlay_order: overlay_order::translate(frame.overlay_order()),
            reference_source_over: reference_raster::compose(frame)?,
        })
    }

    pub const fn frame(&self) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.frame
    }

    pub const fn semantic_surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.semantic_surface
    }

    pub fn mechanics(&self) -> &[UiHeadlessAppearanceMechanic] {
        &self.mechanics
    }

    pub const fn overlay_order(&self) -> &UiMountedOverlayOrderMechanic {
        &self.overlay_order
    }

    pub const fn reference_source_over(&self) -> worth_ui_host_contract::UiMountedAppearanceColor {
        self.reference_source_over
    }
}

impl UiHeadlessAppearanceWorkTranscript {
    pub(crate) fn from_mounted(
        work: &worth_ui_host_contract::UiMountedAppearanceWork,
    ) -> Option<Self> {
        Some(Self {
            posture: work.posture(),
            predecessor: work.predecessor(),
            predecessor_manifest: work.predecessor_manifest().cloned(),
            successor: UiHeadlessAppearanceFrameTranscript::from_mounted(work.successor())?,
            changes: work::translate_changes(work.changes()),
            damage: work.damage().to_vec().into_boxed_slice(),
            text_damage_requirements: work.text_damage_requirements().collect(),
            order_changed: work.order_changed(),
        })
    }

    pub const fn posture(&self) -> worth_ui_host_contract::UiMountedAppearanceWorkPosture {
        self.posture
    }

    pub const fn predecessor(&self) -> Option<worth_ui_host_contract::UiMountedFrameIdentity> {
        self.predecessor
    }

    pub fn predecessor_manifest(&self) -> Option<&UiMountedAppearancePredecessorManifest> {
        self.predecessor_manifest.as_ref()
    }

    pub fn successor(&self) -> &UiHeadlessAppearanceFrameTranscript {
        &self.successor
    }

    pub fn changes(&self) -> &[UiHeadlessAppearanceMechanicChange] {
        &self.changes
    }

    pub fn damage(&self) -> &[UiAppearanceDamageRegion] {
        &self.damage
    }

    pub fn text_damage_requirements(
        &self,
    ) -> &[worth_ui_host_contract::UiAppearanceTextDamageRequirement] {
        &self.text_damage_requirements
    }

    pub const fn order_changed(&self) -> bool {
        self.order_changed
    }
}

impl UiHeadlessAppearanceFragmentTranscript {
    pub(crate) fn from_source(
        source: &worth_ui_host_contract::UiUnpublishedAppearanceFragment,
        work: UiHeadlessAppearanceWorkTranscript,
    ) -> Self {
        Self {
            identity: source.identity(),
            work,
            text_candidates: source.text_candidates().to_vec().into_boxed_slice(),
            surface_binding: source.surface_binding(),
            presentation_affinity: source.presentation_affinity(),
        }
    }

    pub const fn identity(
        &self,
    ) -> worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity {
        self.identity
    }

    pub const fn work(&self) -> &UiHeadlessAppearanceWorkTranscript {
        &self.work
    }

    pub fn text_candidates(&self) -> &[worth_ui_host_contract::UiMountedSemanticTextMechanic] {
        &self.text_candidates
    }

    pub const fn surface_binding(
        &self,
    ) -> worth_ui_host_contract::UiMountedSurfaceBindingRequirement {
        self.surface_binding
    }

    pub const fn presentation_affinity(
        &self,
    ) -> worth_ui_host_contract::UiMountedPresentationAffinity {
        self.presentation_affinity
    }
}

impl UiHeadlessAppearancePresentationTranscript {
    pub(crate) fn from_source(
        source: &worth_ui_host_contract::UiMountedAppearancePresentationWork,
        fragments: Vec<UiHeadlessAppearanceFragmentTranscript>,
    ) -> Self {
        Self {
            frame: source.frame(),
            presentation: source.presentation(),
            requirement: source.requirement(),
            fragments: fragments.into_boxed_slice(),
            sample_overrides: source.sample_overrides().into(),
        }
    }

    pub const fn frame(&self) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.frame
    }
    pub const fn presentation(
        &self,
    ) -> worth_ui_host_contract::UiMountedPresentationAttemptIdentity {
        self.presentation
    }
    pub const fn requirement(&self) -> worth_ui_host_contract::UiMountedSurfaceBindingRequirement {
        self.requirement
    }
    pub fn fragments(&self) -> &[UiHeadlessAppearanceFragmentTranscript] {
        &self.fragments
    }
    pub fn sample_overrides(&self) -> &[worth_ui_host_contract::UiMountedPresentationSampleChange] {
        &self.sample_overrides
    }

    pub(crate) fn mechanic_count(
        &self,
    ) -> Result<u32, worth_ui_host_contract::UiHostSurfacePresentationDenial> {
        let count = self
            .fragments
            .iter()
            .map(|fragment| fragment.work.successor.mechanics.len())
            .try_fold(0usize, usize::checked_add)
            .ok_or(worth_ui_host_contract::UiHostSurfacePresentationDenial::CapacityExceeded)?;
        u32::try_from(count)
            .map_err(|_| worth_ui_host_contract::UiHostSurfacePresentationDenial::CapacityExceeded)
    }

    pub(crate) fn row_count(
        &self,
    ) -> Result<usize, worth_ui_host_contract::UiHostSurfacePresentationDenial> {
        let initial = self
            .fragments
            .len()
            .checked_add(self.sample_overrides.len())
            .ok_or(worth_ui_host_contract::UiHostSurfacePresentationDenial::CapacityExceeded)?;
        self.fragments.iter().try_fold(initial, |total, fragment| {
            total
                .checked_add(fragment.work.successor.mechanics.len())
                .and_then(|value| value.checked_add(fragment.work.changes.len()))
                .and_then(|value| value.checked_add(fragment.work.damage.len()))
                .and_then(|value| value.checked_add(fragment.text_candidates.len()))
                .ok_or(worth_ui_host_contract::UiHostSurfacePresentationDenial::CapacityExceeded)
        })
    }
}

impl UiHeadlessAppearanceProjectionTranscript {
    #[cfg(feature = "certification-support")]
    pub(crate) fn from_source(
        source: &worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection,
        fragments: Vec<UiHeadlessAppearanceFragmentTranscript>,
    ) -> Self {
        Self {
            frame: source.frame(),
            presentation: source.presentation(),
            fragments: fragments.into_boxed_slice(),
        }
    }

    pub const fn frame(&self) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.frame
    }
    pub const fn presentation(
        &self,
    ) -> worth_ui_host_contract::UiMountedPresentationAttemptIdentity {
        self.presentation
    }
    pub fn fragments(&self) -> &[UiHeadlessAppearanceFragmentTranscript] {
        &self.fragments
    }
}
