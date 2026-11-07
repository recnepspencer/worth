use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::FieldKey;
use worth_runtime_bridge::facade::BridgeMaterializedFieldProjection;

use super::super::super::consumed::ConsumedNativeValue;
use super::super::super::contracts::MaterializedProjectionContract;
use super::super::super::facts::{
    projection_fact_field_path_from_segments, ProjectionFactFieldPath, ProjectionFactKind,
};
use super::super::super::identity::compose_scoped_row_source_identity;
use super::super::ProjectionFactExtractionError;
use super::consumed_scalar_value::consumed_scalar_value_from_entity_path;
use crate::memory_workspace::WorthQueryEntity;

#[derive(Clone, Debug)]
pub(super) struct ProjectionMaterializedField {
    field_path: ProjectionFactFieldPath,
    value: ConsumedNativeValue,
}

impl ProjectionMaterializedField {
    pub(super) fn from_relational_projected_aspect_key(
        _contract: &MaterializedProjectionContract,
        _row_identity: &str,
        aspect_key: &worth_foundational::facade::AspectKey,
        value: &worth_runtime_bridge::facade::SnapshotReadValue,
    ) -> Result<Self, ProjectionFactExtractionError> {
        Ok(Self {
            field_path: ProjectionFactFieldPath::from_native_aspect_key(aspect_key.clone()),
            value: ConsumedNativeValue::from_snapshot_read_value(value),
        })
    }

    pub(super) fn from_bridge_field_value(
        contract: &MaterializedProjectionContract,
        row_identity: &str,
        projection: &BridgeMaterializedFieldProjection,
        value: ConsumedNativeValue,
    ) -> Result<Self, ProjectionFactExtractionError> {
        projection_fact_field_path_from_bridge_projection(projection)
            .map(|field_path| Self { field_path, value })
            .map_err(
                |_message| ProjectionFactExtractionError::InvalidDeclaredFieldValueShape {
                    source_family: contract.source_family(),
                    source_identity: compose_scoped_row_source_identity(
                        contract.source_identity(),
                        row_identity,
                    ),
                    field_key: projection.field_identity().as_str().to_string(),
                    fact_kind: ProjectionFactKind::DerivedField,
                    expected_shape: "bridge projection field locator",
                },
            )
    }

    fn into_parts(self) -> (ProjectionFactFieldPath, ConsumedNativeValue) {
        (self.field_path, self.value)
    }
}

pub(super) fn lower_materialized_fields<Fields>(
    contract: &MaterializedProjectionContract,
    fields: Fields,
) -> Result<BTreeMap<ProjectionFactFieldPath, ConsumedNativeValue>, ProjectionFactExtractionError>
where
    Fields: Iterator<Item = ProjectionMaterializedField>,
{
    let mut materialized = fields
        .map(ProjectionMaterializedField::into_parts)
        .collect();
    add_legacy_declared_aliases(contract, &mut materialized);
    Ok(materialized)
}

pub(in crate::projection_consumption::extraction) fn query_read_result_row_fields(
    contract: &MaterializedProjectionContract,
    row: &WorthQueryEntity,
) -> Result<BTreeMap<ProjectionFactFieldPath, ConsumedNativeValue>, ProjectionFactExtractionError> {
    let mut row_fields = row
        .aspect_values()
        .map(|(aspect_key, value)| {
            (
                ProjectionFactFieldPath::from_native_aspect_key(aspect_key.clone()),
                ConsumedNativeValue::scalar(value.clone()),
            )
        })
        .collect::<BTreeMap<_, _>>();

    row_fields.extend(row.struct_aspect_values().map(|(aspect_key, value)| {
        (
            ProjectionFactFieldPath::from_native_aspect_key(aspect_key.clone()),
            ConsumedNativeValue::struct_value(value.clone()),
        )
    }));

    add_legacy_declared_aliases(contract, &mut row_fields);

    for field_path in contract_declared_field_paths(contract) {
        if row_fields.contains_key(&field_path) {
            continue;
        }
        if let Some(value) = lookup_declared_scalar(contract, row, &field_path)? {
            row_fields.insert(field_path, ConsumedNativeValue::scalar(value));
        }
    }

    Ok(row_fields)
}

fn add_legacy_declared_aliases(
    contract: &MaterializedProjectionContract,
    materialized: &mut BTreeMap<ProjectionFactFieldPath, ConsumedNativeValue>,
) {
    let mut requested = contract
        .fact_families()
        .iter()
        .filter(|family| family.native_contract().is_none())
        .filter_map(|family| family.field_path())
        .filter(|path| path.canonical_field_path().is_some())
        .cloned()
        .collect::<BTreeSet<_>>();
    if contract
        .fact_families()
        .iter()
        .any(|family| family.kind() == ProjectionFactKind::EntityIdentity)
    {
        requested.insert(identity_field_path());
    }
    if requested.is_empty() {
        return;
    }
    let aliases = materialized
        .iter()
        .filter_map(|(native, value)| {
            legacy_alias_for_native_path(native)
                .filter(|alias| requested.contains(alias))
                .map(|alias| (alias, value.clone()))
        })
        .collect::<Vec<_>>();
    materialized.extend(aliases);
}

fn legacy_alias_for_native_path(
    native: &ProjectionFactFieldPath,
) -> Option<ProjectionFactFieldPath> {
    native
        .canonical_source_path()
        .cloned()
        .map(ProjectionFactFieldPath::from_canonical_field_path)
}

pub(super) fn identity_field_path() -> ProjectionFactFieldPath {
    projection_fact_field_path_from_segments([
        FieldKey::new("identity").expect("identity field key should admit"),
        FieldKey::new("id").expect("id field key should admit"),
    ])
}

fn contract_declared_field_paths(
    contract: &MaterializedProjectionContract,
) -> BTreeSet<ProjectionFactFieldPath> {
    let mut field_paths = contract
        .fact_families()
        .iter()
        .filter_map(|fact| match fact.kind() {
            ProjectionFactKind::EntityIdentity => Some(identity_field_path()),
            ProjectionFactKind::DisplayField | ProjectionFactKind::DerivedField => {
                fact.field_path().cloned()
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    field_paths.insert(identity_field_path());
    field_paths
}

fn lookup_declared_scalar(
    contract: &MaterializedProjectionContract,
    row: &WorthQueryEntity,
    field_path: &ProjectionFactFieldPath,
) -> Result<Option<worth_foundational::facade::AspectValue>, ProjectionFactExtractionError> {
    let value = if let Some(aspect) = field_path.native_aspect_key() {
        field_path.native_field_key().and_then(|field| {
            row.struct_aspect_values()
                .find(|(candidate, _)| *candidate == aspect)
                .and_then(|(_, value)| value.get(field))
                .cloned()
                .or_else(|| {
                    field_path
                        .canonical_source_path()
                        .and_then(|path| row.scalar_value_at(path))
                        .cloned()
                })
        })
    } else {
        consumed_scalar_value_from_entity_path(
            row,
            field_path
                .canonical_field_path()
                .expect("non-native projection path remains canonical"),
        )
        .map_err(|_message| {
            ProjectionFactExtractionError::InvalidDeclaredFieldValueShape {
                source_family: contract.source_family(),
                source_identity: compose_scoped_row_source_identity(
                    contract.source_identity(),
                    row.identity().evidence_identity().reporting_projection(),
                ),
                field_key: field_path.terminal_projection_for_boundary().to_string(),
                fact_kind: ProjectionFactKind::DerivedField,
                expected_shape: "foundational scalar",
            }
        })?
    };
    Ok(value)
}

fn projection_fact_field_path_from_bridge_projection(
    projection: &BridgeMaterializedFieldProjection,
) -> Result<ProjectionFactFieldPath, String> {
    if let Some(field_locator) = projection.field_locator() {
        if let [field] = field_locator.field_path().fields() {
            return Ok(ProjectionFactFieldPath::from_native_keys(
                projection.aspect_key().clone(),
                field.clone(),
            ));
        }
        return Ok(ProjectionFactFieldPath::from_canonical_field_path(
            field_locator.field_path().clone(),
        ));
    }
    Ok(ProjectionFactFieldPath::from_native_aspect_key(
        projection.aspect_key().clone(),
    ))
}
