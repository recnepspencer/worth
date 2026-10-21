//! Source-carrier readmission and supported consequence admission before restore.
use crate::durability::data::{DurabilityError, DurableCheckpoint, RecoveryFailureClass};
use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexKind};
use crate::runtime::{CodecFieldLocus, RelationalRuntime};
use crate::schema::data::RelationalSchemaRegistry;
use worth_foundational::FoundationalBranchTarget;

mod relation_consequences;
pub(super) use relation_consequences::admit_retired_relation_consequences;

pub(super) fn readmit_current_source_schema(
    checkpoint: &DurableCheckpoint,
    main_branch: &crate::history::data::BranchId,
) -> Result<RelationalSchemaRegistry, DurabilityError> {
    for cell in checkpoint
        .branch_cells
        .iter()
        .filter(|cell| &cell.branch_id == main_branch)
    {
        let FoundationalBranchTarget::Basis(target) = cell.observation.target() else {
            continue;
        };
        let root = checkpoint
            .branch_roots
            .iter()
            .find(|root| root.commit_id.0 == target.selected_commit_id())
            .ok_or_else(|| schema_denial("current source root image is unavailable".into()))?;
        let carrier = checkpoint
            .branch_root_schema_images
            .iter()
            .find(|carrier| carrier.carrier_digest() == root.schema_carrier_digest)
            .ok_or_else(|| schema_denial("current source schema carrier is unavailable".into()))?;
        if carrier.schema_root() != *target.roots().schema_root() {
            return Err(schema_denial(
                "current source schema carrier differs from its exact target".into(),
            ));
        }
        let (registry, _) = carrier.readmit_registry().map_err(schema_denial)?;
        return Ok(registry);
    }
    Err(schema_denial(
        "codec transition requires a committed current source schema".into(),
    ))
}

pub(super) fn admit_supported_consequences<'a>(
    runtime: &RelationalRuntime,
    indexes: impl IntoIterator<Item = &'a DerivedIndexDefinition>,
    changes: &[CodecFieldLocus],
) -> Result<(), String> {
    for invariant in runtime
        .schema_contract_runtime
        .custom_invariant_registries
        .iter()
    {
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

pub(super) fn admit_single_codec_head(checkpoint: &DurableCheckpoint) -> Result<(), String> {
    if checkpoint.branch_cells.len() != 1 {
        return Err("codec transition requires one current branch; other source-contract heads cannot be adopted partially".into());
    }
    Ok(())
}

pub(super) fn schema_denial(detail: String) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::SchemaMismatch, detail)
}
