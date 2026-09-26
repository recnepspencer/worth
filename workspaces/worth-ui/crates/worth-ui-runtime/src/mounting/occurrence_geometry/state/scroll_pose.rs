//! A Scroll pose prepared over one surface's mounted geometry, not yet
//! applied: the rows and regions it moves, and how far it moves each
//! occurrence's hit row.
use super::*;
use crate::mounting::UiHitScrollMove;

pub(crate) struct UiPreparedMountedScrollPose {
    pub(super) surface: UiSemanticSurfaceIdentity,
    pub(super) poses: Vec<(
        UiMountedInstanceIdentity,
        crate::runtime::scroll::UiScrollOffset,
    )>,
    pub(super) rows: Vec<(UiMountedInstanceIdentity, UiMountedOccurrenceGeometryRow)>,
    pub(super) regions: Vec<(
        worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
        usize,
        UiMountedCanonicalBox,
    )>,
    pub(super) translations: Vec<(UiMountedInstanceIdentity, UiHitScrollMove)>,
    pub(super) changes_coverage: bool,
    pub(super) work: crate::mounting::UiHitTestSpatialWork,
}

impl UiPreparedMountedScrollPose {
    pub(crate) const fn work(&self) -> crate::mounting::UiHitTestSpatialWork {
        self.work
    }

    pub(crate) fn changed_instances(&self) -> Box<[UiMountedInstanceIdentity]> {
        self.rows.iter().map(|row| row.0).collect()
    }

    /// Whether this pose moves any occurrence's ancestor clips across the
    /// line between sharing coverage and sharing none. Content that crosses
    /// it is paint the host was never given, or paint it must retire, and a
    /// displayed sample can do neither.
    pub(crate) const fn changes_coverage(&self) -> bool {
        self.changes_coverage
    }

    pub(crate) const fn surface(&self) -> UiSemanticSurfaceIdentity {
        self.surface
    }

    /// How far this pose moves each occurrence's hit row, in presented
    /// points, and the ancestor clips it leaves over each.
    ///
    /// Handing it out is what lets a pointer resolve against the content a
    /// settle or a lead just put under it rather than the content that was
    /// there before. Rows move from where they committed, so where mounted
    /// geometry is staged past the frame the host shows, they move farther or
    /// nearer than the geometry does.
    pub(crate) fn translations(&self) -> &[(UiMountedInstanceIdentity, UiHitScrollMove)] {
        &self.translations
    }
}
