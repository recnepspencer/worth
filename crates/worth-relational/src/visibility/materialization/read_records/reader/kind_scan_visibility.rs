//! Which records a bounded kind scan reads from the state it walks.

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KindScanVisibility {
    /// The live slots of the walked state, which is exactly one branch root.
    Live,
    /// The records the walked state's history holds at one version.
    AtVersion(crate::identity::data::VersionId),
}

impl KindScanVisibility {
    /// A state's live slots are exactly the runtime's current version; any
    /// other version is read from the state's history.
    pub(crate) fn for_version(
        runtime: &crate::runtime::RelationalRuntime,
        version_id: crate::identity::data::VersionId,
    ) -> Self {
        Self::for_selected_version(VersionSource::current_version_id(runtime), version_id)
    }
    pub(crate) fn for_selected_version(
        current: crate::identity::data::VersionId,
        version_id: crate::identity::data::VersionId,
    ) -> Self {
        if version_id == current {
            Self::Live
        } else {
            Self::AtVersion(version_id)
        }
    }
}
