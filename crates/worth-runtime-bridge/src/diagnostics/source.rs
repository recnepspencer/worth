use crate::snapshot::TruthSnapshotIdentity;
use crate::source::{SourceFailureRecord, SourceMaterializationRecord};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSourceMaterializationExplanation {
    record_identity: String,
    source_contract_identity: String,
    source_declaration_identity: String,
    truth_view_digest: String,
    planned_packet_set_digest: String,
    materialized_packet_set_digest: String,
    packet_count: usize,
    snapshot_identities: Vec<TruthSnapshotIdentity>,
    materialization_paths: Vec<crate::diagnostics::BridgeHistoricalMaterializationPath>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSourceFailureExplanation {
    failure_identity: String,
    declaration_identity: String,
    failure_class: crate::source::SourceFailureClass,
    delivery_error_kind: crate::error::BridgeDeliveryErrorKind,
}

impl BridgeSourceFailureExplanation {
    pub fn from_record(record: &SourceFailureRecord) -> Self {
        Self {
            failure_identity: record.failure_identity().as_str().to_owned(),
            declaration_identity: record.declaration_identity().as_str().to_owned(),
            failure_class: record.failure_class(),
            delivery_error_kind: record.delivery_error_kind(),
        }
    }

    pub fn failure_identity(&self) -> &str {
        &self.failure_identity
    }

    pub fn declaration_identity(&self) -> &str {
        &self.declaration_identity
    }

    pub fn failure_class(&self) -> crate::source::SourceFailureClass {
        self.failure_class
    }

    pub fn delivery_error_kind(&self) -> crate::error::BridgeDeliveryErrorKind {
        self.delivery_error_kind.clone()
    }
}

impl BridgeSourceMaterializationExplanation {
    pub fn from_record(record: &SourceMaterializationRecord) -> Self {
        Self {
            record_identity: record.record_identity().as_str().to_owned(),
            source_contract_identity: record.source_contract_identity().to_owned(),
            source_declaration_identity: record.source_declaration_identity().to_owned(),
            truth_view_digest: record.truth_view_digest().to_owned(),
            planned_packet_set_digest: record.planned_packet_set_digest().to_owned(),
            materialized_packet_set_digest: record.materialized_packet_set_digest().to_owned(),
            packet_count: record.counters().source_packet_count(),
            snapshot_identities: record.snapshot_identities().to_vec(),
            materialization_paths: record.materialization_paths().to_vec(),
        }
    }

    pub fn record_identity(&self) -> &str {
        &self.record_identity
    }

    pub fn source_contract_identity(&self) -> &str {
        &self.source_contract_identity
    }

    pub fn source_declaration_identity(&self) -> &str {
        &self.source_declaration_identity
    }

    pub fn truth_view_digest(&self) -> &str {
        &self.truth_view_digest
    }

    pub fn planned_packet_set_digest(&self) -> &str {
        &self.planned_packet_set_digest
    }

    pub fn materialized_packet_set_digest(&self) -> &str {
        &self.materialized_packet_set_digest
    }

    pub fn packet_count(&self) -> usize {
        self.packet_count
    }

    pub fn snapshot_identities(&self) -> &[TruthSnapshotIdentity] {
        &self.snapshot_identities
    }

    pub fn materialization_paths(
        &self,
    ) -> &[crate::diagnostics::BridgeHistoricalMaterializationPath] {
        &self.materialization_paths
    }
}
