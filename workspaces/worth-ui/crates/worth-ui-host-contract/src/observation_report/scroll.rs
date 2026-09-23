#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostScrollDeltaSource {
    PointerWheel,
    Touch,
    Pen,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostScrollDeltaPhase {
    Started,
    Updated,
    Ended,
    Cancelled,
}

/// Lines one wheel notch scrolls when the platform reports no usable count.
pub const UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH: u16 = 3;

/// Why a line precision carries the line count it carries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostScrollLineCountBasis {
    /// The platform reported the count and the host relayed it unchanged.
    PlatformReported,
    /// The platform exposed no count, so the host used the default.
    DefaultedAfterMissing,
    /// The platform reported a count the host cannot honour, so it used the
    /// default. A count of zero and a count no line-based reader can represent
    /// both land here.
    DefaultedAfterInvalid,
}

/// The unit the scroll delta subpixels are denominated in.
///
/// `Line` deltas carry lines at
/// [`super::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT`] subpixels per line:
/// notches turned, multiplied by `platform_lines_per_notch`. The host states
/// the count it applied so the reader can recover the notch count exactly; it
/// does not know how tall a line of content is. `Page` deltas carry pages at
/// the same subpixel scale, for platforms whose wheel setting asks for a page
/// per notch rather than a line count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostScrollDeltaPrecision {
    Line {
        platform_lines_per_notch: u16,
        basis: UiHostScrollLineCountBasis,
    },
    Page,
    Pixel,
}

impl UiHostScrollDeltaPrecision {
    pub const fn lines_per_notch(self) -> Option<u16> {
        match self {
            Self::Line {
                platform_lines_per_notch,
                ..
            } => Some(platform_lines_per_notch),
            Self::Page | Self::Pixel => None,
        }
    }

    pub const fn line_count_basis(self) -> Option<UiHostScrollLineCountBasis> {
        match self {
            Self::Line { basis, .. } => Some(basis),
            Self::Page | Self::Pixel => None,
        }
    }

    pub(super) const fn encoded_len(self) -> usize {
        match self {
            Self::Line { .. } => 4,
            Self::Page | Self::Pixel => 1,
        }
    }

    pub(super) const fn digest_basis(self) -> u64 {
        match self {
            Self::Pixel => 0,
            Self::Page => 1,
            Self::Line {
                platform_lines_per_notch,
                basis,
            } => 2 | (platform_lines_per_notch as u64) << 8 | (basis as u64) << 24,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostScrollDeltaTargetAffinity {
    ExactCoordinate {
        presentation: super::UiHostObservationPresentationBasis,
        position: super::UiHostSurfacePosition,
    },
    ExactMountedTarget {
        presentation: super::UiHostObservationPresentationBasis,
        mounted: super::UiHostObservationMountedBasis,
    },
    PresentedSurfaceFallback {
        presentation: super::UiHostObservationPresentationBasis,
    },
}

impl UiHostScrollDeltaTargetAffinity {
    pub const fn exact_coordinate(
        presentation: super::UiHostObservationPresentationBasis,
        position: super::UiHostSurfacePosition,
    ) -> Self {
        Self::ExactCoordinate {
            presentation,
            position,
        }
    }

    pub const fn exact_mounted_target(
        presentation: super::UiHostObservationPresentationBasis,
        mounted: super::UiHostObservationMountedBasis,
    ) -> Self {
        Self::ExactMountedTarget {
            presentation,
            mounted,
        }
    }

    pub const fn presented_surface_fallback(
        presentation: super::UiHostObservationPresentationBasis,
    ) -> Self {
        Self::PresentedSurfaceFallback { presentation }
    }

    pub const fn presentation(self) -> super::UiHostObservationPresentationBasis {
        match self {
            Self::ExactCoordinate { presentation, .. }
            | Self::ExactMountedTarget { presentation, .. }
            | Self::PresentedSurfaceFallback { presentation } => presentation,
        }
    }

    pub const fn position(self) -> Option<super::UiHostSurfacePosition> {
        match self {
            Self::ExactCoordinate { position, .. } => Some(position),
            Self::ExactMountedTarget { .. } | Self::PresentedSurfaceFallback { .. } => None,
        }
    }

    pub const fn mounted_target(self) -> Option<super::UiHostObservationMountedBasis> {
        match self {
            Self::ExactMountedTarget { mounted, .. } => Some(mounted),
            Self::ExactCoordinate { .. } | Self::PresentedSurfaceFallback { .. } => None,
        }
    }

    pub const fn is_surface_fallback(self) -> bool {
        matches!(self, Self::PresentedSurfaceFallback { .. })
    }

    pub(super) const fn encoded_len(self) -> usize {
        match self {
            Self::ExactCoordinate { .. } => 51,
            Self::ExactMountedTarget { .. } => 49,
            Self::PresentedSurfaceFallback { .. } => 33,
        }
    }
}
