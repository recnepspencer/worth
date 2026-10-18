use std::collections::BTreeSet;

use crate::domain_installation::{
    WorthQueryBoundCollectionWindow, WorthQueryCollectionDeliveryCounters,
    WorthQueryCollectionPatchFact, WorthQueryNativeAccessKey,
};
use crate::memory_workspace::{WorthQueryEntity, WorthQueryEntityIdentity};
use crate::projection_consumption::ConsumedNativeValue;

use super::MaintenanceRow;

pub(super) struct PatchFactRequest<'a> {
    pub(super) selected: &'a [MaintenanceRow],
    pub(super) prior: &'a WorthQueryBoundCollectionWindow,
    pub(super) affected: &'a BTreeSet<WorthQueryEntityIdentity>,
    pub(super) affected_keys: &'a [WorthQueryNativeAccessKey],
    pub(super) selected_keys: &'a [WorthQueryNativeAccessKey],
}

pub(super) fn patch_facts(
    request: PatchFactRequest<'_>,
    counters: &mut WorthQueryCollectionDeliveryCounters,
) -> Vec<WorthQueryCollectionPatchFact> {
    let prior_identities = request
        .prior
        .rows()
        .iter()
        .map(|row| row.entity_identity())
        .collect::<BTreeSet<_>>();
    let mut facts = Vec::new();
    for row in request.selected {
        let was_mounted = prior_identities.contains(&row.consumer_identity);
        let keys = if !was_mounted {
            request.selected_keys
        } else if request.affected.contains(&row.consumer_identity) {
            request.affected_keys
        } else {
            continue;
        };
        for key in keys {
            facts.push(WorthQueryCollectionPatchFact::new(
                row.consumer_identity.clone(),
                key.clone(),
                native_value(&row.entity, key),
            ));
            counters.native_facts_materialized += 1;
        }
    }
    facts
}

pub(super) fn native_value(
    entity: &WorthQueryEntity,
    key: &WorthQueryNativeAccessKey,
) -> ConsumedNativeValue {
    if let Some(field) = key.field_path().native_field_key() {
        let relative = worth_foundational::facade::CanonicalFieldPath::new([field.clone()])
            .expect("native access keys retain one field");
        if let Some(value) =
            crate::memory_workspace::aspect_relative_scalar(entity, key.contract_key(), &relative)
                .cloned()
        {
            return ConsumedNativeValue::scalar(value);
        }
    }
    if let Some(path) = key
        .field_path()
        .canonical_source_path()
        .or_else(|| key.field_path().canonical_field_path())
    {
        return entity
            .scalar_value_at(path)
            .cloned()
            .map(ConsumedNativeValue::scalar)
            .unwrap_or_else(|| ConsumedNativeValue::absent(key.absence_posture()));
    }
    if let Some(path) = native_storage_path(key)
        && let Some(value) = entity.scalar_value_at(&path).cloned()
    {
        return ConsumedNativeValue::scalar(value);
    }
    if let Some(value) = entity.struct_aspect_value(key.contract_key()) {
        return ConsumedNativeValue::struct_value(value.clone());
    }
    entity
        .aspect_value(key.contract_key())
        .cloned()
        .map(ConsumedNativeValue::scalar)
        .unwrap_or_else(|| ConsumedNativeValue::absent(key.absence_posture()))
}

pub(super) fn native_storage_path(
    key: &WorthQueryNativeAccessKey,
) -> Option<worth_foundational::facade::CanonicalFieldPath> {
    let field = key.field_path().native_field_key()?.clone();
    let relative = worth_foundational::facade::CanonicalFieldPath::new([field])?;
    crate::memory_workspace::normalized_native_storage_path(key.contract_key(), &relative)
}

pub(super) fn grouping_identity(
    entity: &WorthQueryEntity,
    fields: &[worth_query_installation::facade::WorthQueryOperationCollectionField],
) -> Vec<String> {
    fields
        .iter()
        .map(|field| {
            let value = crate::memory_workspace::aspect_relative_scalar(
                entity,
                field.aspect_key(),
                field.field_path(),
            );
            value
                .map(worth_foundational::facade::prepare_aspect_value_identity_basis)
                .map(|value| value.as_str().to_owned())
                .unwrap_or_else(|| "absent".to_owned())
        })
        .collect()
}
