use std::sync::Arc;

use worth_foundational::facade::{AspectKey, ContractValidationDenial, MaskAdmissibilityDenial};

use super::SnapshotReadCorrelationId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeSnapshotReadErrorKind {
    ExternalSnapshotReadFailure,
    SnapshotIdentityMismatch,
    DuplicateRecord,
    RecordCountMismatch,
    MissingRecord,
    ExtraRecord,
    ProjectionMaskRejected,
    AspectContractValidationDenied,
    ExecutionDenied(crate::error::BridgeExecutionDenial),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSnapshotReadError {
    kind: BridgeSnapshotReadErrorKind,
    message: Arc<str>,
    correlation_id: Option<SnapshotReadCorrelationId>,
    aspect_key: Option<AspectKey>,
    mask_denial: Option<MaskAdmissibilityDenial>,
    validation_denial: Option<ContractValidationDenial>,
}

impl BridgeSnapshotReadError {
    pub(crate) fn checkpoint(
        request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Self> {
        request
            .run(worth_execution::ExecutionWorkCeiling::new(0), |_| ())
            .map(|_| ())
            .map_err(Self::execution_scope_denied)
    }

    pub(crate) fn execution_denied(denial: crate::error::BridgeExecutionDenial) -> Self {
        Self {
            kind: BridgeSnapshotReadErrorKind::ExecutionDenied(denial),
            message: Arc::from(denial.label()),
            correlation_id: None,
            aspect_key: None,
            mask_denial: None,
            validation_denial: None,
        }
    }

    pub(crate) fn execution_scope_denied(denial: worth_execution::WorkCeilingDenial) -> Self {
        Self::execution_denied(denial.into())
    }

    pub fn new(message: impl Into<Arc<str>>) -> Self {
        Self {
            kind: BridgeSnapshotReadErrorKind::ExternalSnapshotReadFailure,
            message: message.into(),
            correlation_id: None,
            aspect_key: None,
            mask_denial: None,
            validation_denial: None,
        }
    }

    pub(crate) fn snapshot_identity_mismatch(
        returned_snapshot: &str,
        expected_snapshot: &str,
    ) -> Self {
        Self {
            kind: BridgeSnapshotReadErrorKind::SnapshotIdentityMismatch,
            message: Arc::from(format!(
                "Truth-view observation read returned `{returned_snapshot}` but materialized snapshot authority was `{expected_snapshot}`."
            )),
            correlation_id: None,
            aspect_key: None,
            mask_denial: None,
            validation_denial: None,
        }
    }

    pub(crate) fn duplicate_record(correlation_id: SnapshotReadCorrelationId) -> Self {
        Self {
            kind: BridgeSnapshotReadErrorKind::DuplicateRecord,
            message: Arc::from(format!(
                "Snapshot read result contained duplicate record for correlation `{}`.",
                correlation_id.as_str(),
            )),
            correlation_id: Some(correlation_id),
            aspect_key: None,
            mask_denial: None,
            validation_denial: None,
        }
    }

    pub(crate) fn record_count_mismatch(returned_count: usize, requested_count: usize) -> Self {
        Self {
            kind: BridgeSnapshotReadErrorKind::RecordCountMismatch,
            message: Arc::from(format!(
                "Snapshot read result returned {returned_count} records for {requested_count} requested reads."
            )),
            correlation_id: None,
            aspect_key: None,
            mask_denial: None,
            validation_denial: None,
        }
    }

    pub(crate) fn missing_record(correlation_id: SnapshotReadCorrelationId) -> Self {
        Self {
            kind: BridgeSnapshotReadErrorKind::MissingRecord,
            message: Arc::from(format!(
                "Snapshot read result omitted required correlation `{}`.",
                correlation_id.as_str(),
            )),
            correlation_id: Some(correlation_id),
            aspect_key: None,
            mask_denial: None,
            validation_denial: None,
        }
    }

    pub(crate) fn extra_record(correlation_id: SnapshotReadCorrelationId) -> Self {
        Self {
            kind: BridgeSnapshotReadErrorKind::ExtraRecord,
            message: Arc::from(format!(
                "Snapshot read result returned undeclared correlation `{}`.",
                correlation_id.as_str(),
            )),
            correlation_id: Some(correlation_id),
            aspect_key: None,
            mask_denial: None,
            validation_denial: None,
        }
    }

    pub(crate) fn projection_mask_rejected(
        correlation_id: SnapshotReadCorrelationId,
        aspect_key: AspectKey,
        denial: MaskAdmissibilityDenial,
    ) -> Self {
        Self {
            kind: BridgeSnapshotReadErrorKind::ProjectionMaskRejected,
            message: Arc::from(format!(
                "Snapshot read correlation `{}` carries projection mask rejected by aspect contract `{}`: {denial:?}.",
                correlation_id.as_str(),
                aspect_key.as_str(),
            )),
            correlation_id: Some(correlation_id),
            aspect_key: Some(aspect_key),
            mask_denial: Some(denial),
            validation_denial: None,
        }
    }

    pub(crate) fn aspect_contract_validation_denied(
        correlation_id: SnapshotReadCorrelationId,
        aspect_key: AspectKey,
        denial: ContractValidationDenial,
    ) -> Self {
        Self {
            kind: BridgeSnapshotReadErrorKind::AspectContractValidationDenied,
            message: Arc::from(format!(
                "Snapshot read result for correlation `{}` failed aspect contract `{}` validation: {denial:?}.",
                correlation_id.as_str(),
                aspect_key.as_str(),
            )),
            correlation_id: Some(correlation_id),
            aspect_key: Some(aspect_key),
            mask_denial: None,
            validation_denial: Some(denial),
        }
    }

    pub fn kind(&self) -> BridgeSnapshotReadErrorKind {
        self.kind
    }

    pub fn correlation_id(&self) -> Option<&SnapshotReadCorrelationId> {
        self.correlation_id.as_ref()
    }

    pub fn aspect_key(&self) -> Option<&AspectKey> {
        self.aspect_key.as_ref()
    }

    pub fn mask_denial(&self) -> Option<&MaskAdmissibilityDenial> {
        self.mask_denial.as_ref()
    }

    pub fn validation_denial(&self) -> Option<&ContractValidationDenial> {
        self.validation_denial.as_ref()
    }
}

impl std::fmt::Display for BridgeSnapshotReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BridgeSnapshotReadError {}
