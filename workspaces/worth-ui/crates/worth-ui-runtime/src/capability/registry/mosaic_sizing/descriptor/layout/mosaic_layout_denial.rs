/// Why a Mosaic layout declaration cannot be constructed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MosaicLayoutDenial {
    /// A fixed track declared no extent.
    EmptyTrack,
    /// A flexible track declared no share of the remaining space.
    ZeroWeight,
    /// A flexible track's minimum exceeds its maximum, or its maximum is zero.
    InconsistentBounds,
    /// An axis declared no tracks.
    NoTracks,
    /// An axis declared more tracks than a cell can address.
    TooManyTracks,
    /// A cell spans no tracks.
    EmptyCell,
    /// A cell reaches past the declared tracks.
    CellOutsideTracks,
    /// A component was placed in more than one cell.
    DuplicateMember,
    /// A viewport-width interval admits no width.
    EmptyViewportInterval,
    /// Two responsive variants admit the same viewport width.
    OverlappingViewportIntervals,
    /// A responsive variant places a different member set than its fallback.
    VariantMembershipMismatch,
}
