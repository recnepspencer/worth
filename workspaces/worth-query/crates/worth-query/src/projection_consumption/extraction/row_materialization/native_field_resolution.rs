use super::super::super::consumed::{ConsumedNativeValue, ConsumedNativeValueView};
use super::super::super::contracts::{BoundProjectionFactFamily, MaterializedProjectionContract};
use super::super::super::facts::{ProjectionFactFieldPath, ProjectionFactKind};
use super::super::super::identity::compose_scoped_row_source_identity;
use super::super::ProjectionFactExtractionError;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_foundational::facade::AspectValuePosture;

pub(in crate::projection_consumption::extraction) fn native_value_or_absence(
    contract: &MaterializedProjectionContract,
    family: &BoundProjectionFactFamily,
    row_identity: &str,
    value: Option<&ConsumedNativeValue>,
) -> Result<ConsumedNativeValue, ProjectionFactExtractionError> {
    let Some(native_contract) = family.native_contract() else {
        return value.cloned().ok_or_else(|| {
            missing_declared_field(
                contract,
                row_identity,
                family.field_path().expect("field family carries a path"),
                family.kind(),
            )
        });
    };
    let Some(value) = value else {
        return match native_contract.absence() {
            worth_foundational::facade::AbsenceLaw::Required => {
                Err(ProjectionFactExtractionError::MissingRequiredNativeFact {
                    source_family: contract.source_family(),
                    source_identity: compose_scoped_row_source_identity(
                        contract.source_identity(),
                        row_identity,
                    )?,
                    field_path: native_contract.field_path().clone(),
                    contract_key: native_contract.contract().key().clone(),
                    contract_revision: native_contract.contract().revision(),
                    projection_authority: contract.contract_digest().to_string(),
                })?
            }
            posture => Ok(ConsumedNativeValue::absent(posture)),
        };
    };
    if native_contract.field_path().native_field_key().is_none() {
        validate_whole_aspect_value(contract, native_contract, row_identity, value)?;
    }
    let actual = native_value_posture(value);
    if actual != native_contract.expected_shape() {
        return Err(
            ProjectionFactExtractionError::NativeContractValueShapeMismatch {
                source_family: contract.source_family(),
                source_identity: compose_scoped_row_source_identity(
                    contract.source_identity(),
                    row_identity,
                )?,
                field_path: native_contract.field_path().clone(),
                contract_key: native_contract.contract().key().clone(),
                contract_revision: native_contract.contract().revision(),
                expected: native_contract.expected_shape(),
                actual,
                projection_authority: contract.contract_digest().to_string(),
            },
        )?;
    }
    Ok(value.clone())
}

fn validate_whole_aspect_value(
    materialized: &MaterializedProjectionContract,
    native: &super::super::super::DeclaredNativeFactContract,
    row_identity: &str,
    value: &ConsumedNativeValue,
) -> Result<(), ProjectionFactExtractionError> {
    let input = match value.view() {
        ConsumedNativeValueView::Scalar(value) => {
            worth_foundational::facade::ContractValidationInput::Scalar(value.clone())
        }
        ConsumedNativeValueView::Struct(value) => {
            worth_foundational::facade::ContractValidationInput::Struct(value.clone())
        }
        ConsumedNativeValueView::Absent(_) => return Ok(()),
    };
    if let Err(denial) =
        worth_foundational::facade::validate_aspect_value(native.contract(), input).into_result()
    {
        return Err(
            ProjectionFactExtractionError::NativeContractValueValidationDenied {
                source_family: materialized.source_family(),
                source_identity: compose_scoped_row_source_identity(
                    materialized.source_identity(),
                    row_identity,
                )?,
                field_path: native.field_path().clone(),
                contract_key: native.contract().key().clone(),
                contract_revision: native.contract().revision(),
                denial,
                projection_authority: materialized.contract_digest().to_string(),
            },
        );
    }
    Ok(())
}

fn native_value_posture(value: &ConsumedNativeValue) -> AspectValuePosture {
    match value.view() {
        ConsumedNativeValueView::Scalar(value) => AspectValuePosture::Scalar(value.value_family()),
        ConsumedNativeValueView::Struct(_) => AspectValuePosture::Struct,
        ConsumedNativeValueView::Absent(posture) => AspectValuePosture::Absent(posture),
    }
}

pub(super) fn missing_declared_field(
    contract: &MaterializedProjectionContract,
    row_identity: &str,
    field_path: &ProjectionFactFieldPath,
    fact_kind: ProjectionFactKind,
) -> ProjectionFactExtractionError {
    (|| -> Result<ProjectionFactExtractionError, IdentityDerivationDenial> {
        Ok(
            ProjectionFactExtractionError::MissingDeclaredFieldEvidence {
                source_family: contract.source_family(),
                source_identity: compose_scoped_row_source_identity(
                    contract.source_identity(),
                    row_identity,
                )?,
                field_key: field_path.terminal_projection_for_boundary().to_string(),
                fact_kind,
            },
        )
    })()
    .unwrap_or_else(ProjectionFactExtractionError::from)
}
