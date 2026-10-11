use std::collections::BTreeSet;

use worth_foundational::facade::AspectFieldLocator;

use crate::runtime::RelationalRuntime;
use crate::storage::data::AuthoritativeFieldComparisonKey;

use super::projected_field_values::{
    build_entity_aspect_field_index, entity_aspect_field_index_entry,
};

pub(crate) fn refresh_unique_entity_aspect_field_index_for_records(
    runtime: &RelationalRuntime,
    changed_records: &[crate::transactions::data::RecordRef],
    basis: &crate::mvcc::PreparedIndexRefreshBasis,
) {
    let tracked_fields = tracked_unique_entity_aspect_fields(runtime);
    if tracked_fields.is_empty() {
        return;
    }
    let refreshed_values = collect_changed_unique_entity_aspect_field_entries(
        runtime,
        basis,
        changed_records,
        &tracked_fields,
    );
    remove_changed_entities_from_unique_entity_aspect_field_index(
        runtime,
        changed_records,
        &tracked_fields,
    );
    write_unique_entity_aspect_field_index_entries(runtime, refreshed_values);
}

pub(crate) fn rebuild_unique_entity_aspect_field_indexes(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> Result<(), crate::indexes::data::DerivedIndexMaintenanceDenialKind> {
    let tracked_fields = tracked_unique_entity_aspect_fields(runtime);
    if tracked_fields.is_empty() {
        runtime.indexes.clear_unique_index();
        return Ok(());
    }
    let branch_id = runtime.config.history.main_branch.clone();
    let Some(head) = runtime.history().branch_head(&branch_id) else {
        runtime.indexes.clear_unique_index();
        return Ok(());
    };
    let projection = runtime
        .read_truth()
        .project_branch_head(&branch_id, head.version_id)
        .map_err(crate::indexes::data::DerivedIndexMaintenanceDenialKind::Basis)?;
    let Some(projection) = projection else {
        return Ok(());
    };
    let rebuilt_values =
        collect_all_unique_entity_aspect_field_entries(execution, &projection, &tracked_fields)
            .map_err(crate::indexes::data::DerivedIndexMaintenanceDenialKind::Execution)?;
    runtime.indexes.clear_unique_index();
    write_unique_entity_aspect_field_index_entries(runtime, rebuilt_values);
    Ok(())
}

fn remove_changed_entities_from_unique_entity_aspect_field_index(
    runtime: &RelationalRuntime,
    changed_records: &[crate::transactions::data::RecordRef],
    tracked_fields: &BTreeSet<AspectFieldLocator>,
) {
    runtime.indexes.with_unique_index_mut(|index| {
        for record in changed_records {
            let crate::transactions::data::RecordRef::Entity(entity_id) = record else {
                continue;
            };
            for field_locator in tracked_fields {
                if let Some(values) = index.get_mut(field_locator) {
                    values.retain(|_, entity_ids| {
                        entity_ids.remove(entity_id);
                        !entity_ids.is_empty()
                    });
                }
            }
        }
    });
}

fn write_unique_entity_aspect_field_index_entries(
    runtime: &RelationalRuntime,
    entries: Vec<(
        AspectFieldLocator,
        AuthoritativeFieldComparisonKey,
        crate::identity::data::EntityId,
    )>,
) {
    runtime.indexes.with_unique_index_mut(|index| {
        for (field_locator, value, entity_id) in entries {
            index
                .entry(field_locator)
                .or_default()
                .entry(value)
                .or_default()
                .insert(entity_id);
        }
    });
}

fn collect_all_unique_entity_aspect_field_entries(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &crate::runtime::VisibilityProjectionView<'_>,
    tracked_fields: &BTreeSet<AspectFieldLocator>,
) -> Result<
    Vec<(
        AspectFieldLocator,
        AuthoritativeFieldComparisonKey,
        crate::identity::data::EntityId,
    )>,
    crate::execution::RelationalExecutionDenialCause,
> {
    let mut entries = Vec::new();
    let source = super::projected_field_values::IndexProjectionSource::exact(projection)
        .expect("current unique-index projection must carry an exact basis");
    for field_locator in tracked_fields {
        entries.extend(
            build_entity_aspect_field_index(execution, &source, field_locator)?
                .into_iter()
                .flat_map(|(value, entity_ids)| {
                    entity_ids
                        .into_iter()
                        .map(move |entity_id| (field_locator.clone(), value.clone(), entity_id))
                }),
        );
    }
    Ok(entries)
}

fn collect_changed_unique_entity_aspect_field_entries(
    runtime: &RelationalRuntime,
    basis: &crate::mvcc::PreparedIndexRefreshBasis,
    changed_records: &[crate::transactions::data::RecordRef],
    tracked_fields: &BTreeSet<AspectFieldLocator>,
) -> Vec<(
    AspectFieldLocator,
    AuthoritativeFieldComparisonKey,
    crate::identity::data::EntityId,
)> {
    let mut entries = Vec::new();
    let reader = runtime.read_truth();
    for record in changed_records {
        let crate::transactions::data::RecordRef::Entity(entity_id) = record else {
            continue;
        };
        let Some(record) = reader.authoritative_entity_record_for_id_at_version(
            basis.root(),
            *entity_id,
            basis.version_id(),
        ) else {
            continue;
        };
        for field_locator in tracked_fields {
            if let Some((value, entity_id)) =
                entity_aspect_field_index_entry(runtime, &record, field_locator)
            {
                entries.push((field_locator.clone(), value, entity_id));
            }
        }
    }
    entries
}

fn tracked_unique_entity_aspect_fields(
    runtime: &RelationalRuntime,
) -> BTreeSet<AspectFieldLocator> {
    let mut fields = BTreeSet::new();
    for registration in &runtime.config.schema.invariant_catalog.registrations {
        if let crate::validation::data::InvariantRule::UniqueEntityAspectField { field_locator } =
            &registration.rule
        {
            fields.insert(field_locator.clone());
        }
    }
    fields
}
