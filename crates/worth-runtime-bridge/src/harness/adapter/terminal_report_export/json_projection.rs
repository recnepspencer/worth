use serde_json::json;

pub(in crate::harness::adapter) fn route_record_json(
    record: crate::diagnostics::BridgeRouteRecord,
) -> serde_json::Value {
    json!({
        "route_identity": record.route_identity().as_str(),
        "invalidation_identity": record.invalidation_identity().as_str(),
        "source_commit": record.source_commit().canonical_label(),
        "source_patch": record.source_patch().canonical_label(),
        "source_snapshot": record.source_snapshot().canonical_label(),
        "source_digest": record.source_digest().as_str(),
        "route_planning_policy_digest": record.route_planning_policy_digest(),
        "mapping_context_digest": record.mapping_context().digest(),
        "planning_provenance_digest": record.planning_provenance_digest(),
        "planning_summary_digest": record.planning_summary_digest(),
        "lowering_provenance_digest": record.lowering_provenance_digest(),
        "lowering_summary_digest": record.lowering_summary_digest(),
        "subscription_slice_identity": record.subscription_slice_identity().as_str(),
        "entries": record.entries().iter().map(|entry| {
            json!({
                "entity_identity_diagnostic_label": entry.entity_identity().diagnostic_label(),
                "aspect_key": entry.aspect_key().as_str(),
                "target_canonical_basis": entry.target_canonical_basis(),
                "mapping_id": entry.mapping_id().as_str(),
                "signal_scope": entry.signal_scope(),
                "routing_mode": format!("{:?}", entry.routing_mode()),
                "widening_class": entry.widening_class().map(|class| format!("{class:?}")),
                "truth_surface_kind": format!("{:?}", entry.truth_surface_kind()),
                "fine_grained_match_status": format!("{:?}", entry.fine_grained_match_status()),
                "aspect_registration_id": entry.aspect_registration_id().map(|id| id.as_str()),
                "subscription_slice_kind": entry.subscription_slice_kind().map(|kind| format!("{kind:?}")),
                "slice_widening_policy": entry.slice_widening_policy().map(|policy| format!("{policy:?}")),
            })
        }).collect::<Vec<_>>(),
        "subscription_slices": record.subscription_slices().iter().map(|slice| {
            json!({
                "entity_identity": slice.entity_identity(),
                "aspect_key": slice.aspect_key().as_str(),
                "target_canonical_basis": slice.native_target_basis(),
                "slice_kind": format!("{:?}", slice.slice_kind()),
                "match_status": format!("{:?}", slice.match_status()),
            })
        }).collect::<Vec<_>>(),
        "invalidation_targets": record.invalidation_targets().iter().map(|target| {
            json!({
                "signal_scope": target.signal_scope(),
                "routing_mode": format!("{:?}", target.routing_mode()),
            })
        }).collect::<Vec<_>>(),
        "counters": {
            "patch_item_count": record.counters().patch_item_count(),
            "normalized_patch_item_count": record.counters().normalized_patch_item_count(),
            "truth_delta_surface_count": record.counters().truth_delta_surface_count(),
            "normalized_truth_delta_surface_count": record.counters().normalized_truth_delta_surface_count(),
            "planned_slice_match_count": record.counters().planned_slice_match_count(),
            "slice_widening_count": record.counters().slice_widening_count(),
            "slice_suppression_count": record.counters().slice_suppression_count(),
            "routing_entry_count": record.counters().routing_entry_count(),
            "invalidation_target_count": record.counters().invalidation_target_count(),
            "mapping_lookup_count": record.counters().mapping_lookup_count(),
            "mapping_widening_count": record.counters().mapping_widening_count(),
            "snapshot_read_count": record.counters().snapshot_read_count(),
            "snapshot_read_packet_count": record.counters().snapshot_read_packet_count(),
            "snapshot_identity_mismatch_count": record.counters().snapshot_identity_mismatch_count(),
            "route_replay_mismatch_count": record.counters().route_replay_mismatch_count(),
        }
    })
}

pub(in crate::harness::adapter) fn historical_replay_summary_json(
    replay_record: &crate::facade::BridgeHistoricalEvaluationReplaySummary,
) -> serde_json::Value {
    json!({
        "historical_record_identity": replay_record.record_identity().as_str(),
        "decision_log_identity": replay_record.decision_log_identity().as_str(),
        "source_snapshot": replay_record.snapshot_identity().canonical_label(),
    })
}

pub(in crate::harness::adapter) fn route_replay_summary_json(
    replay_record: &crate::facade::BridgeReplayRecord,
) -> serde_json::Value {
    json!({
        "route_identity": replay_record.route_identity().as_str(),
        "invalidation_identity": replay_record.invalidation_identity().as_str(),
        "subscription_slice_identity": replay_record.subscription_slice_identity().as_str(),
        "source_commit": replay_record.source_commit().canonical_label(),
        "source_patch": replay_record.source_patch().canonical_label(),
        "source_snapshot": replay_record.source_snapshot().canonical_label(),
    })
}

pub(in crate::harness::adapter) fn diagnostics_summary_json(
    runtime_bridge: &crate::facade::RuntimeBridge,
) -> serde_json::Value {
    json!({
        "tier": format!("{:?}", runtime_bridge.diagnostics().tier()),
        "record_count": runtime_bridge.diagnostics().route_records().len(),
        "source_materialization_record_count": runtime_bridge
            .diagnostics()
            .source_materialization_records()
            .len(),
        "source_failure_record_count": runtime_bridge
            .diagnostics()
            .source_failure_records()
            .len(),
        "route_records": runtime_bridge
            .diagnostics()
            .route_records()
            .into_iter()
            .map(route_record_json)
            .collect::<Vec<_>>(),
    })
}
