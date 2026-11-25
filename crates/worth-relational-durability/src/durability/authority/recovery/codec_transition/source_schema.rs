//! Source-carrier readmission and supported consequence admission before restore.
use crate::durability::data::DurableCheckpoint;
use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use contracts::indexes::data::definition::{DerivedIndexDefinition, DerivedIndexKind};
use contracts::schema::codec_field_comparison::CodecFieldLocus;
use contracts::schema::data::{RelationalSchemaRegistry, RelationalSchemaRegistryFieldsPort as _};
use worth_foundational::FoundationalBranchTarget;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::validation::custom_registry::FrozenCustomInvariantRegistry;

pub mod relation_consequences;
pub use relation_consequences::admit_retired_relation_consequences;

pub fn readmit_current_source_schema(
    checkpoint: &DurableCheckpoint,
    main_branch: &contracts::history::data::coordinates::BranchId,
    available: &contracts::content::registry::RelationalContentFormatRegistry,
) -> Result<
    RelationalSchemaRegistry,
    contracts::durability::data::recovery_stop::RelationalRecoveryStop,
> {
    for cell in checkpoint
        .branch_cells
        .iter()
        .filter(|cell| &cell.branch_id == main_branch)
    {
        let FoundationalBranchTarget::Basis(target) = cell.observation.target() else {
            continue;
        };
        let (schema_carrier_digest, carrier) = if let Some(root) = checkpoint
            .branch_roots
            .iter()
            .find(|root| root.commit_id.0 == target.selected_commit_id())
        {
            let carrier = checkpoint
                .branch_root_schema_images
                .iter()
                .find(|carrier| carrier.carrier_digest() == root.schema_carrier_digest)
                .ok_or_else(|| {
                    schema_denial("current source schema carrier is unavailable".into())
                })?;
            (root.schema_carrier_digest, carrier)
        } else if let Some(directory) = checkpoint
            .directory
            .as_ref()
            .or(checkpoint.canonical_directory.as_ref())
        {
            let root = directory
                .manifest
                .roots
                .iter()
                .find(|root| root.commit_id.0 == target.selected_commit_id())
                .ok_or_else(|| schema_denial("current source root image is unavailable".into()))?;
            let carrier = directory
                .manifest
                .schemas
                .iter()
                .find(|carrier| carrier.carrier_digest() == root.schema_carrier_digest)
                .ok_or_else(|| {
                    schema_denial("current source schema carrier is unavailable".into())
                })?;
            (root.schema_carrier_digest, carrier)
        } else {
            return Err(schema_denial("current source root image is unavailable".into()).into());
        };
        let _ = schema_carrier_digest;
        if carrier.schema_root() != *target.roots().schema_root() {
            return Err(schema_denial(
                "current source schema carrier differs from its exact target".into(),
            )
            .into());
        }
        let (mut registry, _) = carrier.readmit_registry().map_err(schema_denial)?;
        (*registry.owner_content_formats_mut()) =
            (*registry.owner_content_formats()).bind_recorded(available)?;
        return Ok(registry);
    }
    Err(schema_denial("codec transition requires a committed current source schema".into()).into())
}

pub fn admit_supported_consequences<'a>(
    registrations: &FrozenCustomInvariantRegistry,
    indexes: impl IntoIterator<Item = &'a DerivedIndexDefinition>,
    changes: &[CodecFieldLocus],
) -> Result<(), String> {
    for invariant in registrations.iter() {
        let access = &invariant.descriptor().operational.access;
        if changes.iter().any(|change| {
            access.read_entity_kinds.contains(&change.kind_id)
                || access.affected_entity_kinds.contains(&change.kind_id)
        }) {
            return Err(
                "codec transition with custom invariant consequences is unsupported".into(),
            );
        }
    }
    for group in registrations.source_inventory().iter() {
        let access = group.access_contract();
        if changes.iter().any(|change| {
            access.read_entity_kinds.contains(&change.kind_id)
                || access.affected_entity_kinds.contains(&change.kind_id)
        }) {
            return Err("codec transition with managed Source consequences is unsupported".into());
        }
    }
    for index in indexes {
        let unsupported = match &index.kind {
            DerivedIndexKind::RelatedEntityOrdering {
                child_kind,
                ordering,
                ..
            } => changes.iter().any(|change| {
                change.kind_id == *child_kind
                    && ordering.iter().any(|field| {
                        let locator = field.locator();
                        locator.aspect().aspect_key() == &change.aspect_key
                            && locator.field_path().fields().first() == Some(&change.field_key)
                    })
            }),
            DerivedIndexKind::RelationJoin(join) => changes.iter().any(|change| {
                change.kind_id == join.shared_entity_kind()
                    || change.kind_id == join.left().external_entity_kind()
                    || change.kind_id == join.right().external_entity_kind()
            }),
            DerivedIndexKind::EntityField { .. } | DerivedIndexKind::RelationField { .. } => false,
        };
        if unsupported {
            return Err(
                "codec transition has an unsupported ordering or join index consequence".into(),
            );
        }
    }
    Ok(())
}

pub fn admit_single_codec_head(checkpoint: &DurableCheckpoint) -> Result<(), String> {
    if checkpoint.branch_cells.len() != 1 {
        return Err("codec transition requires one current branch; other source-contract heads cannot be adopted partially".into());
    }
    Ok(())
}

pub fn schema_denial(detail: String) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::SchemaMismatch, detail)
}

#[cfg(test)]
pub mod tests;
