//! Transportable source coordinates are resolved against actual completed native evidence.
use super::WorthQueryRecordedCorrectionSource;
use worth_relational::facade::history::RelationalCommitReceipt;

/// Untrusted historical coordinates. Possession grants no disclosure or correction authority.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorthQueryCorrectionSourceCoordinates {
    world_owner: u64,
    world_commit: u64,
    relational: RelationalCommitReceipt,
}

impl WorthQueryCorrectionSourceCoordinates {
    pub fn from_untrusted_parts(
        world_owner: u64,
        world_commit: u64,
        relational: RelationalCommitReceipt,
    ) -> Self {
        Self {
            world_owner,
            world_commit,
            relational,
        }
    }
    pub fn relational(&self) -> &RelationalCommitReceipt {
        &self.relational
    }
    pub const fn world_owner(&self) -> u64 {
        self.world_owner
    }
    pub const fn world_commit(&self) -> u64 {
        self.world_commit
    }
}

impl WorthQueryRecordedCorrectionSource {
    pub fn coordinates(&self) -> WorthQueryCorrectionSourceCoordinates {
        WorthQueryCorrectionSourceCoordinates::from_untrusted_parts(
            self.world().owner_identity().get(),
            self.world().ordinal(),
            self.relational().clone(),
        )
    }
}
