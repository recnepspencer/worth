//! Exact first/final control equality and slices retained for the Serving seal.

use super::{Controls, SelectedControlMediaFingerprint};

impl Controls {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin::pending_wal_release) fn same_bytes(
        &self,
        other: &Self,
    ) -> bool {
        self.routes == other.routes
            && self.source_routes == other.source_routes
            && self.checkpoint_routes == other.checkpoint_routes
            && match (&self.selected_base, &other.selected_base) {
                (Some(before), Some(after)) => before.matches_reread(after),
                (None, None) => true,
                _ => false,
            }
            && match (
                &self.selected_head_v2_controls,
                &other.selected_head_v2_controls,
            ) {
                (Some(before), Some(after)) => before.same_bytes(after),
                (None, None) => true,
                _ => false,
            }
            && self.source_topology == other.source_topology
            && self.published_topology == other.published_topology
            && self.descriptor == other.descriptor
            && self.reservation == other.reservation
            && self.manifest == other.manifest
            && self.slices == other.slices
            && self.checkpoint_slices == other.checkpoint_slices
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin::pending_wal_release) fn into_fingerprint(
        self,
    ) -> Result<SelectedControlMediaFingerprint, super::Denial> {
        let mut fingerprint = self.routes.into_media_fingerprint();
        fingerprint.extend(self.source_routes.into_media_fingerprint())?;
        if let Some(routes) = self.checkpoint_routes {
            fingerprint.extend(routes.into_media_fingerprint())?;
        }
        if let Some(base) = self.selected_base {
            fingerprint.extend(base.fingerprint())?;
        }
        if let Some(base) = self.selected_head_v2_controls {
            fingerprint.extend(base.into_fingerprint())?;
        }
        fingerprint.extend(SelectedControlMediaFingerprint::observed(self.slices))?;
        fingerprint.extend(SelectedControlMediaFingerprint::observed(
            self.checkpoint_slices,
        ))?;
        Ok(fingerprint)
    }
}
