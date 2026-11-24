//! Descriptive role and producer meaning carried by canonical output.
use crate::identity::data::{EntityId, KindId};
use crate::transactions::data::primitives::EntityReference;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RequiredOutputPublicationMeaning {
    pub implementation: super::producing_implementation::RequiredOutputProducingImplementation,
    pub family: String,
    pub source_scope: EntityId,
    pub source_partition: worth_foundational::CanonicalDigestId,
    pub source_epoch: worth_foundational::CanonicalDigestId,
}

impl RequiredOutputPublicationMeaning {
    // A new build succeeds the same authored route. Exact readmission still
    // compares the complete original meaning, including its implementation.
    pub(crate) fn same_route(&self, other: &Self) -> bool {
        self.family == other.family
            && self.source_scope == other.source_scope
            && self.source_partition == other.source_partition
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RequiredOutputRolePosture {
    Create,
    Preserve,
    Retire,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequiredOutputRoleTarget {
    pub role: String,
    pub entity_name: String,
    pub kind: KindId,
    pub posture: RequiredOutputRolePosture,
    pub target: EntityReference,
}

#[cfg(test)]
pub(crate) fn native_roles_fixture(
    roles: &[(EntityId, KindId)],
    posture: RequiredOutputRolePosture,
) -> Vec<RequiredOutputRoleTarget> {
    roles
        .iter()
        .enumerate()
        .map(|(index, (id, kind))| RequiredOutputRoleTarget {
            role: match posture {
                RequiredOutputRolePosture::Create => format!("output-{index}"),
                _ => format!("preserved-{index}"),
            },
            entity_name: "native-unit-output".into(),
            kind: *kind,
            posture,
            target: EntityReference::Existing(*id),
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn native_meaning_fixture(family: &str) -> RequiredOutputPublicationMeaning {
    RequiredOutputPublicationMeaning {
        implementation:
            super::producing_implementation::RequiredOutputProducingImplementation::from_checkpoint(
                [0x11; 16], [0x22; 32],
            )
            .unwrap(),
        family: family.to_owned(),
        source_scope: EntityId::new(crate::identity::data::PartitionId::main(), 0, 1),
        source_partition: worth_foundational::CanonicalDigestId::new([0; 32]),
        source_epoch: worth_foundational::CanonicalDigestId::new([0; 32]),
    }
}
