use worth_foundational::facade::ContractValidatedAspectValueView;
use worth_runtime_bridge::facade::BridgeMaterializedRowSetArtifact;
use worth_runtime_bridge::facade::RelationalAuthoritativeRowSetArtifact;

use super::super::super::consumed::ConsumedNativeValue;
use super::super::super::consumed::ConsumedProjectionFactSet;
use super::super::super::contracts::MaterializedProjectionContract;
use super::materialized_rows::{extract_materialized_rows, RowIdentityExtractionMode};
use super::row_like_field_paths::{
    lower_materialized_fields, query_read_result_row_fields, ProjectionMaterializedField,
};
use crate::memory_workspace::WorthQueryEntity;
use crate::projection_consumption::ProjectionFactExtractionError;
use crate::runtime::{WorthQueryLiveReadResult, WorthQueryReadResult};

pub(in crate::projection_consumption::extraction) fn extract_relational_row_set_facts(
    contract: &MaterializedProjectionContract,
    row_set: &RelationalAuthoritativeRowSetArtifact,
) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
    super::super::ensure_contract_family(
        contract,
        super::super::super::source::ProjectionSourceFamily::RelationalRowSet,
    )?;
    super::super::ensure_source_identity(contract.source_identity(), row_set.digest().as_str())?;
    let materialized_rows = row_set
        .rows()
        .iter()
        .map(|row| {
            Ok((
                row.row_identity().as_str(),
                row.projected_aspect_values()
                    .iter()
                    .map(|(key, value)| {
                        ProjectionMaterializedField::from_relational_projected_aspect_key(
                            contract,
                            row.row_identity().as_str(),
                            key,
                            value,
                        )
                    })
                    .collect::<Result<Vec<_>, ProjectionFactExtractionError>>()?,
            ))
        })
        .collect::<Result<Vec<_>, ProjectionFactExtractionError>>()?;
    extract_field_map_rows(
        contract,
        materialized_rows
            .iter()
            .map(|(row_identity, fields)| (*row_identity, fields.iter().cloned())),
        RowIdentityExtractionMode::IdentityFieldBackedEntityIdentity,
    )
}

pub(in crate::projection_consumption::extraction) fn extract_bridge_row_set_facts(
    contract: &MaterializedProjectionContract,
    row_set: &BridgeMaterializedRowSetArtifact,
) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
    super::super::ensure_contract_family(
        contract,
        super::super::super::source::ProjectionSourceFamily::BridgeTruthViewRowSet,
    )?;
    super::super::ensure_source_identity(contract.source_identity(), row_set.digest().as_str())?;
    let materialized_rows = row_set
        .rows()
        .iter()
        .map(|row| {
            Ok((
                row.row_identity().as_str(),
                row.fields()
                    .values()
                    .map(|bridge_field| {
                        let value = match bridge_field.validated_value().payload().view() {
                            ContractValidatedAspectValueView::Scalar(value) => {
                                ConsumedNativeValue::scalar(value.clone())
                            }
                            ContractValidatedAspectValueView::Struct(value) => {
                                ConsumedNativeValue::struct_value(value.clone())
                            }
                        };
                        ProjectionMaterializedField::from_bridge_field_value(
                            contract,
                            row.row_identity().as_str(),
                            bridge_field.projection(),
                            value,
                        )
                    })
                    .collect::<Result<Vec<_>, ProjectionFactExtractionError>>()?,
            ))
        })
        .collect::<Result<Vec<_>, ProjectionFactExtractionError>>()?;
    extract_field_map_rows(
        contract,
        materialized_rows
            .iter()
            .map(|(row_identity, fields)| (*row_identity, fields.iter().cloned())),
        RowIdentityExtractionMode::IdentityFieldBackedEntityIdentity,
    )
}

pub(in crate::projection_consumption::extraction) fn extract_read_result_facts(
    contract: &MaterializedProjectionContract,
    result: &WorthQueryReadResult,
) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
    super::super::ensure_contract_family(
        contract,
        super::super::super::source::ProjectionSourceFamily::QueryReadReceipt,
    )?;
    super::super::ensure_source_identity(
        contract.source_identity(),
        result.receipt().read_graph_digest(),
    )?;
    extract_entity_rows(
        contract,
        result.rows(),
        RowIdentityExtractionMode::RowIdentityAsEntityIdentity,
    )
}

pub(in crate::projection_consumption::extraction) fn extract_live_read_result_facts(
    contract: &MaterializedProjectionContract,
    result: &WorthQueryLiveReadResult,
) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
    super::super::ensure_contract_family(
        contract,
        super::super::super::source::ProjectionSourceFamily::QueryLiveReadReceipt,
    )?;
    super::super::ensure_source_identity(
        contract.source_identity(),
        result.receipt().installation_digest(),
    )?;
    extract_entity_rows(
        contract,
        result.rows(),
        RowIdentityExtractionMode::RowIdentityAsEntityIdentity,
    )
}

fn extract_field_map_rows<'a, Rows, Fields>(
    contract: &MaterializedProjectionContract,
    rows: Rows,
    row_identity_mode: RowIdentityExtractionMode,
) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError>
where
    Rows: Iterator<Item = (&'a str, Fields)>,
    Fields: Iterator<Item = ProjectionMaterializedField>,
{
    let materialized_rows = rows
        .map(|(row_identity, fields)| {
            Ok((
                row_identity.to_string(),
                None,
                lower_materialized_fields(contract, fields)?,
            ))
        })
        .collect::<Result<Vec<_>, ProjectionFactExtractionError>>()?;
    extract_materialized_rows(
        contract,
        &materialized_rows,
        |_row_identity, field_map, field_key, _fact_kind| Ok(field_map.get(field_key)),
        row_identity_mode,
    )
}

fn extract_entity_rows(
    contract: &MaterializedProjectionContract,
    rows: &[WorthQueryEntity],
    row_identity_mode: RowIdentityExtractionMode,
) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
    let materialized_rows = rows
        .iter()
        .map(|row| {
            Ok((
                row.identity()
                    .evidence_identity()
                    .reporting_projection()
                    .to_string(),
                Some(row.identity().clone()),
                query_read_result_row_fields(contract, row)?,
            ))
        })
        .collect::<Result<Vec<_>, ProjectionFactExtractionError>>()?;
    extract_materialized_rows(
        contract,
        &materialized_rows,
        |_row_identity, row_fields, field_key, _fact_kind| Ok(row_fields.get(field_key)),
        row_identity_mode,
    )
}
