use super::super::*;
use super::state::{NativeExternalRow, StatefulBridgeState};
use worth_foundational::facade::{
    AspectValue, CanonicalFieldPath, ContractValidatedAspectValueView, EntityId, FieldKey,
    PartitionId,
};
use worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts;

use crate::memory_workspace::{WorthQueryCommitIdentity, WorthQuerySnapshotIdentity};
use crate::runtime::backend::build_bridge_authority_bundle;

use super::SharedState;

pub(super) fn execute_write(
    shared: &SharedState,
    mutation: WorthQueryBackendAdmissibleMutation,
) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
    let mut state = shared.borrow_mut();
    let collection = mutation
        .declared_collection_identity()
        .map(|collection| collection.as_str().to_string())
        .or_else(|| {
            mutation.existing_truth_binding().and_then(|binding| {
                binding
                    .terminal_target_collection_projection()
                    .map(str::to_string)
            })
        })
        .or_else(|| {
            mutation
                .declared_entity_identity_ref()
                .and_then(|identity| {
                    state
                        .collection_by_identity
                        .get(&identity.terminal_projection_for_reporting())
                        .cloned()
                })
        })
        .ok_or_else(|| {
            WorthQueryWorkspaceError::new("stateful bridge could not resolve collection")
        })?;
    let (entity_identity, entity_identity_text) = match mutation.mutation_family() {
        WorthQueryMutationFamily::Insert => {
            state.next_entity_identity += 1;
            let identity = WorthQueryEntityIdentity::from_relational_record(
                RelationalBridgeRecordIdentityParts::entity(
                    1,
                    state.next_entity_identity as u64,
                    0,
                ),
            );
            let identity_text = identity.terminal_projection_for_reporting();
            (identity, identity_text)
        }
        _ => {
            let identity = mutation
                .declared_entity_identity_ref()
                .cloned()
                .or_else(|| {
                    mutation
                        .existing_truth_binding()
                        .map(|binding| binding.resolved_target_identity().clone())
                })
                .or_else(|| {
                    mutation.symbolic_target_reference().and_then(|reference| {
                        state.identity_by_symbol.get(reference.symbol()).cloned()
                    })
                })
                .ok_or_else(|| {
                    WorthQueryWorkspaceError::new(
                        "stateful bridge could not resolve target entity identity",
                    )
                })?;
            let identity_text = mutation
                .symbolic_target_reference()
                .and_then(|reference| state.identity_text_by_symbol.get(reference.symbol()))
                .cloned()
                .unwrap_or_else(|| identity.terminal_projection_for_reporting());
            (identity, identity_text)
        }
    };
    let mutation_kind = apply_command(
        &mut state,
        &mutation,
        &collection,
        &entity_identity,
        &entity_identity_text,
    )?;
    state.next_commit_identity += 1;
    state.next_snapshot_token += 1;
    let commit_identity =
        WorthQueryCommitIdentity::from_relational_commit_id(state.next_commit_identity as u64);
    let snapshot_identity = WorthQuerySnapshotIdentity::from_relational_snapshot(
        worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts::new(
            1,
            state.next_snapshot_token as u64,
        ),
    );
    let bridge_authority = build_bridge_authority_bundle(
        &state.bridge,
        &snapshot_identity,
        &mutation,
        crate::runtime::WorthQueryBridgeMutationTarget::new(
            &collection,
            &entity_identity,
            mutation_kind.clone(),
        ),
    )?;
    Ok(test_mutation_receipt_with_bridge_authority(
        commit_identity,
        snapshot_identity,
        collection,
        entity_identity,
        mutation_kind,
        mutation.declared_aspect_touches(),
        bridge_authority,
    ))
}

pub(super) fn apply_command(
    state: &mut StatefulBridgeState,
    mutation: &WorthQueryBackendAdmissibleMutation,
    collection: &str,
    entity_identity: &WorthQueryEntityIdentity,
    entity_identity_text: &str,
) -> Result<WorthQueryMutationKind, WorthQueryWorkspaceError> {
    let entity_identity_key = entity_identity
        .terminal_projection_for_reporting()
        .to_string();
    match mutation.mutation_family() {
        WorthQueryMutationFamily::Insert => {
            let external_row = external_row_from_mutation(state, mutation)?;
            state
                .rows_by_collection
                .entry(collection.to_string())
                .or_default()
                .insert(entity_identity_key.clone(), external_row);
            state
                .collection_by_identity
                .insert(entity_identity_key.clone(), collection.to_string());
            state
                .identity_by_storage_key
                .insert(entity_identity_key.clone(), entity_identity.clone());
            if let Some(reference) = mutation.symbolic_target_reference() {
                state
                    .identity_by_symbol
                    .insert(reference.symbol().to_string(), entity_identity.clone());
                state.identity_text_by_symbol.insert(
                    reference.symbol().to_string(),
                    entity_identity_text.to_string(),
                );
            }
            Ok(WorthQueryMutationKind::Created)
        }
        WorthQueryMutationFamily::Update | WorthQueryMutationFamily::Assertion => {
            let symbolic_values = resolved_symbolic_aspect_values(state, mutation)?;
            let row = state
                .rows_by_collection
                .entry(collection.to_string())
                .or_default()
                .get_mut(&entity_identity_key)
                .ok_or_else(|| {
                    WorthQueryWorkspaceError::new(format!(
                        "stateful bridge update could not find `{entity_identity_key}` in `{collection}`"
                    ))
                })?;
            apply_authoritative_patch_to_external_row(row, mutation.authoritative_patch())?;
            apply_resolved_symbolic_values(row, symbolic_values)?;
            Ok(WorthQueryMutationKind::Updated)
        }
        WorthQueryMutationFamily::Delete => {
            if let Some(rows) = state.rows_by_collection.get_mut(collection) {
                rows.remove(&entity_identity_key);
            }
            state.collection_by_identity.remove(&entity_identity_key);
            state.identity_by_storage_key.remove(&entity_identity_key);
            state.identity_by_symbol.retain(|_, resolved_identity| {
                resolved_identity.evidence_identity() != entity_identity.evidence_identity()
            });
            state
                .identity_text_by_symbol
                .retain(|_, resolved_identity| resolved_identity != entity_identity_text);
            Ok(WorthQueryMutationKind::Deleted)
        }
    }
}

fn external_row_from_mutation(
    state: &StatefulBridgeState,
    mutation: &WorthQueryBackendAdmissibleMutation,
) -> Result<NativeExternalRow, WorthQueryWorkspaceError> {
    let mut external_row = NativeExternalRow::new();
    apply_authoritative_patch_to_external_row(&mut external_row, mutation.authoritative_patch())?;
    let symbolic_values = resolved_symbolic_aspect_values(state, mutation)?;
    apply_resolved_symbolic_values(&mut external_row, symbolic_values)?;
    Ok(external_row)
}

fn apply_authoritative_patch_to_external_row(
    external_row: &mut NativeExternalRow,
    patch: &worth_foundational::facade::AuthoritativeRecordAspectPatch,
) -> Result<(), WorthQueryWorkspaceError> {
    for aspect_key in patch.whole_aspect_clears() {
        external_row.retain(|path, _| {
            path.fields()
                .first()
                .is_none_or(|field| field.as_str() != aspect_key.as_str())
        });
    }
    for (aspect_key, validated) in patch.whole_aspect_sets() {
        external_row.retain(|path, _| {
            path.fields()
                .first()
                .is_none_or(|field| field.as_str() != aspect_key.as_str())
        });
        match validated.view() {
            ContractValidatedAspectValueView::Scalar(value) => {
                set_external_row_touch(
                    external_row,
                    &WorthQueryAspectTouch::whole_aspect(aspect_key.clone()),
                    value.clone(),
                )?;
            }
            ContractValidatedAspectValueView::Struct(value) => {
                for (field, value) in value.fields() {
                    set_external_row_touch(
                        external_row,
                        &WorthQueryAspectTouch::aspect_field_path(
                            aspect_key.clone(),
                            CanonicalFieldPath::single(field.clone()),
                        ),
                        value.clone(),
                    )?;
                }
            }
        }
    }
    for (aspect_key, field_patch) in patch.field_patches() {
        for field in field_patch.field_clears() {
            external_row.remove(&native_external_field_path_for_touch(
                &WorthQueryAspectTouch::aspect_field_path(
                    aspect_key.clone(),
                    CanonicalFieldPath::single(field.clone()),
                ),
            )?);
        }
        for (field, value) in field_patch.field_sets() {
            set_external_row_touch(
                external_row,
                &WorthQueryAspectTouch::aspect_field_path(
                    aspect_key.clone(),
                    CanonicalFieldPath::single(field.clone()),
                ),
                value.clone(),
            )?;
        }
    }
    Ok(())
}

fn resolved_symbolic_aspect_values(
    state: &StatefulBridgeState,
    mutation: &WorthQueryBackendAdmissibleMutation,
) -> Result<Vec<(WorthQueryAspectTouch, AspectValue)>, WorthQueryWorkspaceError> {
    mutation
        .symbolic_aspect_references()
        .iter()
        .map(|reference| {
            let identity = state
                .identity_by_symbol
                .get(reference.reference().symbol())
                .and_then(WorthQueryEntityIdentity::relational_entity_record_parts)
                .ok_or_else(|| {
                    WorthQueryWorkspaceError::new(format!(
                        "stateful bridge could not resolve native symbolic entity reference `{}`",
                        reference.reference().symbol()
                    ))
                })?;
            Ok((
                reference.aspect_touch().clone(),
                AspectValue::EntityRef(EntityId::new(
                    PartitionId(identity.partition_id()),
                    identity.local_slot(),
                    identity.generation(),
                )),
            ))
        })
        .collect()
}

fn apply_resolved_symbolic_values(
    external_row: &mut NativeExternalRow,
    values: Vec<(WorthQueryAspectTouch, AspectValue)>,
) -> Result<(), WorthQueryWorkspaceError> {
    for (touch, value) in values {
        set_external_row_touch(external_row, &touch, value)?;
    }
    Ok(())
}

fn set_external_row_touch(
    external_row: &mut NativeExternalRow,
    aspect_touch: &WorthQueryAspectTouch,
    value: AspectValue,
) -> Result<(), WorthQueryWorkspaceError> {
    external_row.insert(native_external_field_path_for_touch(aspect_touch)?, value);
    Ok(())
}

pub(super) fn native_external_field_path_for_touch(
    aspect_touch: &WorthQueryAspectTouch,
) -> Result<CanonicalFieldPath, WorthQueryWorkspaceError> {
    let mut fields = vec![
        FieldKey::new(aspect_touch.native_aspect_key().as_str()).ok_or_else(|| {
            WorthQueryWorkspaceError::new(format!(
                "stateful bridge could not use native aspect `{}` as an external field",
                aspect_touch.native_aspect_key().as_str()
            ))
        })?,
    ];
    if let Some(field_path) = aspect_touch.native_field_path() {
        fields.extend(field_path.fields().iter().cloned());
    }
    CanonicalFieldPath::new(fields).ok_or_else(|| {
        WorthQueryWorkspaceError::new(format!(
            "stateful bridge could not derive external field path for `{}`",
            aspect_touch.admitted_touch_digest_part()
        ))
    })
}

pub(super) fn external_row_text_at_path(
    external_row: &NativeExternalRow,
    field_path: &CanonicalFieldPath,
) -> Option<String> {
    match external_row.get(field_path)? {
        AspectValue::String(value) => Some(match value {
            worth_foundational::facade::InternedString::Stored(value) => {
                let value = value.as_string();
                value.clone()
            }
            worth_foundational::facade::InternedString::Raw(value) => value.clone(),
            worth_foundational::facade::InternedString::Symbol(symbol) => {
                format!("symbol:{}", symbol.0)
            }
        }),
        AspectValue::Int8(value) => Some(value.to_string()),
        AspectValue::Int16(value) => Some(value.to_string()),
        AspectValue::Int32(value) => Some(value.to_string()),
        AspectValue::Int64(value) => Some(value.to_string()),
        AspectValue::UInt8(value) => Some(value.to_string()),
        AspectValue::UInt16(value) => Some(value.to_string()),
        AspectValue::UInt32(value) => Some(value.to_string()),
        AspectValue::UInt64(value) => Some(value.to_string()),
        AspectValue::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}
