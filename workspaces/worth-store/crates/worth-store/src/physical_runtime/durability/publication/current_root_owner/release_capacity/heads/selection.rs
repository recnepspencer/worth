//! Owner-authenticated release-head selection for one protected root.

use worth_store_physical_format::{
    ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1, RootPublicationCell,
};

use super::PhysicalCurrentRootOwner;
use crate::physical_runtime::PhysicalProtectedRootObservation;

/// Why the current-root owner cannot issue a selected release-head basis.
/// These diagnostics confer no custody or permission to reconstruct a ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedReleaseHeadDenial {
    ForeignRuntime,
    ProtectedRootChanged,
    /// Reopen requires C8 custody and independent Store rejoin before release.
    SelectedLedgerUnavailable,
    SelectedHeadRootMismatch,
}

/// Owner-sealed current release state for one exact protected selected root.
/// An absent edge may be treated as settled only when this basis contains a
/// current head; raw route absence or a decoded descriptor is insufficient.
#[derive(Clone, Copy)]
pub(in crate::physical_runtime) struct SelectedReleaseHeadBasis {
    root: RootPublicationCell,
    head: Option<ReleaseCustodyHeadEntryV1>,
}

impl SelectedReleaseHeadBasis {
    pub(in crate::physical_runtime) const fn root(self) -> RootPublicationCell {
        self.root
    }

    pub(in crate::physical_runtime) const fn head(self) -> Option<ReleaseCustodyHeadEntryV1> {
        self.head
    }

    pub(in crate::physical_runtime) const fn authorizes_settled_absence(self) -> bool {
        self.head.is_some()
    }
}

impl PhysicalCurrentRootOwner {
    /// The protected reader supplies the exact selected root. The owner must
    /// still hold that root and its independently authenticated effective head
    /// roster; no caller-provided entry can mint this basis.
    pub(in crate::physical_runtime) fn selected_release_head_for_root(
        &self,
        inspector: PhysicalProtectedRootObservation,
        key: ReleaseCustodyHeadKeyV1,
    ) -> Result<SelectedReleaseHeadBasis, SelectedReleaseHeadDenial> {
        let state = self.lock_publication_state();
        if inspector.runtime() != self.runtime_identity {
            return Err(SelectedReleaseHeadDenial::ForeignRuntime);
        }
        if inspector.root() != state.current_root.root_cell() {
            return Err(SelectedReleaseHeadDenial::ProtectedRootChanged);
        }
        let ledger = state
            .release_ledger
            .selected()
            .ok_or(SelectedReleaseHeadDenial::SelectedLedgerUnavailable)?;
        if ledger.effective_heads.root() != state.current_root.release_custody_head_root() {
            return Err(SelectedReleaseHeadDenial::SelectedHeadRootMismatch);
        }
        Ok(SelectedReleaseHeadBasis {
            root: inspector.root(),
            head: ledger.effective_heads.head(key),
        })
    }
}
