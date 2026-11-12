//! Original-reference meaning survives mechanical sequential application.
use super::*;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::symbols::data::{ClientKey, StringInterner};
use crate::transactions::data::CreatedEntityRef;

fn entity(key: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId::new(4),
        client_key: ClientKey::raw(key),
    }
}
fn relation(source: EntityReference, target: EntityReference) -> CreatedRelationRef {
    CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId::new(8),
        client_key: ClientKey::raw("writer-link"),
        source,
        target,
    }
}

#[test]
fn original_created_endpoints_survive_prefix_lowering_in_either_position() {
    let symbols = StringInterner::default();
    let prefix_key = entity("restored-anchor");
    let prefix_id = EntityId::new(PartitionId::main(), 11, 1);
    let existing = EntityReference::Existing(EntityId::new(PartitionId::main(), 12, 1));
    let created = EntityReference::Created(prefix_key.clone());
    let prefix = CommitCreatedEntityBindings::from_owner_map(
        BTreeMap::from([(prefix_key, prefix_id)]),
        &symbols,
    );
    let canonical_prefix = prefix.canonical_meaning(&symbols).unwrap();
    let created_entities = canonical_prefix
        .iter()
        .map(|created| (created.assigned, &created.key))
        .collect();
    for (source, target) in [
        (created.clone(), existing.clone()),
        (existing, created.clone()),
        (created.clone(), created),
    ] {
        let original = relation(source, target);
        let lower = |endpoint: &EntityReference| match endpoint {
            EntityReference::Existing(id) => EntityReference::Existing(*id),
            EntityReference::Created(_) => EntityReference::Existing(prefix_id),
        };
        let executed = relation(lower(&original.source), lower(&original.target));
        let assigned = RelationId::new(PartitionId::main(), 17, 2);
        let generated = CommitCreatedRelationBindings::from_owner_map(
            BTreeMap::from([(executed.clone(), assigned)]),
            &symbols,
        );
        let completed =
            complete_originals(vec![original.clone()], generated, &prefix, &symbols, |_| {
                Ok(())
            })
            .unwrap();
        assert_eq!(completed.resolve(&original), Some(assigned));
        assert_eq!(completed.resolve(&executed), None);
        let semantic = completed
            .canonical_meaning(&symbols, &created_entities)
            .unwrap();
        assert_eq!(semantic.len(), 1);
        assert_eq!(semantic[0].assigned, assigned);
        let other = CreatedRelationRef {
            client_key: ClientKey::raw("different-original"),
            ..original
        };
        assert_eq!(completed.resolve(&other), None);
    }
}

#[test]
fn suffix_local_created_endpoints_keep_their_original_form() {
    let symbols = StringInterner::default();
    let prefix = CommitCreatedEntityBindings::from_owner_map(BTreeMap::new(), &symbols);
    let original = relation(
        EntityReference::Created(entity("suffix-local")),
        EntityReference::Existing(EntityId::new(PartitionId::main(), 12, 1)),
    );
    let assigned = RelationId::new(PartitionId::main(), 17, 2);
    let generated = CommitCreatedRelationBindings::from_owner_map(
        BTreeMap::from([(original.clone(), assigned)]),
        &symbols,
    );
    let completed =
        complete_originals(vec![original.clone()], generated, &prefix, &symbols, |_| {
            Ok(())
        })
        .unwrap();
    assert_eq!(completed.resolve(&original), Some(assigned));
}

#[test]
fn original_symbol_and_text_aliases_share_one_completed_correspondence() {
    let mut symbols = StringInterner::default();
    let anchor = symbols.intern("restored-anchor").unwrap();
    let link = symbols.intern("writer-link").unwrap();
    let prefix_key = CreatedEntityRef {
        client_key: ClientKey::symbol(anchor),
        ..entity("unused")
    };
    let prefix_id = EntityId::new(PartitionId::main(), 11, 1);
    let target = EntityReference::Existing(EntityId::new(PartitionId::main(), 12, 1));
    let original = CreatedRelationRef {
        client_key: ClientKey::symbol(link),
        ..relation(EntityReference::Created(prefix_key.clone()), target.clone())
    };
    let executed = CreatedRelationRef {
        source: EntityReference::Existing(prefix_id),
        ..original.clone()
    };
    let assigned = RelationId::new(PartitionId::main(), 17, 2);
    let prefix = CommitCreatedEntityBindings::from_owner_map(
        BTreeMap::from([(prefix_key, prefix_id)]),
        &symbols,
    );
    let canonical_prefix = prefix.canonical_meaning(&symbols).unwrap();
    let created_entities = canonical_prefix
        .iter()
        .map(|created| (created.assigned, &created.key))
        .collect();
    let generated = CommitCreatedRelationBindings::from_owner_map(
        BTreeMap::from([(executed, assigned)]),
        &symbols,
    );
    let completed =
        complete_originals(vec![original.clone()], generated, &prefix, &symbols, |_| {
            Ok(())
        })
        .unwrap();
    let raw = relation(EntityReference::Created(entity("restored-anchor")), target);
    assert_eq!(completed.resolve(&original), Some(assigned));
    assert_eq!(completed.resolve(&raw), Some(assigned));
    assert_eq!(
        completed
            .canonical_meaning(&symbols, &created_entities)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn missing_extra_repeated_and_different_original_associations_are_refused() {
    let symbols = StringInterner::default();
    let prefix = CommitCreatedEntityBindings::from_owner_map(BTreeMap::new(), &symbols);
    let original = relation(
        EntityReference::Existing(EntityId::new(PartitionId::main(), 11, 1)),
        EntityReference::Existing(EntityId::new(PartitionId::main(), 12, 1)),
    );
    let other = CreatedRelationRef {
        client_key: ClientKey::raw("other"),
        ..original.clone()
    };
    let assigned = RelationId::new(PartitionId::main(), 17, 2);
    for (originals, entries) in [
        (vec![original.clone()], BTreeMap::new()),
        (vec![], BTreeMap::from([(original.clone(), assigned)])),
        (
            vec![original.clone(), original.clone()],
            BTreeMap::from([(original.clone(), assigned)]),
        ),
        (vec![original], BTreeMap::from([(other, assigned)])),
    ] {
        let generated = CommitCreatedRelationBindings::from_owner_map(entries, &symbols);
        assert!(complete_originals(originals, generated, &prefix, &symbols, |_| Ok(())).is_err());
    }
}
