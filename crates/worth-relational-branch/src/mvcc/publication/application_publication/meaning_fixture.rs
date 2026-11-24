//! Test meanings for the Native publication courts.
use super::output_meaning::{
    RequiredOutputPublicationMeaning, RequiredOutputRolePosture, RequiredOutputRoleTarget,
};
use crate::identity::data::{EntityId, KindId};
use crate::transactions::data::primitives::EntityReference;

#[doc(hidden)]
pub fn native_roles_for_test(
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

#[doc(hidden)]
pub fn native_meaning_for_test(family: &str) -> RequiredOutputPublicationMeaning {
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
