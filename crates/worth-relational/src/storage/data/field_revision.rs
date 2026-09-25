use crate::identity::data::VersionId;

/// A source-owner revision of one declared field at an immutable read basis.
/// Absence is revised just like presence, so remove/recreate cannot compare
/// equal merely because both observations ended absent.
#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize,
)]
pub enum RelationalFieldPresence {
    Present,
    Absent,
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize,
)]
pub struct RelationalFieldRevision {
    version: VersionId,
    presence: RelationalFieldPresence,
}

impl RelationalFieldRevision {
    pub const fn new(version: VersionId, presence: RelationalFieldPresence) -> Self {
        Self { version, presence }
    }

    pub const fn version(self) -> VersionId {
        self.version
    }

    pub const fn presence(self) -> RelationalFieldPresence {
        self.presence
    }
}
