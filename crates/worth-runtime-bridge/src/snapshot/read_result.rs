use worth_foundational::facade::{
    AspectValue, ContractValidatedAspectArtifact, ContractValidatedAspectValueView,
    ContractValidationInput, StructAspectValue,
};

use super::SnapshotReadCorrelationId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotReadValue {
    Scalar(AspectValue),
    Struct(StructAspectValue),
}

impl SnapshotReadValue {
    pub fn scalar_value(&self) -> Option<&AspectValue> {
        match self {
            Self::Scalar(value) => Some(value),
            Self::Struct(_) => None,
        }
    }

    pub(crate) fn into_validation_input(self) -> ContractValidationInput {
        match self {
            Self::Scalar(value) => value.into(),
            Self::Struct(value) => value.into(),
        }
    }
}

impl From<AspectValue> for SnapshotReadValue {
    fn from(value: AspectValue) -> Self {
        Self::Scalar(value)
    }
}

impl From<StructAspectValue> for SnapshotReadValue {
    fn from(value: StructAspectValue) -> Self {
        Self::Struct(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotReadRecord {
    correlation_id: SnapshotReadCorrelationId,
    read_value: Option<SnapshotReadValue>,
}

impl SnapshotReadRecord {
    pub fn for_request(
        request: &super::SnapshotReadRequest,
        read_value: impl Into<SnapshotReadValue>,
    ) -> Self {
        Self {
            correlation_id: request.correlation_id().clone(),
            read_value: Some(read_value.into()),
        }
    }

    /// Retains an authoritative statement that the requested aspect is absent
    /// at the bound snapshot. Absence is a value posture, not an omitted row.
    pub fn absent_for_request(request: &super::SnapshotReadRequest) -> Self {
        Self {
            correlation_id: request.correlation_id().clone(),
            read_value: None,
        }
    }

    pub fn correlation_id(&self) -> &SnapshotReadCorrelationId {
        &self.correlation_id
    }

    pub fn read_value_posture(&self) -> Option<&SnapshotReadValue> {
        self.read_value.as_ref()
    }

    pub fn scalar_aspect_value(&self) -> Option<&AspectValue> {
        self.read_value
            .as_ref()
            .and_then(SnapshotReadValue::scalar_value)
    }

    pub fn is_absent(&self) -> bool {
        self.read_value.is_none()
    }
}

#[derive(Debug, Clone)]
pub struct SnapshotReadPacketResult {
    snapshot_identity: super::TruthSnapshotIdentity,
    records: std::sync::Arc<Vec<SnapshotReadRecord>>,
    memory: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
}

impl SnapshotReadPacketResult {
    pub fn new(
        snapshot_identity: super::TruthSnapshotIdentity,
        records: Vec<SnapshotReadRecord>,
    ) -> Self {
        Self {
            snapshot_identity,
            records: std::sync::Arc::new(records),
            memory: None,
        }
    }

    pub(crate) fn with_memory(
        mut self,
        memory: worth_execution::ExecutionMemoryReservation,
    ) -> Self {
        self.memory = Some(std::sync::Arc::new(memory));
        self
    }

    pub fn snapshot_identity(&self) -> &super::TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn records(&self) -> &[SnapshotReadRecord] {
        &self.records
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        super::TruthSnapshotIdentity,
        std::sync::Arc<Vec<SnapshotReadRecord>>,
        Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
    ) {
        (self.snapshot_identity, self.records, self.memory)
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedSnapshotReadPacketResult {
    snapshot_identity: super::TruthSnapshotIdentity,
    records: std::sync::Arc<Vec<ValidatedSnapshotReadRecord>>,
    _memory: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
}

impl ValidatedSnapshotReadPacketResult {
    pub(crate) fn validated(
        snapshot_identity: super::TruthSnapshotIdentity,
        records: Vec<ValidatedSnapshotReadRecord>,
        memory: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
    ) -> Self {
        Self {
            snapshot_identity,
            records: std::sync::Arc::new(records),
            _memory: memory,
        }
    }

    pub fn snapshot_identity(&self) -> &super::TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn records(&self) -> &[ValidatedSnapshotReadRecord] {
        &self.records
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedSnapshotReadRecord {
    correlation_id: SnapshotReadCorrelationId,
    validated_value: Option<ContractValidatedAspectArtifact>,
}

impl ValidatedSnapshotReadRecord {
    pub(crate) fn new(
        correlation_id: SnapshotReadCorrelationId,
        validated_value: ContractValidatedAspectArtifact,
    ) -> Self {
        Self {
            correlation_id,
            validated_value: Some(validated_value),
        }
    }

    pub(crate) fn absent(correlation_id: SnapshotReadCorrelationId) -> Self {
        Self {
            correlation_id,
            validated_value: None,
        }
    }

    pub fn correlation_id(&self) -> &SnapshotReadCorrelationId {
        &self.correlation_id
    }

    pub fn validated_value_posture(&self) -> Option<&ContractValidatedAspectArtifact> {
        self.validated_value.as_ref()
    }

    pub fn scalar_aspect_value(&self) -> Option<&AspectValue> {
        self.validated_value
            .as_ref()
            .and_then(contract_validated_scalar_aspect_value)
    }

    pub fn is_absent(&self) -> bool {
        self.validated_value.is_none()
    }
}

pub(crate) fn contract_validated_scalar_aspect_value(
    validated_value: &ContractValidatedAspectArtifact,
) -> Option<&AspectValue> {
    match validated_value.payload().view() {
        ContractValidatedAspectValueView::Scalar(value) => Some(value),
        ContractValidatedAspectValueView::Struct(_) => None,
    }
}

impl PartialEq for SnapshotReadPacketResult {
    fn eq(&self, other: &Self) -> bool {
        self.snapshot_identity == other.snapshot_identity && self.records == other.records
    }
}
impl Eq for SnapshotReadPacketResult {}
impl PartialEq for ValidatedSnapshotReadPacketResult {
    fn eq(&self, other: &Self) -> bool {
        self.snapshot_identity == other.snapshot_identity && self.records == other.records
    }
}
impl Eq for ValidatedSnapshotReadPacketResult {}
