use crate::runtime::motion::{UiCommittedMotionTrack, UiMotionTargetIdentity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMotionOverlayOwnerRow {
    target: UiMotionTargetIdentity,
    revision: u64,
}

impl UiMotionOverlayOwnerRow {
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
    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }

    pub(crate) fn rows(&self) -> &[UiMotionOverlayOwnerRow] {
        &self.rows
    }
}

impl super::UiMotionRuntimeState {
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
