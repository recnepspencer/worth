//! Full durable field projection, excluding every live-only observation.
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryRecoveredApplicationCommitReceipt,
};

pub(super) fn projection(receipt: &WorthQueryRecoveredApplicationCommitReceipt) -> String {
    let outbox = receipt.dispatch_outbox().map(|row| serde_json::json!({
        "record": format!("{:?}", row.record_ref()), "correlation": format!("{:?}", row.correlation()),
        "family": row.correlation_family(), "effect": row.effect(),
        "protocol": format!("{:?}", row.protocol_identity()), "version": format!("{:?}", row.protocol_version()),
        "maximum_payload_bytes": row.maximum_payload_bytes(), "payload": row.payload(),
        "outcome": row.outcome_identity(), "operation_slot": row.operation_slot(),
    }));
    let causality = receipt.aftermath_causality().map(|fact| serde_json::json!({
        "role": format!("{:?}", fact.role()), "parent": fact.parent(), "child": fact.child(),
        "record": format!("{:?}", fact.fact_record()), "source": fact.source(), "root": fact.root(),
        "selected": fact.selected_alternative(), "active": fact.active_reversal(), "prior": fact.prior_transition(),
    }));
    serde_json::to_string_pretty(&serde_json::json!({
        "native": {
            "receipt": receipt.commit_reference(), "position": receipt.native_journal_position(), "digest": receipt.native_artifact_digest(),
            "semantic": receipt.semantic_result().semantic(), "changed_record_count": receipt.changed_record_count(),
            "before_component": receipt.previous_recorded_component(), "after_component": receipt.recorded_component(),
        },
        "binding": {
            "key": receipt.idempotency_key_identity(), "intent": receipt.recorded_intent(), "affinity": receipt.recorded_affinity(),
            "outcome": receipt.outcome_identity().get(), "emitted": receipt.emitted_effect_count(),
            "settlement": format!("{:?}", receipt.exact_output_settlement()),
        },
        "world": {
            "old_head": format!("{:?}", receipt.old_product_head()), "new_head": format!("{:?}", receipt.new_product_head()),
            "attempt": format!("{:?}", receipt.attempt_identity()), "position": receipt.world_journal_position(),
            "digest": receipt.world_artifact_digest(), "publisher": format!("{:?}", receipt.publishing_incarnation()),
        },
        "outbox": outbox, "causality": causality,
    })).unwrap()
}

/// Check overlapping fields against the independently produced live receipt.
/// Native's complete live/source semantic equality has its own filesystem court.
pub(super) fn assert_live_agrees(
    live: &WorthQueryApplicationCommitReceipt,
    recorded: &WorthQueryRecoveredApplicationCommitReceipt,
) {
    assert_eq!(live.commit_reference(), recorded.commit_reference());
    assert_eq!(live.outcome_identity(), Some(recorded.outcome_identity()));
    assert_eq!(live.changed_record_count(), recorded.changed_record_count());
    assert_eq!(live.emitted_effect_count(), recorded.emitted_effect_count());
    assert_eq!(
        live.idempotency_binding().key_identity(),
        recorded.idempotency_key_identity()
    );
    let publication = live.committed_product_publication();
    let head = recorded
        .new_product_head()
        .durable()
        .expect("a filesystem receipt carries the actual Home");
    assert_eq!(
        publication.product_branch().owner_identity().durable(),
        Some(head.branch().owner())
    );
    assert_eq!(
        publication.product_branch().name().as_str(),
        head.branch().name()
    );
    assert_eq!(
        publication.product_incarnation().ordinal(),
        head.incarnation().ordinal()
    );
    assert_eq!(
        publication.product_generation().get(),
        head.generation().get()
    );
    assert_eq!(
        publication.composite_commit().ordinal(),
        head.selected_commit().ordinal()
    );
    assert_eq!(
        publication.publication_attempt().ordinal(),
        recorded.attempt_identity().ordinal()
    );
    assert_eq!(
        recorded.attempt_identity().owner(),
        worth_query_host::facade::primary_graph::WorldOwnerCoordinate::Home {
            owner: publication
                .publication_attempt()
                .owner_identity()
                .durable()
                .unwrap()
        }
    );
    assert_eq!(publication.relational_commit(), recorded.commit_reference());
    let descriptor = |basis: &worth_relational::facade::branch::RelationalBranchBasisDescriptor| {
        (
            basis.descriptor_version(),
            basis.branch_id().clone(),
            basis.reference().generation(),
            basis.truth_version(),
            basis.schema_commitment().clone(),
            basis.visibility_commitment().clone(),
            basis.posture(),
        )
    };
    assert_eq!(
        descriptor(live.basis_descriptor()),
        descriptor(recorded.basis_descriptor())
    );
    assert!(live.dispatch_outbox().is_none());
    assert!(recorded.dispatch_outbox().is_none());
    assert!(live.aftermath_causality().is_none());
    assert!(recorded.aftermath_causality().is_none());
    assert_eq!(
        live.recorded_output_settlement(),
        recorded.exact_output_settlement()
    );
}
