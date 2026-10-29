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

pub use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
#[path = "restored_replay/cross_process.rs"]
pub mod cross_process;

pub use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryApplicationRequestMutationDenial,
};
pub use worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster;
pub use worth_relational::facade::history::CommitId;

pub use crate::facade::certification::document_retention_model::host::{
    publish_on_first_program, restore_on_first_program, DocumentRetentionRuntime, SEED_RETENTION,
};
pub use crate::facade::certification::document_retention_model::operator_identity::authenticate_operator;
pub use crate::facade::certification::document_retention_model::presented_request::set_retention;
pub use crate::facade::certification::document_retention_model::programs::{
    validated_second_program, RetentionProgramP0,
};
pub use crate::facade::certification::document_retention_model::readback::read_retention;
pub use crate::facade::certification::document_retention_model::retention_entry::{
    SetRetentionIntent, DOCUMENT_IDENTITY,
};
pub use crate::facade::certification::document_retention_model::schema::SetRetentionInput;

pub const KEY: u64 = 0x2701_0201;
pub const RETENTION: u64 = SEED_RETENTION + 1;

/// The unchanged retry names the key's original commit and commits nothing;
/// a changed retry under the same key is drift.
pub fn assert_the_key_resolves_by_its_durable_record(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    restored: &DocumentRetentionRuntime<RetentionProgramP0>,
    commit: CommitId,
) {
    let branch = restored.current_world();
    let ledger = || {
        restored
            .granular_invalidation_installation()
            .retain_primary_graph_integration_handle()
            .with_runtime(request_execution, |runtime| {
                runtime.history().latest_patch_stream_position()
            })
    };
    let restored_head = ledger();
    for _ in 0..3 {
        let unchanged = set_retention(restored, branch, RETENTION, KEY)
            .expect("the freshly admitted retry resolves the durable commit");
        let WorthQueryApplicationMutationOutcome::PreviouslyCommitted(observation) = &unchanged
        else {
            panic!("the restored retry observes its earlier commit: {unchanged:?}");
        };
        assert_eq!(observation.commit_id(), commit);
        assert!(
            unchanged.receipt().is_none(),
            "history grants no live receipt authority"
        );
        assert!(
            unchanged.result().is_none(),
            "history does not invent the handler result"
        );
        assert_eq!(ledger(), restored_head, "the retry commits nothing");
        assert_eq!(read_retention(restored.runtime(), branch), RETENTION);
    }
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() + std::time::Duration::from_secs(30)),
        cancellation.token(),
    );
    let principal = authenticate_operator(restored.installed_schema(), &scope);
    cancellation.cancel();
    let refused = restored
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .advanced()
        .mutate(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: RETENTION,
            },
        })
        .without_source()
        .idempotency(&KEY)
        .execute_in_program(
            restored,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert!(
        matches!(
            &refused,
            Err(WorthQueryApplicationRequestMutationDenial::ExecutionRequest(
                worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(
                    worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled
                )
            ))
        ),
        "a cancelled fresh admission cannot disclose historical commitment: {refused:?}"
    );
    assert_eq!(ledger(), restored_head, "refused admission commits nothing");

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
    assert_eq!(ledger(), restored_head, "drift commits nothing");
}
