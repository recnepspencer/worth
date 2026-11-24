//! Selected payload and latest metadata retain the same actual value carrier.
use super::*;
use crate::identity::data::{EntityId, PartitionId, RelationId};
use crate::storage::substrate::{EntityExtra, RelationEndpoints, RelationExtra};
use worth_foundational::facade::*;
use worth_proof::TransitionOutcome;

fn state(text: &str) -> AuthoritativeRecordAspectState {
    let contract = AspectContract::scalar(
        AspectKey::new("data").unwrap(),
        AspectIdentity(1),
        AspectContractRevision(1),
        ScalarAspectType::String,
    );
    let TransitionOutcome::Success(value) = validate_aspect_value(
        &contract,
        ContractValidationInput::Scalar(AspectValue::String(InternedString::Raw(text.into()))),
    ) else {
        panic!("valid source value")
    };
    let TransitionOutcome::Success(patch) =
        AuthoritativeRecordAspectPatch::whole_aspect([value], [])
    else {
        panic!("valid source patch")
    };
    let TransitionOutcome::Success(state) = patch.apply_to_optional(None) else {
        panic!("valid source state")
    };
    state.unwrap().into_parts().into_parts().0
}

fn check<K: RecordKind>(extra: K::Extra, id: RecordId<K::Domain>)
where
    K::Extra: PartialEq,
{
    for existing in [false, true] {
        let mut source = RecordArena::<K>::with_capacity(1);
        source
            .write_reserved_slot(
                SlotInit {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    version_id: VersionId(1),
                    extra: extra.clone(),
                },
                0,
                1,
            )
            .unwrap();
        let mut target = if existing {
            source.clone()
        } else {
            RecordArena::with_capacity(1)
        };
        target
            .select_retained_version_using(None, &id, source.get(&id).unwrap(), VersionId(2))
            .unwrap();
        let installed = target.get(&id).unwrap().retained_extra();
        let original = source.get(&id).unwrap().retained_extra();
        assert_ne!(installed.id(), original.id());
        assert_eq!(installed.as_ref(), original.as_ref());
        let history = target.metadata_history_at(0).unwrap();
        assert_eq!(
            K::metadata_extra(&history[history.len() - 1]).id(),
            installed.id()
        );
        drop((original, source, target));
        assert_eq!(installed.as_ref(), &extra);
    }
}

#[test]
fn selected_entity_payload_outlives_source_and_matches_latest_history() {
    check::<EntityRecordKind>(
        EntityExtra::from_unbudgeted_parts(None, None, Some(state("retained selection"))),
        EntityId::new(PartitionId::main(), 0, 1),
    );
}

#[test]
fn selected_relation_payload_outlives_source_and_matches_latest_history() {
    check::<RelationRecordKind>(
        RelationExtra::from_unbudgeted_parts(
            Some(RelationEndpoints {
                source: EntityId::new(PartitionId::main(), 1, 1),
                target: EntityId::new(PartitionId::main(), 2, 1),
            }),
            Some(state("retained relation")),
        ),
        RelationId::new(PartitionId::main(), 0, 1),
    );
}
