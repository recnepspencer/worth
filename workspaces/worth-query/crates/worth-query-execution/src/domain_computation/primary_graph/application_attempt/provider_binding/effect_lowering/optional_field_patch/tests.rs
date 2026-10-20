//! Local lowering contracts; real request/commit evidence lives in the consumer.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;
use worth_foundational::facade::{AspectContractRevision, AspectIdentity, AspectValue};
use worth_relational::facade::identity::{KindId, PartitionId};

#[test]
fn optional_aspect_initialization_lowering_requires_exact_presence_evidence() {
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let aspect = AspectKey::new("assessment").unwrap();
    let locator = worth_relational::facade::transactions::planned_single_field_locator(
        aspect.clone(),
        FieldKey::new("finding").unwrap(),
    );
    let absent = Fact::AbsentField {
        entity_id: entity,
        kind: KindId(1),
        locator: locator.clone(),
    };
    let writes = || {
        BTreeMap::from([(
            locator.clone(),
            WorthQueryApplicationOptionalFieldWrite {
                contract: PortableAspectContractBasis::new(
                    aspect.clone(),
                    AspectIdentity(7),
                    AspectContractRevision(1),
                ),
                value: Some(AspectValue::Bool(false)),
            },
        )])
    };
    assert!(portable_patch(
        &ObservedFactIndex::new(std::slice::from_ref(&absent)),
        entity,
        writes(),
    )
    .is_err());
    for present in [false, true] {
        let facts = [
            absent.clone(),
            Fact::AspectPresence {
                entity_id: entity,
                kind: KindId(1),
                aspect: aspect.clone(),
                present,
            },
        ];
        let patch = portable_patch(&ObservedFactIndex::new(&facts), entity, writes()).unwrap();
        assert_eq!(
            matches!(
                &patch.operations()[0],
                PortableAspectPatchOperation::SetWhole { .. }
            ),
            !present
        );
        assert_eq!(
            matches!(
                &patch.operations()[0],
                PortableAspectPatchOperation::PatchFields { .. }
            ),
            present
        );
    }
    let contradictory = [
        absent,
        Fact::AspectPresence {
            entity_id: entity,
            kind: KindId(1),
            aspect: aspect.clone(),
            present: false,
        },
        Fact::AspectPresence {
            entity_id: entity,
            kind: KindId(1),
            aspect: aspect.clone(),
            present: true,
        },
    ];
    assert!(portable_patch(&ObservedFactIndex::new(&contradictory), entity, writes()).is_err());
}

#[test]
fn optional_aspect_initialization_clear_preserves_absence_without_materialization() {
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let aspect = AspectKey::new("assessment").unwrap();
    let locator = worth_relational::facade::transactions::planned_single_field_locator(
        aspect.clone(),
        FieldKey::new("finding").unwrap(),
    );
    let facts = [Fact::AspectPresence {
        entity_id: entity,
        kind: KindId(1),
        aspect: aspect.clone(),
        present: false,
    }];
    let patch = portable_patch(
        &ObservedFactIndex::new(&facts),
        entity,
        BTreeMap::from([(
            locator,
            WorthQueryApplicationOptionalFieldWrite {
                contract: PortableAspectContractBasis::new(
                    aspect,
                    AspectIdentity(7),
                    AspectContractRevision(1),
                ),
                value: None,
            },
        )]),
    )
    .unwrap();
    assert!(patch.is_empty());
}
