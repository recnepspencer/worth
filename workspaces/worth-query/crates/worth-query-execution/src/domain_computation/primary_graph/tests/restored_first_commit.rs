//! A restored runtime answers index maintenance exactly as the runtime that
//! captured it did.
//!
//! The durable ledger owns the patch-stream position, and a checkpoint
//! restore carries it. The first raw commit after a restore, which is the
//! mutation `WorthQueryPrimaryGraphBackendHandle::execute_mutation` delegates
//! to, therefore resumes the patch stream where the capture left it, whether
//! or not the store also holds a retired index definition. A World
//! publication first, followed by a raw commit, gives the same answer.

use crate::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::publication::PatchStreamPosition;

use super::fixture::{installed_world, restored_world, IdentityWorld};
use super::index_refresh::commit_empty;
use super::retired_index_checkpoint::{
    assert_retired_index_is_inert, persist_as_the_retired_code_did, resolve_alice,
    set_principal_identity, with_runtime,
};

#[test]
fn a_raw_commit_first_after_restore_refreshes_every_installed_index() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let host_request = crate::test_request_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        raw_commits_first_after_restore(&host_custody, execution, false);
    })
}

#[test]
fn a_raw_commit_first_after_restore_refreshes_beside_a_retired_index() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let host_request = crate::test_request_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        raw_commits_first_after_restore(&host_custody, execution, true);
    })
}

#[test]
fn a_world_publication_then_raw_commit_after_restore_refreshes_every_installed_index() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        world_publication_first_after_restore(
            &host_custody,
            &host_custody.borrowed_request_for_test(),
            false,
        );
    })
}

#[test]
fn a_world_publication_then_raw_commit_after_restore_refreshes_beside_a_retired_index() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        world_publication_first_after_restore(
            &host_custody,
            &host_custody.borrowed_request_for_test(),
            true,
        );
    })
}

/// Two raw commits in a row: the first resumes the patch stream at the
/// restored ledger position, the second after the first.
fn raw_commits_first_after_restore(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    with_retired_index: bool,
) {
    let (restored, retired) = captured_and_restored(phase_custody, execution, with_retired_index);
    let first = raw_commit(execution, &restored);
    assert_every_installed_index_current(execution, &restored, &first, retired);
    let second = raw_commit(execution, &restored);
    assert_every_installed_index_current(execution, &restored, &second, retired);
}

/// A World publication first, then a raw commit. A raw commit advances main
/// outside the selected Product World, so it is the last step here.
fn world_publication_first_after_restore(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    with_retired_index: bool,
) {
    let (restored, retired) = captured_and_restored(phase_custody, execution, with_retired_index);
    let principal = resolve_alice(phase_custody, &restored, 2);
    set_principal_identity(execution, &restored, principal, 3);
    resolve_alice(phase_custody, &restored, 3);
    let raw = raw_commit(execution, &restored);
    assert_every_installed_index_current(execution, &restored, &raw, retired);
}

/// Builds a world with World-published history, optionally persists a retired
/// index definition, captures the world, and restores the capture.
fn captured_and_restored(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    with_retired_index: bool,
) -> (IdentityWorld, Option<DerivedIndexId>) {
    let world = installed_world(&[("alice", WorthQueryPrincipalMappingStatus::Enabled)]);
    let principal = resolve_alice(phase_custody, &world, 1);
    set_principal_identity(execution, &world, principal, 2);
    let retired = with_retired_index.then(|| persist_as_the_retired_code_did(execution, &world));
    let captured = with_runtime(execution, &world, ledger_position);
    assert!(captured.is_some(), "the captured ledger has commits");
    let checkpoint = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
        })
        .expect("installed host call opens")
        .expect("the installed world captures");
    drop(world);

    let restored = restored_world(checkpoint).expect("the captured world restores");
    assert_eq!(
        with_runtime(execution, &restored, ledger_position),
        captured,
        "a restore answers the ledger position the capture held"
    );
    if let Some(retired) = retired {
        assert_retired_index_is_inert(execution, &restored, retired);
    }
    (restored, retired)
}

/// Commits through the integration handle's index-refreshing mutation, the
/// one the production backend handle delegates to.
fn raw_commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &IdentityWorld,
) -> RelationalCommitReceipt {
    let graph = world.application.runtime.primary_graph().unwrap();
    graph
        .integration_handle()
        .execute_mutation_with_index_refresh(execution, |runtime| {
            Ok::<_, &'static str>(commit_empty(runtime, "main"))
        })
        .expect("a mutation after restore refreshes only the commits it made")
        .expect("the empty commit succeeds")
}

fn assert_every_installed_index_current(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &IdentityWorld,
    commit: &RelationalCommitReceipt,
    retired: Option<DerivedIndexId>,
) {
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    with_runtime(request_execution, world, |runtime| {
        for index_id in handle.primary_index_ids.iter() {
            assert!(
                runtime
                    .index_access()
                    .published_generation_for_commit(*index_id, commit)
                    .is_some(),
                "installed index {index_id:?} is current at the new commit"
            );
        }
        if let Some(retired) = retired {
            assert!(
                runtime
                    .index_access()
                    .published_generation_for_commit(retired, commit)
                    .is_none(),
                "nothing maintains the retired index"
            );
        }
    });
}

fn ledger_position(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
) -> Option<PatchStreamPosition> {
    runtime.history().latest_patch_stream_position()
}
