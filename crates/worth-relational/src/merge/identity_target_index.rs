use std::collections::BTreeMap;

use crate::identity::data::{EntityId, LineageId, RelationId};
use crate::merge::aspect_components::{
    binding_component_from_visible_record, MergeAspectComponent, VisibleRecordSide,
};
use crate::merge::aspect_plan_lookup::lowered_plan_for_record;
use crate::merge::data::{
    IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope, VisibleMergeRecord,
    VisibleMergeRecordKind,
};
use crate::merge::identity_digest::{declared_identity_signature, DeclaredIdentityDigest};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::transactions::data::RecordRef;
use crate::visibility::snapshot_read_view::RelationalReadView;
use worth_foundational::facade::AspectKey;

pub(crate) struct TargetIdentityIndex {
    entities_by_storage: BTreeMap<EntityId, EntityReadRecord>,
    pub(crate) entities_by_lineage: BTreeMap<LineageId, Vec<EntityId>>,
    relations_by_storage: BTreeMap<RelationId, RelationReadRecord>,
    declared_key_indexes:
        BTreeMap<DeclaredKeyIndexKey, BTreeMap<DeclaredIdentityDigest, Vec<RecordRef>>>,
}

impl TargetIdentityIndex {
    pub(crate) fn new(
        runtime: &crate::runtime::RelationalRuntime,
        target_view: &RelationalReadView,
        source_records: &[VisibleMergeRecord],
        effective_identity_declarations: &[IdentityBasisDeclaration],
    ) -> Self {
        let relevant_entity_kinds = source_records
            .iter()
            .filter(|record| record.record_kind == VisibleMergeRecordKind::Entity)
            .flat_map(|record| [record.source_kind_id, record.target_kind_id, record.kind_id])
            .flatten()
            .collect::<std::collections::BTreeSet<_>>();
        let relevant_relation_kinds = source_records
            .iter()
            .filter(|record| record.record_kind == VisibleMergeRecordKind::Relation)
            .flat_map(|record| [record.source_kind_id, record.target_kind_id, record.kind_id])
            .flatten()
            .collect::<std::collections::BTreeSet<_>>();
        let target_entities = target_view
            .entities()
            .iter()
            .filter(|record| relevant_entity_kinds.contains(&record.kind.kind_id))
            .cloned()
            .collect::<Vec<_>>();
        let target_relations = target_view
            .relations()
            .iter()
            .filter(|record| relevant_relation_kinds.contains(&record.kind.kind_id))
            .cloned()
            .collect::<Vec<_>>();
        let entities_by_storage = target_view
            .entities()
            .iter()
            .filter(|record| relevant_entity_kinds.contains(&record.kind.kind_id))
            .cloned()
            .map(|record| (record.entity_id, record))
            .collect::<BTreeMap<_, _>>();
        let relations_by_storage = target_view
            .relations()
            .iter()
            .filter(|record| relevant_relation_kinds.contains(&record.kind.kind_id))
            .cloned()
            .map(|record| (record.relation_id, record))
            .collect::<BTreeMap<_, _>>();
        let entities_by_lineage = target_entities_by_lineage(target_entities.as_slice());
        let visible_records = target_visible_records(target_entities, target_relations);
        let declared_key_indexes =
            build_declared_key_indexes(runtime, &visible_records, effective_identity_declarations);
        runtime
            .performance_access()
            .count_merge_identity_target_indexing(
                target_view.entities().len() + target_view.relations().len(),
                visible_records.len(),
            );
        Self {
            entities_by_storage,
            entities_by_lineage,
            relations_by_storage,
            declared_key_indexes,
        }
    }

    pub(crate) fn storage_match(&self, record_ref: &RecordRef) -> Option<RecordRef> {
        match record_ref {
            RecordRef::Entity(entity_id) => self
                .entities_by_storage
                .contains_key(entity_id)
                .then_some(RecordRef::Entity(*entity_id)),
            RecordRef::Relation(relation_id) => self
                .relations_by_storage
                .contains_key(relation_id)
                .then_some(RecordRef::Relation(*relation_id)),
        }
    }

    fn declared_key_matches(
        &self,
        record_kind: VisibleMergeRecordKind,
        kind_id: Option<crate::identity::data::KindId>,
        aspect_keys: &[AspectKey],
    ) -> &BTreeMap<DeclaredIdentityDigest, Vec<RecordRef>> {
        let key = DeclaredKeyIndexKey {
            record_kind,
            kind_id,
            aspect_keys: aspect_keys.to_vec(),
        };
        self.declared_key_indexes
            .get(&key)
            .unwrap_or(&EMPTY_DECLARED_KEY_INDEX)
    }
}

pub(crate) enum DeclaredKeySetMatch {
    ExactTarget(RecordRef),
    MissingSourceEvidence,
    AmbiguousTarget,
    NoTargetMatch,
}

pub(crate) fn declared_key_set_match(
    runtime: &crate::runtime::RelationalRuntime,
    source: &VisibleMergeRecord,
    aspect_keys: &[AspectKey],
    target_index: &TargetIdentityIndex,
) -> DeclaredKeySetMatch {
    let Some(source_signature) = extract_declared_key_signature(runtime, source, aspect_keys)
    else {
        return DeclaredKeySetMatch::MissingSourceEvidence;
    };
    let matching_targets = target_index
        .declared_key_matches(source.record_kind.clone(), source.kind_id, aspect_keys)
        .get(&source_signature)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|target_record| *target_record != source.record_ref)
        .collect::<Vec<_>>();

    match matching_targets.as_slice() {
        [] => DeclaredKeySetMatch::NoTargetMatch,
        [target_record] => DeclaredKeySetMatch::ExactTarget(target_record.clone()),
        _ => DeclaredKeySetMatch::AmbiguousTarget,
    }
}

fn target_entities_by_lineage(
    target_entities: &[EntityReadRecord],
) -> BTreeMap<LineageId, Vec<EntityId>> {
    let mut entities_by_lineage = BTreeMap::<LineageId, Vec<EntityId>>::new();
    for entity in target_entities {
        if let Some(lineage_id) = entity.lineage_id {
            entities_by_lineage
                .entry(lineage_id)
                .or_default()
                .push(entity.entity_id);
        }
    }
    entities_by_lineage
}

fn target_visible_records(
    target_entities: Vec<EntityReadRecord>,
    target_relations: Vec<RelationReadRecord>,
) -> Vec<VisibleMergeRecord> {
    target_entities
        .into_iter()
        .map(|entity| VisibleMergeRecord {
            record_ref: RecordRef::Entity(entity.entity_id),
            record_kind: VisibleMergeRecordKind::Entity,
            kind_id: Some(entity.kind.kind_id),
            source_kind_id: Some(entity.kind.kind_id),
            target_kind_id: None,
            lineage_id: entity.lineage_id,
            source_lineage_id: entity.lineage_id,
            target_lineage_id: None,
            source_entity: Some(entity),
            target_entity: None,
            source_relation: None,
            target_relation: None,
        })
        .chain(
            target_relations
                .into_iter()
                .map(|relation| VisibleMergeRecord {
                    record_ref: RecordRef::Relation(relation.relation_id),
                    record_kind: VisibleMergeRecordKind::Relation,
                    kind_id: Some(relation.kind.kind_id),
                    source_kind_id: Some(relation.kind.kind_id),
                    target_kind_id: None,
                    lineage_id: None,
                    source_lineage_id: None,
                    target_lineage_id: None,
                    source_entity: None,
                    target_entity: None,
                    source_relation: Some(relation),
                    target_relation: None,
                }),
        )
        .collect::<Vec<_>>()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DeclaredKeyIndexKey {
    record_kind: VisibleMergeRecordKind,
    kind_id: Option<crate::identity::data::KindId>,
    aspect_keys: Vec<AspectKey>,
}

#[derive(Debug, Default)]
struct DeclaredKeyDeclarationCatalog {
    kind_scoped: BTreeMap<IdentityBasisScope, Vec<Vec<AspectKey>>>,
    aspect_scoped: BTreeMap<AspectKey, Vec<Vec<AspectKey>>>,
}

static EMPTY_DECLARED_KEY_INDEX: std::sync::LazyLock<
    BTreeMap<DeclaredIdentityDigest, Vec<RecordRef>>,
> = std::sync::LazyLock::new(BTreeMap::new);

fn build_declared_key_indexes(
    runtime: &crate::runtime::RelationalRuntime,
    visible_records: &[VisibleMergeRecord],
    effective_identity_declarations: &[IdentityBasisDeclaration],
) -> BTreeMap<DeclaredKeyIndexKey, BTreeMap<DeclaredIdentityDigest, Vec<RecordRef>>> {
    let catalog = build_declared_key_declaration_catalog(effective_identity_declarations);
    let mut indexes =
        BTreeMap::<DeclaredKeyIndexKey, BTreeMap<DeclaredIdentityDigest, Vec<RecordRef>>>::new();
    for record in visible_records {
        let Some(kind_id) = record.kind_id else {
            continue;
        };
        let kind_scope = match record.record_kind {
            VisibleMergeRecordKind::Entity => IdentityBasisScope::EntityKind(kind_id),
            VisibleMergeRecordKind::Relation => IdentityBasisScope::RelationKind(kind_id),
        };
        for keys in applicable_declared_key_sets(runtime, record, &catalog, &kind_scope) {
            let Some(signature) = extract_declared_key_signature(runtime, record, keys.as_slice())
            else {
                continue;
            };
            let index_key = DeclaredKeyIndexKey {
                record_kind: record.record_kind.clone(),
                kind_id: record.kind_id,
                aspect_keys: keys.clone(),
            };
            indexes
                .entry(index_key)
                .or_default()
                .entry(signature)
                .or_default()
                .push(record.record_ref.clone());
        }
    }
    indexes
}

fn build_declared_key_declaration_catalog(
    effective_identity_declarations: &[IdentityBasisDeclaration],
) -> DeclaredKeyDeclarationCatalog {
    let mut catalog = DeclaredKeyDeclarationCatalog::default();
    for declaration in effective_identity_declarations {
        let IdentityBasisKind::DeclaredKeySet(keys) = &declaration.basis else {
            continue;
        };
        let key_vec = keys.to_vec();
        match &declaration.scope {
            IdentityBasisScope::EntityKind(_) | IdentityBasisScope::RelationKind(_) => {
                push_unique_key_set(
                    catalog
                        .kind_scoped
                        .entry(declaration.scope.clone())
                        .or_default(),
                    &key_vec,
                );
            }
            IdentityBasisScope::AspectKey(aspect_key) => {
                push_unique_key_set(
                    catalog.aspect_scoped.entry(aspect_key.clone()).or_default(),
                    &key_vec,
                );
            }
        }
    }
    catalog
}

fn applicable_declared_key_sets(
    runtime: &crate::runtime::RelationalRuntime,
    record: &VisibleMergeRecord,
    catalog: &DeclaredKeyDeclarationCatalog,
    kind_scope: &IdentityBasisScope,
) -> Vec<Vec<AspectKey>> {
    let mut applicable = catalog
        .kind_scoped
        .get(kind_scope)
        .cloned()
        .unwrap_or_default();
    let Some(plan) = lowered_plan_for_record(runtime, record) else {
        return applicable;
    };
    for binding in &plan.executable_bindings {
        if let Some(key_sets) = catalog.aspect_scoped.get(binding.aspect_key()) {
            for key_set in key_sets {
                push_unique_key_set(&mut applicable, key_set);
            }
        }
    }
    applicable
}

fn extract_declared_key_signature(
    runtime: &crate::runtime::RelationalRuntime,
    record: &VisibleMergeRecord,
    aspect_keys: &[AspectKey],
) -> Option<DeclaredIdentityDigest> {
    let plan = lowered_plan_for_record(runtime, record)?;
    let bindings = aspect_keys
        .iter()
        .map(|aspect_key| {
            plan.executable_bindings
                .iter()
                .find(|binding| binding.aspect_key() == aspect_key)
        })
        .collect::<Option<Vec<_>>>()?;
    let components = bindings
        .into_iter()
        .map(|binding| extract_identity_component(runtime, record, binding))
        .collect::<Option<Vec<_>>>()?;
    declared_identity_signature(&components)
}

fn extract_identity_component(
    _runtime: &crate::runtime::RelationalRuntime,
    record: &VisibleMergeRecord,
    binding: &crate::schema::data::LoweredAspectContractBinding,
) -> Option<MergeAspectComponent> {
    binding_component_from_visible_record(record, binding, VisibleRecordSide::Source)
}

fn push_unique_key_set(target: &mut Vec<Vec<AspectKey>>, candidate: &[AspectKey]) {
    if !target
        .iter()
        .any(|existing| existing.as_slice() == candidate)
    {
        target.push(candidate.to_vec());
    }
}
