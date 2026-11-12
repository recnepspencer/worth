//! Query's wire encoding of the World owner's descriptive coordinates.
use serde::{Deserialize, Serialize};
use worth_runtime_world::facade::ProductBranchReferenceDescription;

#[derive(Serialize, Deserialize)]
#[serde(remote = "ProductBranchReferenceDescription", deny_unknown_fields)]
pub(super) struct ReferenceDescription {
    pub format_version: u16,
    pub owner_ordinal: u64,
    #[serde(with = "home_owner")]
    pub home_owner: Option<worth_foundational::facade::DurableWorldOwnerIdentity>,
    pub branch_name: String,
    pub lifecycle_ordinal: u64,
    pub reference_generation: u64,
    pub composite_commit_ordinal: u64,
}

mod home_owner {
    use super::*;
    use worth_foundational::facade::{
        DurableWorldOwnerIdentity, HomeIdentity, WorldOwnerSlotIdentity,
    };

    pub(super) fn serialize<S: serde::Serializer>(
        owner: &Option<DurableWorldOwnerIdentity>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        owner
            .map(|owner| (owner.home().bytes(), owner.slot().get()))
            .serialize(serializer)
    }

    pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<DurableWorldOwnerIdentity>, D::Error> {
        Option::<([u8; 16], u64)>::deserialize(deserializer)?
            .map(|(home, slot)| {
                let home = HomeIdentity::from_observed_bytes(home)
                    .map_err(|_| serde::de::Error::custom("zero Home identity"))?;
                Ok(DurableWorldOwnerIdentity::from_recorded(
                    home,
                    WorldOwnerSlotIdentity::from_recorded(slot),
                ))
            })
            .transpose()
    }
}
