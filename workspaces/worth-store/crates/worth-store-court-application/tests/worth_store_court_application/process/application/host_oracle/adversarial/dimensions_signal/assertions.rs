//! Phase classification, reopen parity, and durable-occurrence receipt
//! binding assertions for C4b.3 Signal+Bridge matrices.
use super::super::ReopenPhase;
use super::wire::{SignalReply, WireReceipt, WireSnapshot};

#[derive(Clone)]
pub(super) struct SignalRef {
    pub(super) before: WireSnapshot,
    pub(super) after: WireSnapshot,
    pub(super) target_key_identity: [u8; 32],
    pub(super) definition_identity: [u8; 32],
}

pub(super) fn classify_signal_phase(
    opened: &SignalReply,
    expected_before: &WireSnapshot,
) -> ReopenPhase {
    if !opened.custody.is_empty() {
        ReopenPhase::Custody
    } else if opened.snapshot.as_ref().unwrap().intent == expected_before.intent {
        ReopenPhase::PreOp
    } else {
        ReopenPhase::Committed
    }
}

pub(super) fn assert_reopen_phase_snapshot(
    opened: &SignalReply,
    phase: ReopenPhase,
    sig_ref: &SignalRef,
    label: &str,
) {
    let state = opened.snapshot.as_ref().expect("reopen snapshot");
    assert_eq!(
        (
            state.installed_conditional_definition_count,
            state.installed_conditional_target_ref_count,
        ),
        (1, 1),
        "{label}: cold reopen reconstitutes exact 1 active Bridge definition"
    );
    match phase {
        ReopenPhase::PreOp => {
            assert!(opened.custody.is_empty(), "{label}: PreOp 0 custody");
            assert!(
                state.same_durable_semantics(&sig_ref.before),
                "{label}: PreOp durable semantics parity ({state:?} vs {:?})",
                sig_ref.before
            );
            assert_eq!(
                state.world_commit_ordinal,
                sig_ref.before.world_commit_ordinal
            );
        }
        ReopenPhase::Custody => {
            assert_eq!(opened.custody.len(), 1, "{label}: Custody 1 row");
            assert!(
                state.same_durable_semantics(&sig_ref.before),
                "{label}: Custody predecessor parity"
            );
            assert_eq!(
                state.world_commit_ordinal,
                sig_ref.before.world_commit_ordinal
            );
            assert_eq!(
                opened.custody[0].expected_commit, sig_ref.before.world_commit_ordinal,
                "{label}: custody expected commit"
            );
            assert!(
                opened.custody[0].native_commit_id.is_some(),
                "{label}: custody must carry original Native commit identity"
            );
        }
        ReopenPhase::Committed => {
            assert!(opened.custody.is_empty(), "{label}: Committed 0 custody");
            assert!(
                state.same_durable_semantics(&sig_ref.after),
                "{label}: Committed durable semantics & Bridge counts parity"
            );
            assert_eq!(
                state.world_commit_ordinal,
                sig_ref.before.world_commit_ordinal + 1,
                "{label}: Committed World ordinal"
            );
            assert!(
                state.head_attempt_ordinal.is_some()
                    && state.head_native_commit_id.is_some()
                    && state.head_native_version_id.is_some(),
                "{label}: Committed World occurrence must record attempt and Native commit"
            );
        }
    }
}

pub(super) fn assert_receipt_bound_to_occurrence(
    rec: &WireReceipt,
    snap: &WireSnapshot,
    sig_ref: &SignalRef,
    label: &str,
) {
    assert_eq!(
        rec.idempotency_key, sig_ref.target_key_identity,
        "{label}: receipt idempotency key must match admitted target key"
    );
    assert_eq!(
        Some(rec.attempt_ordinal),
        snap.head_attempt_ordinal,
        "{label}: receipt attempt must bind to durable World occurrence"
    );
    assert_eq!(
        Some(rec.commit_id),
        snap.head_native_commit_id,
        "{label}: receipt Native commit must bind to durable World occurrence"
    );
    assert_eq!(
        Some(rec.version_id),
        snap.head_native_version_id,
        "{label}: receipt Native version must bind to durable World occurrence"
    );
    assert_eq!(
        rec.composite_commit, snap.world_commit_ordinal,
        "{label}: receipt composite commit"
    );
    assert_eq!(
        rec.product_generation, snap.world_reference_generation,
        "{label}: receipt product generation"
    );
    assert_eq!(
        rec.truth_version, sig_ref.after.truth_version,
        "{label}: receipt truth version"
    );
}
