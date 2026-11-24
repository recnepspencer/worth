//! Which records a bounded kind scan reads from the state it walks.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KindScanVisibility {
    /// The live slots of the walked state, which is exactly one branch root.
    Live,
    /// The records the walked state's history holds at one version.
    AtVersion(crate::identity::data::VersionId),
}

impl KindScanVisibility {
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
