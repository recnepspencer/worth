use crate::routing::canonicalization::digest_string;
use crate::source::{
    AdmittedSourceContract, SourceFailureRecord, SourceMaterializationCounters,
    SourceMaterializationRecord,
};
use std::sync::Arc;

pub(super) struct SourceHarnessCertificationBundle {
    truth_view_digest: Option<Arc<str>>,
    source_contract_digest: Option<Arc<str>>,
    routing_digest: Option<Arc<str>>,
    diagnostics_digest: Arc<str>,
    failure_digest: Option<Arc<str>>,
    replay_digest: Option<Arc<str>>,
    counter_snapshot: SourceHarnessCounterSnapshot,
}

impl SourceHarnessCertificationBundle {
    pub(super) fn materialized(
        contract: &AdmittedSourceContract,
        record: &SourceMaterializationRecord,
        replay_digest: Option<&str>,
    ) -> Self {
        Self {
            truth_view_digest: Some(Arc::from(record.truth_view_digest())),
            source_contract_digest: Some(Arc::from(contract.digest())),
            routing_digest: None,
            diagnostics_digest: Arc::from(materialization_diagnostics_digest(record)),
            failure_digest: None,
            replay_digest: replay_digest.map(Arc::from),
            counter_snapshot: SourceHarnessCounterSnapshot::materialized(
                record.counters(),
                replay_digest.is_some(),
            ),
        }
    }

    pub(super) fn rejected(failure: &SourceFailureRecord) -> Self {
        Self {
            truth_view_digest: None,
            source_contract_digest: None,
            routing_digest: None,
            diagnostics_digest: Arc::from(rejection_diagnostics_digest(failure)),
            failure_digest: Some(Arc::from(failure.digest())),
            replay_digest: None,
            counter_snapshot: SourceHarnessCounterSnapshot::rejected(failure),
        }
    }

    pub(super) fn truth_view_digest(&self) -> Option<&str> {
        self.truth_view_digest.as_deref()
    }

    pub(super) fn source_contract_digest(&self) -> Option<&str> {
        self.source_contract_digest.as_deref()
    }

    pub(super) fn routing_digest(&self) -> Option<&str> {
        self.routing_digest.as_deref()
    }

    pub(super) fn diagnostics_digest(&self) -> &str {
        self.diagnostics_digest.as_ref()
    }

    pub(super) fn failure_digest(&self) -> Option<&str> {
        self.failure_digest.as_deref()
    }

    pub(super) fn replay_digest(&self) -> Option<&str> {
        self.replay_digest.as_deref()
    }

    pub(super) fn counter_snapshot(&self) -> &SourceHarnessCounterSnapshot {
        &self.counter_snapshot
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SourceHarnessCounterSnapshot {
    source_declaration_count: usize,
    source_contract_count: usize,
    source_packet_count: usize,
    source_packet_member_count: usize,
    source_materialization_count: usize,
    source_snapshot_read_count: usize,
    source_historical_read_count: usize,
    source_branch_read_count: usize,
    source_facet_read_count: usize,
    source_capability_rejection_count: usize,
    source_contract_mismatch_count: usize,
    source_adapter_non_native_escape_count: usize,
    source_builder_configuration_conflict_count: usize,
    source_replay_request_count: usize,
    retained_source_record_count: usize,
    retained_failure_record_count: usize,
    retained_source_failure_record_count: usize,
}

impl SourceHarnessCounterSnapshot {
    fn materialized(counters: &SourceMaterializationCounters, replay_requested: bool) -> Self {
        Self {
            source_declaration_count: counters.source_declaration_count(),
            source_contract_count: counters.source_contract_count(),
            source_packet_count: counters.source_packet_count(),
            source_packet_member_count: counters.source_packet_member_count(),
            source_materialization_count: counters.source_materialization_count(),
            source_snapshot_read_count: counters.source_snapshot_read_count(),
            source_historical_read_count: counters.source_historical_read_count(),
            source_branch_read_count: counters.source_branch_read_count(),
            source_facet_read_count: counters.source_facet_read_count(),
            source_capability_rejection_count: counters.source_capability_rejection_count(),
            source_contract_mismatch_count: counters.source_contract_mismatch_count(),
            source_adapter_non_native_escape_count: counters
                .source_adapter_non_native_escape_count(),
            source_builder_configuration_conflict_count: counters
                .source_builder_configuration_conflict_count(),
            source_replay_request_count: usize::from(replay_requested),
            retained_source_record_count: 1,
            retained_failure_record_count: 0,
            retained_source_failure_record_count: 0,
        }
    }

    fn rejected(failure: &SourceFailureRecord) -> Self {
        Self {
            source_declaration_count: 1,
            source_contract_count: 0,
            source_packet_count: 0,
            source_packet_member_count: 0,
            source_materialization_count: 0,
            source_snapshot_read_count: 0,
            source_historical_read_count: 0,
            source_branch_read_count: 0,
            source_facet_read_count: 0,
            source_capability_rejection_count: 0,
            source_contract_mismatch_count: usize::from(
                failure.delivery_error_kind()
                    == crate::error::BridgeDeliveryErrorKind::SourceContractMismatch,
            ),
            source_adapter_non_native_escape_count: 0,
            source_builder_configuration_conflict_count: 0,
            source_replay_request_count: 0,
            retained_source_record_count: 0,
            retained_failure_record_count: 0,
            retained_source_failure_record_count: 1,
        }
    }

    pub(super) fn source_declaration_count(&self) -> usize {
        self.source_declaration_count
    }

    pub(super) fn source_contract_count(&self) -> usize {
        self.source_contract_count
    }

    pub(super) fn source_packet_count(&self) -> usize {
        self.source_packet_count
    }

    pub(super) fn source_packet_member_count(&self) -> usize {
        self.source_packet_member_count
    }

    pub(super) fn source_materialization_count(&self) -> usize {
        self.source_materialization_count
    }

    pub(super) fn source_snapshot_read_count(&self) -> usize {
        self.source_snapshot_read_count
    }

    pub(super) fn source_historical_read_count(&self) -> usize {
        self.source_historical_read_count
    }

    pub(super) fn source_branch_read_count(&self) -> usize {
        self.source_branch_read_count
    }

    pub(super) fn source_facet_read_count(&self) -> usize {
        self.source_facet_read_count
    }

    pub(super) fn source_capability_rejection_count(&self) -> usize {
        self.source_capability_rejection_count
    }

    pub(super) fn source_contract_mismatch_count(&self) -> usize {
        self.source_contract_mismatch_count
    }

    pub(super) fn source_adapter_non_native_escape_count(&self) -> usize {
        self.source_adapter_non_native_escape_count
    }

    pub(super) fn source_builder_configuration_conflict_count(&self) -> usize {
        self.source_builder_configuration_conflict_count
    }

    pub(super) fn source_replay_request_count(&self) -> usize {
        self.source_replay_request_count
    }

    pub(super) fn retained_source_record_count(&self) -> usize {
        self.retained_source_record_count
    }

    pub(super) fn retained_failure_record_count(&self) -> usize {
        self.retained_failure_record_count
    }

    pub(super) fn retained_source_failure_record_count(&self) -> usize {
        self.retained_source_failure_record_count
    }
}

pub(super) fn materialization_diagnostics_digest(record: &SourceMaterializationRecord) -> String {
    digest_string(
        "source-diagnostics-digest",
        &format!(
            "record={}|contract={}|truth-view={}|snapshots={}|paths={}",
            record.record_identity().as_str(),
            record.source_contract_identity(),
            record.truth_view_digest(),
            record
                .snapshot_identities()
                .iter()
                .map(crate::snapshot::TruthSnapshotIdentity::canonical_label)
                .collect::<Vec<_>>()
                .join(","),
            record
                .materialization_paths()
                .iter()
                .map(|path| format!("{path:?}"))
                .collect::<Vec<_>>()
                .join(","),
        ),
    )
    .to_string()
}

pub(super) fn rejection_diagnostics_digest(failure: &SourceFailureRecord) -> String {
    digest_string(
        "source-rejection-diagnostics-digest",
        &format!(
            "failure={}|declaration={}|class={:?}|delivery-kind={:?}",
            failure.failure_identity().as_str(),
            failure.declaration_identity().as_str(),
            failure.failure_class(),
            failure.delivery_error_kind(),
        ),
    )
    .to_string()
}
