//! Declared meaning and roles for the court's installed native producers.
use worth_foundational::CanonicalDigestId;
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    mvcc::{
        RequiredOutputProducingImplementation, RequiredOutputPublication,
        RequiredOutputPublicationMeaning, RequiredOutputRolePosture, RequiredOutputRoleTarget,
    },
    transactions::{CreatedEntityRef, EntityReference},
};

pub(super) fn roles(
    targets: &[(EntityId, KindId)],
    posture: RequiredOutputRolePosture,
) -> Vec<RequiredOutputRoleTarget> {
    targets
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

pub(super) fn meaning(family: &str) -> RequiredOutputPublicationMeaning {
    RequiredOutputPublicationMeaning {
        implementation: RequiredOutputProducingImplementation::from_checkpoint(
            [0x11; 16], [0x22; 32],
        )
        .unwrap(),
        family: family.to_owned(),
        source_scope: EntityId::new(PartitionId::main(), 0, 1),
        source_partition: CanonicalDigestId::new([0; 32]),
        source_epoch: CanonicalDigestId::new([0; 32]),
    }
}

pub(super) fn birth(
    admission: worth_proof::AdmittedRequiredOutputPublication,
    targets: Vec<CreatedEntityRef>,
) -> RequiredOutputPublication {
    let meaning = meaning(admission.producer());
    RequiredOutputPublication::Birth {
        admission,
        absence: None,
        meaning,
        roles: targets
            .into_iter()
            .enumerate()
            .map(|(index, target)| RequiredOutputRoleTarget {
                role: format!("output-{index}"),
                entity_name: "native-unit-output".into(),
                kind: target.kind_id,
                posture: RequiredOutputRolePosture::Create,
                target: EntityReference::Created(target),
            })
            .collect(),
    }
}
