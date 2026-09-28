mod error;
mod grouped;
mod live_binding;
mod query_context;
mod retained_binding;
mod row_materialization;
mod write_receipt;

use super::consumed::{ConsumedProjectionFactSet, ConsumedSourceReferenceFact};
use super::contracts::MaterializedProjectionContract;
use super::source::ProjectionSourceFamily;
use super::source::ProjectionSourceReferenceIdentity;
use crate::query_context::QueryContextExecutionArtifact;
use crate::runtime::{
    WorthQueryDerivedArtifactBinding, WorthQueryLiveArtifactBinding, WorthQueryLiveReadResult,
    WorthQueryReadResult, WorthQueryWriteReceipt,
};
use worth_runtime_bridge::facade::{
    BridgeGroupedTruthViewArtifact, BridgeMaterializedRowSetArtifact,
};
use worth_runtime_bridge::facade::{
    RelationalAuthoritativeRowSetArtifact, RelationalGroupedProjectionArtifact,
};

pub use error::ProjectionFactExtractionError;

impl MaterializedProjectionContract {
    pub fn extract_from_read_result(
        &self,
        result: &WorthQueryReadResult,
    ) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
        ensure_optional_metadata(
            "query_digest",
            self.query_digest(),
            Some(result.receipt().canonical_query_digest()),
        )?;
        ensure_optional_metadata(
            "basis_digest",
            self.basis_digest(),
            Some(result.receipt().basis_digest()),
        )?;
        ensure_optional_metadata(
            "result_digest",
            self.result_digest(),
            Some(result.receipt().result_digest()),
        )?;
        row_materialization::extract_read_result_facts(self, result)
    }

    pub fn extract_from_live_read_result(
        &self,
        result: &WorthQueryLiveReadResult,
    ) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
        ensure_optional_metadata(
            "query_digest",
            self.query_digest(),
            Some(result.receipt().canonical_query_digest()),
        )?;
        ensure_optional_metadata(
            "basis_digest",
            self.basis_digest(),
            Some(result.receipt().snapshot_evidence_identity().as_str()),
        )?;
        ensure_optional_metadata(
            "result_digest",
            self.result_digest(),
            Some(result.receipt().result_digest()),
        )?;
        ensure_optional_metadata(
            "result_shape_digest",
            Some(self.canonical_result_shape_digest()),
            Some(result.receipt().view_shape_digest()),
        )?;
        row_materialization::extract_live_read_result_facts(self, result)
    }

    pub fn extract_from_relational_row_set(
        &self,
        row_set: &RelationalAuthoritativeRowSetArtifact,
    ) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
        row_materialization::extract_relational_row_set_facts(self, row_set)
    }

    pub fn extract_from_bridge_truth_view_row_set(
        &self,
        row_set: &BridgeMaterializedRowSetArtifact,
    ) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
        row_materialization::extract_bridge_row_set_facts(self, row_set)
    }

    pub fn extract_from_relational_grouped_projection(
        &self,
        projection: &RelationalGroupedProjectionArtifact,
    ) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
        grouped::extract_relational_grouped_facts(self, projection)
    }

    pub fn extract_from_bridge_grouped_truth_view(
        &self,
        grouped_truth_view: &BridgeGroupedTruthViewArtifact,
    ) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
        grouped::extract_bridge_grouped_facts(self, grouped_truth_view)
    }

    pub fn extract_from_query_context_execution(
        &self,
        execution: &QueryContextExecutionArtifact,
    ) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
        ensure_optional_metadata(
            "query_digest",
            self.query_digest(),
            Some(execution.query_digest()),
        )?;
        ensure_optional_metadata(
            "basis_digest",
            self.basis_digest(),
            Some(execution.basis_digest()),
        )?;
        ensure_optional_metadata(
            "result_digest",
            self.result_digest(),
            Some(execution.result_digest()),
        )?;
        ensure_optional_metadata(
            "result_shape_digest",
            Some(self.canonical_result_shape_digest()),
            Some(execution.result_shape_digest()),
        )?;
        query_context::extract_query_context_facts(self, execution)
    }

    pub fn extract_from_retained_derived_artifact_binding(
        &self,
        binding: &WorthQueryDerivedArtifactBinding,
    ) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
        retained_binding::extract_retained_binding_facts(self, binding)
    }

    pub fn extract_from_live_artifact_binding(
        &self,
        binding: &WorthQueryLiveArtifactBinding,
    ) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
        live_binding::extract_live_binding_facts(self, binding)
    }

    pub fn extract_from_write_receipt(
        &self,
        receipt: &WorthQueryWriteReceipt,
    ) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
        write_receipt::extract_write_receipt_facts(self, receipt)
    }
}

fn ensure_optional_metadata(
    metadata_label: &'static str,
    contract_value: Option<&str>,
    provided_value: Option<&str>,
) -> Result<(), ProjectionFactExtractionError> {
    match (contract_value, provided_value) {
        (Some(contract_value), Some(provided_value)) if contract_value == provided_value => Ok(()),
        (Some(contract_value), Some(provided_value)) => Err(
            ProjectionFactExtractionError::SourceArtifactMetadataMismatch {
                metadata_label,
                contract_value: contract_value.to_string(),
                provided_value: provided_value.to_string(),
            },
        ),
        (None, None) => Ok(()),
        (Some(contract_value), None) => Err(
            ProjectionFactExtractionError::SourceArtifactMetadataMismatch {
                metadata_label,
                contract_value: contract_value.to_string(),
                provided_value: "<missing>".to_string(),
            },
        ),
        (None, Some(provided_value)) => Err(
            ProjectionFactExtractionError::SourceArtifactMetadataMismatch {
                metadata_label,
                contract_value: "<absent>".to_string(),
                provided_value: provided_value.to_string(),
            },
        ),
    }
}

pub(super) fn ensure_contract_family(
    contract: &MaterializedProjectionContract,
    extractor_family: ProjectionSourceFamily,
) -> Result<(), ProjectionFactExtractionError> {
    if contract.source_family() == extractor_family {
        Ok(())
    } else {
        Err(
            ProjectionFactExtractionError::ContractSourceFamilyMismatch {
                contract_family: contract.source_family(),
                extractor_family,
            },
        )
    }
}

pub(super) fn ensure_source_identity(
    contract_source_identity: &str,
    provided_source_identity: &str,
) -> Result<(), ProjectionFactExtractionError> {
    if contract_source_identity == provided_source_identity {
        Ok(())
    } else {
        Err(ProjectionFactExtractionError::SourceIdentityMismatch {
            contract_source_identity: contract_source_identity.to_string(),
            provided_source_identity: provided_source_identity.to_string(),
        })
    }
}

pub(super) fn source_reference_inventory_matches(
    expected: &[ProjectionSourceReferenceIdentity],
    actual: &[ConsumedSourceReferenceFact],
) -> bool {
    expected.len() == actual.len()
        && expected
            .iter()
            .zip(actual.iter())
            .all(|(expected, actual)| {
                expected.label() == actual.label() && expected.identity() == actual.identity()
            })
}
