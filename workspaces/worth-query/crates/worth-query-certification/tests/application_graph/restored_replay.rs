//! An idempotency key committed before a capture keeps its intent after the
//! host restores from that capture.
//!
//! The durable intent binds the operation's definition and the admitted
//! principal and scope, never the runtime that admitted the request, so the
//! unchanged retry after restore is not intent drift and a changed retry under
//! the same key still is. The restored runtime does not retain the original
//! receipt: that evidence is process memory. The unchanged retry therefore
//! answers that the key's commit took effect, names that commit, and commits
//! nothing a second time.

#[path = "restored_replay/cross_process.rs"]
mod cross_process;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
};
use worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster;
use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenialKind;
use worth_relational::facade::history::CommitId;

use crate::document_retention_model::host::{
    publish_on_first_program, restore_on_first_program, DocumentRetentionRuntime, SEED_RETENTION,
};
use crate::document_retention_model::presented_request::set_retention;
use crate::document_retention_model::programs::{validated_second_program, RetentionProgramP0};
use crate::document_retention_model::readback::read_retention;

const KEY: u64 = 0x2701_0201;
const RETENTION: u64 = SEED_RETENTION + 1;

#[test]
fn a_key_committed_before_capture_keeps_its_intent_after_restore() {
    let host = publish_on_first_program();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: original, ..
    } = set_retention(&host, host.current_world(), RETENTION, KEY)
        .expect("the first request settles")
    else {
        panic!("the first request commits");
    };
    let WorthQueryApplicationMutationOutcome::AlreadyCommitted(before) =
        set_retention(&host, host.current_world(), RETENTION, KEY)
            .expect("the retry before capture settles")
    else {
        panic!("the retry before capture replays");
    };
    assert!(before.is_same_authoritative_commit(&original));
    let commit = original.committed_changes().commit_reference().commit_id;
    let checkpoint = host
        .runtime()
        .capture_application_checkpoint()
        .expect("the host captures");
    drop(host);

    let restored = restore_on_first_program(
        checkpoint,
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .expect("the host restores under the roster it installed");
    assert_the_key_resolves_by_its_durable_record(&restored, commit);
}

/// The unchanged retry names the key's original commit and commits nothing;
/// a changed retry under the same key is drift.
fn assert_the_key_resolves_by_its_durable_record(
    restored: &DocumentRetentionRuntime<RetentionProgramP0>,
    commit: CommitId,
) {
    let branch = restored.current_world();
    let ledger = || {
        restored
            .granular_invalidation_installation()
            .retain_primary_graph_integration_handle()
            .with_runtime(|runtime| runtime.history().latest_patch_stream_position())
    };
    let restored_head = ledger();
    let unchanged = set_retention(restored, branch, RETENTION, KEY);
    let Err(WorthQueryApplicationRequestMutationDenial::Idempotency(denial)) = &unchanged else {
        panic!("the unchanged retry after restore is not drift and commits nothing: {unchanged:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationIdempotencyResolutionDenialKind::CommittedReceiptNotRetained {
            commit
        },
        "the restored runtime names the key's commit, whose receipt it did not retain"
    );
    assert_eq!(
        ledger(),
        restored_head,
        "the unchanged retry commits nothing"
    );

    let drifted = set_retention(restored, branch, RETENTION + 1, KEY)
        .expect("the drifted retry after restore settles");
    assert!(
        matches!(
            drifted,
            WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
        ),
        "a changed request under the key is still drift after restore: {drifted:?}"
    );
    assert_eq!(read_retention(restored.runtime(), branch), RETENTION);
}
