use std::collections::BTreeSet;

use sha2::Digest;

use crate::capabilities::RuntimeConfigSource;
use crate::merge::data::{
    BranchTouchedRecordDelta, MergeSchemaKindClass, MergeSchemaKindSemanticSnapshot,
    MergeSchemaSnapshotDigestBasis, VisibleMergeRecord, VisibleMergeRecordKind,
};
use crate::runtime::RelationalRuntime;
use crate::schema::data::RelationalSchemaRegistry;
use crate::transactions::data::RecordRef;

pub(crate) fn merge_schema_snapshot_for_execution_ready(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    target_version_id: crate::identity::data::VersionId,
    source_records: &[VisibleMergeRecord],
    target_touched_records: &[BranchTouchedRecordDelta],
) -> MergeSchemaSnapshotDigestBasis {
    let target_view = runtime
        .read_truth()
        .read_version(execution, target_version_id);
    merge_schema_snapshot(
        &runtime.runtime_config().schema.registry,
        source_records,
        &target_view,
        target_touched_records,
    )
}

pub(super) fn merge_schema_snapshot(
    registry: &RelationalSchemaRegistry,
    source_records: &[VisibleMergeRecord],
    target_view: &crate::storage::data::RelationalReadView,
    target_touched_records: &[BranchTouchedRecordDelta],
) -> MergeSchemaSnapshotDigestBasis {
    let touched_kind_sets =
        collect_touched_schema_kind_sets(source_records, target_touched_records, target_view);
    let touched_kinds = materialize_touched_schema_kind_snapshots(registry, touched_kind_sets);

    MergeSchemaSnapshotDigestBasis {
        authoritative_schema_id: touched_kinds.first().map(|kind| kind.schema_id.clone()),
        authoritative_schema_version_id: touched_kinds.first().map(|kind| kind.schema_version_id),
        registry_digest: schema_registry_digest(registry),
        touched_kinds: std::sync::Arc::from(touched_kinds),
    }
}

struct TouchedSchemaKindSets {
    entity_kinds: BTreeSet<crate::identity::data::KindId>,
    relation_kinds: BTreeSet<crate::identity::data::KindId>,
}

fn collect_touched_schema_kind_sets(
    source_records: &[VisibleMergeRecord],
    target_touched_records: &[BranchTouchedRecordDelta],
    target_view: &crate::storage::data::RelationalReadView,
) -> TouchedSchemaKindSets {
    let mut touched_entity_kinds = BTreeSet::new();
    let mut touched_relation_kinds = BTreeSet::new();
    collect_source_schema_kinds(
        source_records,
        &mut touched_entity_kinds,
        &mut touched_relation_kinds,
    );
    collect_target_schema_kinds(
        target_touched_records,
        target_view,
        &mut touched_entity_kinds,
        &mut touched_relation_kinds,
    );
    TouchedSchemaKindSets {
        entity_kinds: touched_entity_kinds,
        relation_kinds: touched_relation_kinds,
    }
}

fn collect_source_schema_kinds(
    source_records: &[VisibleMergeRecord],
    touched_entity_kinds: &mut BTreeSet<crate::identity::data::KindId>,
    touched_relation_kinds: &mut BTreeSet<crate::identity::data::KindId>,
) {
    for record in source_records {
        for kind_id in [record.source_kind_id, record.target_kind_id]
            .into_iter()
            .flatten()
        {
            match record.record_kind {
                VisibleMergeRecordKind::Entity => {
                    touched_entity_kinds.insert(kind_id);
                }
                VisibleMergeRecordKind::Relation => {
                    touched_relation_kinds.insert(kind_id);
                }
            }
        }
    }
}

fn collect_target_schema_kinds(
    target_touched_records: &[BranchTouchedRecordDelta],
    target_view: &crate::storage::data::RelationalReadView,
    touched_entity_kinds: &mut BTreeSet<crate::identity::data::KindId>,
    touched_relation_kinds: &mut BTreeSet<crate::identity::data::KindId>,
) {
    for delta in target_touched_records {
        match &delta.target {
            RecordRef::Entity(entity_id) => {
                if let Some(entity) = target_view.get_entity(*entity_id) {
                    touched_entity_kinds.insert(entity.kind.kind_id);
                }
            }
            RecordRef::Relation(relation_id) => {
                if let Some(relation) = target_view.get_relation(*relation_id) {
                    touched_relation_kinds.insert(relation.kind.kind_id);
                }
            }
        }
    }
}

fn materialize_touched_schema_kind_snapshots(
    registry: &RelationalSchemaRegistry,
    touched_kind_sets: TouchedSchemaKindSets,
) -> Vec<MergeSchemaKindSemanticSnapshot> {
    let mut touched_kinds = Vec::new();
    for kind_id in touched_kind_sets.entity_kinds {
        if let Ok(registration) = registry.entity_registration(kind_id) {
            touched_kinds.push(MergeSchemaKindSemanticSnapshot {
                kind_class: MergeSchemaKindClass::Entity,
                kind_id,
                kind_name: registration.kind_name.clone(),
                schema_id: registration.schema_id.clone(),
                schema_version_id: registration.schema_version_id,
                aspect_plan_revision: registration.aspect_contract_declarations.plan_revision,
                identity_declarations: registration
                    .aspect_contract_declarations
                    .identity_declarations
                    .clone(),
                merge_policy_declarations: registration
                    .aspect_contract_declarations
                    .merge_policy_declarations
                    .clone(),
                relation_integrity_plan_revision: None,
            });
        }
    }
    for kind_id in touched_kind_sets.relation_kinds {
        if let Ok(registration) = registry.relation_registration(kind_id) {
            touched_kinds.push(MergeSchemaKindSemanticSnapshot {
                kind_class: MergeSchemaKindClass::Relation,
                kind_id,
                kind_name: registration.kind_name.clone(),
                schema_id: registration.schema_id.clone(),
                schema_version_id: registration.schema_version_id,
                aspect_plan_revision: registration.aspect_contract_declarations.plan_revision,
                identity_declarations: registration
                    .aspect_contract_declarations
                    .identity_declarations
                    .clone(),
                merge_policy_declarations: registration
                    .aspect_contract_declarations
                    .merge_policy_declarations
                    .clone(),
                relation_integrity_plan_revision: Some(
                    registration.relation_integrity.plan_revision,
                ),
            });
        }
    }

    touched_kinds.sort_by(|left, right| {
        left.kind_class
            .cmp(&right.kind_class)
            .then(left.kind_id.cmp(&right.kind_id))
    });
    touched_kinds
}

fn schema_registry_digest(registry: &RelationalSchemaRegistry) -> String {
    let digest = sha2::Sha256::digest(registry.authority_digest_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
