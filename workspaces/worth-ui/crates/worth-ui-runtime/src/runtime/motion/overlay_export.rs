#![allow(
    dead_code,
    reason = "Gate 1 retains motion overlay exports for later composition publication"
)]

use crate::runtime::motion::{UiCommittedMotionTrack, UiMotionTargetIdentity};
pub(crate) type UiMotionOverlayRows = crate::runtime::persistent_index::UiPersistentOrdMap<
    UiMotionTargetIdentity,
    UiMotionOverlayOwnerRow,
>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMotionOverlayOwnerRow {
    target: UiMotionTargetIdentity,
    revision: u64,
}

impl UiMotionOverlayOwnerRow {
    pub(super) fn from_request(request: super::UiMotionTransitionRequest) -> Self {
        Self {
            target: request.successor().target(),
            revision: request.successor().owner_revision(),
        }
    }

    pub(crate) const fn target(self) -> UiMotionTargetIdentity {
        self.target
    }

    pub(crate) const fn revision(self) -> u64 {
        self.revision
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMotionOverlayOwnerExport {
    owner_revision: u64,
    rows: Box<[UiMotionOverlayOwnerRow]>,
}

impl UiMotionOverlayOwnerExport {
    pub(crate) fn from_rows(
        owner_revision: u64,
        rows: impl IntoIterator<Item = UiMotionOverlayOwnerRow>,
    ) -> Self {
        Self {
            owner_revision,
            rows: rows.into_iter().collect::<Vec<_>>().into_boxed_slice(),
        }
    }
    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }

    pub(crate) fn rows(&self) -> &[UiMotionOverlayOwnerRow] {
        &self.rows
    }
}

impl super::UiMotionRuntimeState {
    pub(crate) fn overlay_rows_for_surface(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> UiMotionOverlayRows {
        self.overlay_rows.get(&surface).cloned().unwrap_or_default()
    }

    pub(super) fn refresh_overlay_row(
        &mut self,
        request: super::UiMotionTransitionRequest,
        kind: super::UiMotionProducedFactKind,
    ) {
        let target = request.successor().target();
        let surface = target.semantic_surface();
        if matches!(kind, super::UiMotionProducedFactKind::Terminal(_)) {
            if let Some(rows) = self.overlay_rows.get_mut(&surface) {
                rows.remove(&target);
                if rows.is_empty() {
                    self.overlay_rows.remove(&surface);
                }
            }
        } else {
            let rows = self.overlay_rows.entry(surface).or_default();
            let row = UiMotionOverlayOwnerRow::from_request(request);
            if rows.get(&target) != Some(&row) {
                rows.insert(target, row);
            }
        }
    }

    /// Publish only the committed Motion target facts needed by Gate 1. The
    /// overlay adapter remains the sole owner of overlay snapshot sealing.
    pub(crate) fn overlay_owner_export(&self) -> UiMotionOverlayOwnerExport {
        let mut rows = self
            .tracks
            .values()
            .map(|track: &UiCommittedMotionTrack| UiMotionOverlayOwnerRow {
                target: track.target(),
                revision: track.successor_revision(),
            })
            .collect::<Vec<_>>();
        rows.sort_unstable_by_key(|row| row.target());
        UiMotionOverlayOwnerExport {
            owner_revision: self.publication_count(),
            rows: rows.into_boxed_slice(),
        }
    }
}
