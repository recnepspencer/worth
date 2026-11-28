use super::super::certification_bundle::{
    SourceHarnessCertificationBundle, SourceHarnessCounterSnapshot,
};
use super::super::SourceHarnessExecution;
use crate::source::{AdmittedSourceContract, SourceFailureRecord, SourceMaterializationRecord};
use serde_json::json;
use std::collections::BTreeMap;

pub(in crate::harness::adapter::adapter_impl) fn summary_json(
    execution: &SourceHarnessExecution,
) -> serde_json::Value {
    match execution {
        SourceHarnessExecution::Materialize { contract, record } => {
            materialization_summary_json(contract, record, None)
        }
        SourceHarnessExecution::Replay {
            contract,
            record,
            replayed,
        } => materialization_summary_json(contract, record, Some(replayed.digest())),
        SourceHarnessExecution::Rejected { failure } => rejection_summary_json(failure),
    }
}

pub(in crate::harness::adapter::adapter_impl) fn extensions_json(
    execution: &SourceHarnessExecution,
    runtime_bridge: &crate::facade::RuntimeBridge,
) -> BTreeMap<String, serde_json::Value> {
    match execution {
        SourceHarnessExecution::Materialize { contract, record } => BTreeMap::from([
            (
                "bridge_source_certification_bundle".to_string(),
                certification_bundle_json(&SourceHarnessCertificationBundle::materialized(
                    contract, record, None,
                )),
            ),
            source_record_extension(runtime_bridge, record),
        ]),
        SourceHarnessExecution::Replay {
            contract,
            record,
            replayed,
        } => BTreeMap::from([
            (
                "bridge_source_certification_bundle".to_string(),
                certification_bundle_json(&SourceHarnessCertificationBundle::materialized(
                    contract,
                    record,
                    Some(replayed.digest()),
                )),
            ),
            source_record_extension(runtime_bridge, record),
            replay_record_extension(replayed),
        ]),
        SourceHarnessExecution::Rejected { failure } => BTreeMap::from([
            (
                "bridge_source_certification_bundle".to_string(),
                certification_bundle_json(&SourceHarnessCertificationBundle::rejected(failure)),
            ),
            rejection_extension(runtime_bridge, failure),
        ]),
    }
}

fn materialization_summary_json(
    contract: &AdmittedSourceContract,
    record: &SourceMaterializationRecord,
    replay_digest: Option<&str>,
) -> serde_json::Value {
    let certification =
        SourceHarnessCertificationBundle::materialized(contract, record, replay_digest);
    match replay_digest {
        Some(replay_digest) => json!({
            "source_declaration_identity": record.source_declaration_identity(),
            "source_contract_identity": contract.contract_identity().as_str(),
            "source_materialization_record_identity": record.record_identity().as_str(),
            "planned_packet_set_digest": record.planned_packet_set_digest(),
            "materialized_packet_set_digest": record.materialized_packet_set_digest(),
            "truth_view_digest": record.truth_view_digest(),
            "source_contract_digest": contract.digest(),
            "diagnostics_digest": certification.diagnostics_digest(),
            "replay_digest": replay_digest,
            "failure_digest": serde_json::Value::Null,
            "counter_snapshot": counter_snapshot_json(certification.counter_snapshot()),
        }),
        None => json!({
            "source_declaration_identity": record.source_declaration_identity(),
            "source_contract_identity": contract.contract_identity().as_str(),
            "source_materialization_record_identity": record.record_identity().as_str(),
            "planned_packet_set_digest": record.planned_packet_set_digest(),
            "materialized_packet_set_digest": record.materialized_packet_set_digest(),
            "truth_view_digest": record.truth_view_digest(),
            "source_contract_digest": contract.digest(),
            "diagnostics_digest": certification.diagnostics_digest(),
            "failure_digest": serde_json::Value::Null,
            "counter_snapshot": counter_snapshot_json(certification.counter_snapshot()),
        }),
    }
}

fn rejection_summary_json(failure: &SourceFailureRecord) -> serde_json::Value {
    let certification = SourceHarnessCertificationBundle::rejected(failure);
    json!({
        "source_declaration_identity": failure.declaration_identity().as_str(),
        "source_contract_identity": serde_json::Value::Null,
        "source_materialization_record_identity": serde_json::Value::Null,
        "truth_view_digest": serde_json::Value::Null,
        "source_contract_digest": serde_json::Value::Null,
        "diagnostics_digest": certification.diagnostics_digest(),
        "failure_digest": certification.failure_digest(),
        "counter_snapshot": counter_snapshot_json(certification.counter_snapshot()),
    })
}

fn source_record_extension(
    runtime_bridge: &crate::facade::RuntimeBridge,
    record: &SourceMaterializationRecord,
) -> (String, serde_json::Value) {
    let explanation = runtime_bridge
        .diagnostics()
        .explain_source_materialization_record(record);
    let source_contract_digest = runtime_bridge
        .source_registry()
        .contract_for_identity(record.source_contract_identity())
        .map(|contract| contract.digest().to_string());
    (
        "bridge_source_materialization_record".to_string(),
        json!({
            "source_materialization_record_identity": record.record_identity().as_str(),
            "source_contract_identity": record.source_contract_identity(),
            "source_declaration_identity": record.source_declaration_identity(),
            "planned_packet_set_digest": record.planned_packet_set_digest(),
            "materialized_packet_set_digest": record.materialized_packet_set_digest(),
            "truth_view_digest": record.truth_view_digest(),
            "source_contract_digest": source_contract_digest,
            "source_capability_digest": record.source_capability_digest(),
            "adapter_capability_digest": record.adapter_capability_digest(),
            "planned_packet_digests": record.planned_packet_digests(),
            "read_packet_digests": record.read_packets().iter().map(crate::snapshot::SnapshotReadPacket::digest).collect::<Vec<_>>(),
            "authority_basis_digests": record.authority_basis_digests(),
            "snapshot_identities": record.snapshot_identities().iter().map(crate::snapshot::TruthSnapshotIdentity::canonical_label).collect::<Vec<_>>(),
            "materialization_paths": record.materialization_paths().iter().map(|path| format!("{path:?}")).collect::<Vec<_>>(),
            "digest": record.digest(),
            "explanation": {
                "record_identity": explanation.record_identity(),
                "source_contract_identity": explanation.source_contract_identity(),
                "source_declaration_identity": explanation.source_declaration_identity(),
                "truth_view_digest": explanation.truth_view_digest(),
                "planned_packet_set_digest": explanation.planned_packet_set_digest(),
                "materialized_packet_set_digest": explanation.materialized_packet_set_digest(),
                "packet_count": explanation.packet_count(),
                "snapshot_identities": explanation.snapshot_identities().iter().map(crate::snapshot::TruthSnapshotIdentity::canonical_label).collect::<Vec<_>>(),
                "materialization_paths": explanation.materialization_paths().iter().map(|path| format!("{path:?}")).collect::<Vec<_>>(),
            }
        }),
    )
}

fn replay_record_extension(replayed: &SourceMaterializationRecord) -> (String, serde_json::Value) {
    (
        "bridge_source_replay_record".to_string(),
        json!({
            "source_materialization_record_identity": replayed.record_identity().as_str(),
            "source_contract_identity": replayed.source_contract_identity(),
            "source_declaration_identity": replayed.source_declaration_identity(),
            "replay_digest": replayed.digest(),
        }),
    )
}

fn rejection_extension(
    runtime_bridge: &crate::facade::RuntimeBridge,
    failure: &SourceFailureRecord,
) -> (String, serde_json::Value) {
    let explanation = runtime_bridge
        .diagnostics()
        .explain_source_failure_record(failure);
    (
        "bridge_source_rejection".to_string(),
        json!({
            "source_failure_identity": failure.failure_identity().as_str(),
            "source_declaration_identity": failure.declaration_identity().as_str(),
            "selector_identity": failure.selector_identity(),
            "failure_kind": format!("{:?}", failure.delivery_error_kind()),
            "failure_class": format!("{:?}", failure.failure_class()),
            "failure_detail": failure.detail(),
            "source_capability_digest": failure.source_capability_digest(),
            "digest": failure.digest(),
            "explanation": {
                "failure_identity": explanation.failure_identity(),
                "declaration_identity": explanation.declaration_identity(),
                "failure_class": format!("{:?}", explanation.failure_class()),
                "delivery_error_kind": format!("{:?}", explanation.delivery_error_kind()),
            },
        }),
    )
}

fn certification_bundle_json(bundle: &SourceHarnessCertificationBundle) -> serde_json::Value {
    json!({
        "truth_view_digest": bundle.truth_view_digest(),
        "source_contract_digest": bundle.source_contract_digest(),
        "routing_digest": bundle.routing_digest(),
        "diagnostics_digest": bundle.diagnostics_digest(),
        "failure_digest": bundle.failure_digest(),
        "replay_digest": bundle.replay_digest(),
        "counter_snapshot": counter_snapshot_json(bundle.counter_snapshot()),
    })
}

fn counter_snapshot_json(counters: &SourceHarnessCounterSnapshot) -> serde_json::Value {
    json!({
        "source_declaration_count": counters.source_declaration_count(),
        "source_contract_count": counters.source_contract_count(),
        "source_packet_count": counters.source_packet_count(),
        "source_packet_member_count": counters.source_packet_member_count(),
        "source_materialization_count": counters.source_materialization_count(),
        "source_snapshot_read_count": counters.source_snapshot_read_count(),
        "source_historical_read_count": counters.source_historical_read_count(),
        "source_branch_read_count": counters.source_branch_read_count(),
        "source_facet_read_count": counters.source_facet_read_count(),
        "source_capability_rejection_count": counters.source_capability_rejection_count(),
        "source_contract_mismatch_count": counters.source_contract_mismatch_count(),
        "source_adapter_non_native_escape_count": counters.source_adapter_non_native_escape_count(),
        "source_builder_configuration_conflict_count": counters.source_builder_configuration_conflict_count(),
        "source_replay_request_count": counters.source_replay_request_count(),
        "retained_source_record_count": counters.retained_source_record_count(),
        "retained_failure_record_count": counters.retained_failure_record_count(),
        "retained_source_failure_record_count": counters.retained_source_failure_record_count(),
    })
}
