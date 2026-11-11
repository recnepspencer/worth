//! The selected payload survives the actual source and target arena owners.
use super::*;
use crate::identity::data::{EntityId, RecordId, RelationId};
use crate::storage::substrate::RelationRecordKind;

fn check<K: RecordKind>(
    make_extra: impl Fn(&ExecutionRequest<'_, '_>) -> K::Extra,
    retained_bill: impl Fn(&K::Extra) -> u64,
    id: RecordId<K::Domain>,
) where
    K::Extra: PartialEq,
{
    for existing in [false, true] {
        let source_budget = SerialMemoryBudget::new(Some(128 * 1024));
        let source_host = request(&source_budget);
        let source_request = ExecutionRequest::serial(&source_host);
        let mut source = RecordArena::<K>::with_capacity(1);
        source
            .write_reserved_slot(
                SlotInit {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    version_id: VersionId(1),
                    extra: make_extra(&source_request),
                },
                0,
                1,
            )
            .unwrap();
        let target_budget = SerialMemoryBudget::new(Some(128 * 1024));
        let target_host = request(&target_budget);
        let target_request = ExecutionRequest::serial(&target_host);
        let mut target = if existing {
            source.clone()
        } else {
            RecordArena::with_capacity(1)
        };
        let zero = SerialMemoryBudget::new(Some(0));
        let zero_host = request(&zero);
        assert!(matches!(
            target.select_retained_version(
                &ExecutionRequest::serial(&zero_host),
                &id,
                source.get(&id).unwrap(),
                VersionId(2),
            ),
            Err(crate::storage::substrate::MetadataWriteDenial::Request(_))
        ));
        assert_eq!(target.slot_count(), usize::from(existing));
        assert_eq!(zero.charged_memory_bytes(), 0);
        target
            .select_retained_version(&target_request, &id, source.get(&id).unwrap(), VersionId(2))
            .unwrap();
        let installed = target.get(&id).unwrap().retained_extra();
        let original = source.get(&id).unwrap().retained_extra();
        assert_ne!(installed.id(), original.id());
        assert_eq!(installed.as_ref(), original.as_ref());
        assert_eq!(
            K::extra_reachable_bound_bytes(&installed),
            K::extra_reachable_bound_bytes(&original)
        );
        let history = target.metadata_history_at(0).unwrap();
        assert_eq!(
            K::metadata_extra(&history[history.len() - 1]).id(),
            installed.id()
        );
        let bill = retained_bill(installed.as_ref());
        drop((original, source, target));
        assert_eq!(source_budget.charged_memory_bytes(), 0);
        assert_eq!(target_budget.charged_memory_bytes(), bill);
        assert!(bill > StorageAllocation::<K::Extra>::layout_bytes());
        drop(installed);
        assert_eq!(target_budget.charged_memory_bytes(), 0);
    }
}

#[test]
fn selected_entity_extra_retains_its_successor_bill_after_source_retirement() {
    check::<EntityRecordKind>(
        |request| extra(request, "retained selection"),
        expected,
        EntityId::new(PartitionId::main(), 0, 1),
    );
}

#[test]
fn selected_relation_extra_retains_its_successor_bill_after_source_retirement() {
    check::<RelationRecordKind>(
        |request| {
            let mut extra =
                RelationExtra::apply_payload(request, None, &patch("retained relation").1).unwrap();
            extra.endpoints = Some(RelationEndpoints {
                source: EntityId::new(PartitionId::main(), 1, 1),
                target: EntityId::new(PartitionId::main(), 2, 1),
            });
            extra
        },
        |extra| {
            StorageAllocation::<RelationExtra>::layout_bytes()
                + extra.authoritative_aspect_state().map_or(0, |state| {
                    u64::try_from(state.aspects().retained_storage_bytes().unwrap()).unwrap()
                })
        },
        RelationId::new(PartitionId::main(), 0, 1),
    );
}
