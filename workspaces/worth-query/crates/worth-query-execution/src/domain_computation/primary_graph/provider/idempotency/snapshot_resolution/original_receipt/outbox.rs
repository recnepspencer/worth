//! Persistent outbox meaning excludes process-local inbound installation.
use crate::domain_computation::application_aftermath::{
    ExternalEffectCorrelationIdentity, WorthQueryDispatchOutboxRecord,
};
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_relational::facade::transactions::RecordRef;

#[derive(Debug)]
pub struct WorthQueryRecoveredDispatchOutbox {
    record_ref: RecordRef,
    record: WorthQueryDispatchOutboxRecord,
}

impl WorthQueryRecoveredDispatchOutbox {
    pub(super) fn from_source_row(
        record_ref: RecordRef,
        record: WorthQueryDispatchOutboxRecord,
    ) -> Self {
        Self { record_ref, record }
    }
    pub fn record_ref(&self) -> &RecordRef {
        &self.record_ref
    }
    pub fn correlation(&self) -> &ExternalEffectCorrelationIdentity {
        self.record.correlation()
    }
    pub fn correlation_family(&self) -> &str {
        self.record.correlation_family().as_str()
    }
    pub fn effect(&self) -> &str {
        self.record.effect()
    }
    pub fn protocol_identity(&self) -> &BoundaryProtocolIdentity {
        self.record.protocol_identity()
    }
    pub fn protocol_version(&self) -> BoundaryProtocolVersion {
        self.record.protocol_version()
    }
    pub fn maximum_payload_bytes(&self) -> u64 {
        self.record.maximum_payload_bytes()
    }
    pub fn payload(&self) -> &[u8] {
        self.record.payload()
    }
    pub fn outcome_identity(&self) -> u64 {
        self.record.outcome_identity()
    }
    pub fn operation_slot(&self) -> Option<&str> {
        self.record.operation_slot()
    }
}

impl PartialEq for WorthQueryRecoveredDispatchOutbox {
    fn eq(&self, other: &Self) -> bool {
        self.record_ref == other.record_ref && self.record.same_persisted_fields(&other.record)
    }
}
impl Eq for WorthQueryRecoveredDispatchOutbox {}
