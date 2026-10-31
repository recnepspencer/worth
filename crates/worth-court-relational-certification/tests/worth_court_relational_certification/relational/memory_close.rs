//! Memory business retirement over the real Native public owner and read cache.
use std::sync::{Arc, Mutex};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    change_source::{
        RelationalMemoryClose as Close, RelationalMemoryCloseDenial as EntryDenial,
        RelationalMemoryCloseObservation as O, RelationalRuntimeHandle as Handle,
    },
    mvcc::RelationalPublicationOutcome,
    runtime::{
        RelationalRuntimeApi, RelationalRuntimeCloseDenial as Denial, RelationalRuntimeConfig,
    },
};
use worth_testkit_relational::{
    aspect_storage::publications::{begin_main_transaction, release_test_commit_snapshot},
    durable_medium::TestDurableMedium,
    durable_publication::{
        batch_create, create_entity_outcome, durable_runtime, memory_runtime, schema,
    },
    shared_relational_certification_test_execution_policy::serial_request,
};

fn finish(close: &mut Close, request: &ExecutionRequest<'_, '_>) {
    for _ in 0..100 {
        match close.advance(request) {
            O::Advanced => {}
            O::Complete => return,
            other => panic!("unexpected genuine close progress: {other:?}"),
        }
    }
    panic!("the small actual fixture has finite cache/head retirement")
}

#[test]
fn memory_terminal_close_retires_real_cached_read_and_preserves_original_stop() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let result = create_entity_outcome(request, &runtime, "cached-close");
    let read = runtime
        .read_truth()
        .read_snapshot(&result.snapshot, request)
        .unwrap()
        .unwrap();
    assert_eq!(read.entities().len(), 1);
    release_test_commit_snapshot(&runtime, &result);
    drop(result);
    let forks = runtime.fork_port();
    let (_, source) = forks
        .observe_fork_source(runtime.main_branch_identity().branch_id())
        .unwrap();
    forks
        .fork_branch(
            request,
            worth_relational::facade::history::BranchId("memory-sibling".into()),
            source,
        )
        .unwrap();
    let refused = runtime.prepare_close().unwrap_err();
    assert_eq!(
        refused.reason(),
        Denial::ReadCustodyRetained,
        "real prior cache-close obstruction"
    );
    let runtime = refused.into_runtime();
    let identity = runtime.main_branch_identity();
    let port = runtime.owner_component_services().basis_port();
    let mut close = Handle::immutable(Arc::new(runtime))
        .into_memory_close()
        .unwrap();
    let cancellation = CancellationSource::new();
    let stopped_host =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    cancellation.cancel();
    let stopped = &ExecutionRequest::serial(&stopped_host);
    assert!(matches!(close.advance(stopped), O::Stopped(_)));
    assert!(
        port.observe_branch(&identity).is_ok(),
        "stop before cut changes no admission"
    );
    assert!(matches!(close.advance(request), O::Advanced));
    assert!(port.observe_branch(&identity).is_err());
    assert!(
        forks.observe_fork_source(identity.branch_id()).is_err(),
        "retained service cannot mutate/reorder the sealed head inventory"
    );
    assert!(matches!(close.advance(stopped), O::Stopped(_)));
    let mut close = close.into_refused_handle().unwrap_err();
    finish(&mut close, request);
    assert!(
        matches!(close.advance(stopped), O::Complete),
        "actual completion precedes late stop"
    );
    let retired = close.into_retired_memory().unwrap();
    assert_eq!(read.entities().len(), 1, "owned read output remains valid");
    assert!(
        port.observe_branch(&identity).is_err(),
        "retained heap is nonserving"
    );
    retired.reclaim(); // Explicitly unbudgeted heap destruction, not the original request.
}

#[test]
fn memory_terminal_close_keeps_actual_independent_projection_until_its_release() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let result = create_entity_outcome(request, &runtime, "independent-memory-reader");
    let entity = result
        .changed_records
        .iter()
        .find_map(|record| match record {
            worth_relational::facade::transactions::RecordRef::Entity(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    runtime
        .read_truth()
        .read_snapshot(&result.snapshot, request)
        .unwrap()
        .unwrap();
    let view = runtime
        .read_truth()
        .project_snapshot_in_request(&result.snapshot, request)
        .unwrap()
        .unwrap();
    release_test_commit_snapshot(&runtime, &result);
    drop(result);
    let mut close = Handle::immutable(Arc::new(runtime))
        .into_memory_close()
        .unwrap();
    let mut refused = false;
    for _ in 0..30 {
        match close.advance(request) {
            O::Advanced => {}
            O::Refused(Denial::ReadCustodyRetained) => {
                refused = true;
                break;
            }
            other => panic!("independent reader must retain real custody: {other:?}"),
        }
    }
    assert!(refused);
    assert_eq!(
        view.read_exact_entity_state(request, entity, |metadata, _| metadata.entity_id)
            .unwrap(),
        Some(entity)
    );
    drop(view);
    finish(&mut close, request);
    close.into_retired_memory().unwrap().reclaim();
}

#[test]
fn memory_terminal_close_refuses_shared_and_actual_source_affinity_without_losing_handle() {
    let runtime = Arc::new(memory_runtime());
    let identity = runtime.runtime_instance_id();
    let handle = Handle::immutable(runtime.clone());
    let refused = handle.into_memory_close().unwrap_err();
    assert_eq!(refused.reason(), EntryDenial::SharedOwner);
    let handle = refused.into_handle();
    assert_eq!(handle.runtime_instance_id(), identity);
    drop(runtime);
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let mut close = handle.into_memory_close().unwrap();
    finish(&mut close, request);
    close.into_retired_memory().unwrap().reclaim();

    let shared = Arc::new(Mutex::new(memory_runtime()));
    let handle = Handle::shared(shared.clone());
    let refused = handle.into_memory_close().unwrap_err();
    assert_eq!(refused.reason(), EntryDenial::SharedRepresentation);
    assert_eq!(
        refused
            .into_handle()
            .with_runtime(|runtime| runtime.runtime_instance_id()),
        shared.lock().unwrap().runtime_instance_id()
    );

    let medium = TestDurableMedium::new();
    let runtime = Arc::new(durable_runtime(&medium));
    let identity = runtime.runtime_instance_id();
    let refused = Handle::immutable(runtime).into_memory_close().unwrap_err();
    assert_eq!(refused.reason(), EntryDenial::SourceAffinity);
    assert_eq!(refused.into_handle().runtime_instance_id(), identity);
    assert!(medium.offered().is_empty());
}

#[test]
fn memory_terminal_close_refuses_actual_expired_candidate_until_owner_discards_it() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let mut policy = RelationalRuntimeConfig::default().publication.policy;
    policy.candidate_max_lifetime_millis = 0;
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema())
        .publication(policy)
        .build()
        .unwrap();
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            batch_create("expired-live-candidate"),
            request.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction.validate(request, &runtime).unwrap();
    let candidate = runtime
        .prepare_validated_proposal(request, proposal)
        .unwrap();
    let identity = runtime.main_branch_identity();
    let port = runtime.owner_component_services().basis_port();
    let mut close = Handle::immutable(Arc::new(runtime))
        .into_memory_close()
        .unwrap();
    assert!(matches!(
        close.advance(request),
        O::Refused(Denial::PublicationCustodyRetained)
    ));
    assert!(
        port.observe_branch(&identity).is_ok(),
        "preseal refusal restores admission"
    );
    drop(candidate); // Actual ordinary unbudgeted candidate cleanup, not a close revocation.
    finish(&mut close, request);
    close.into_retired_memory().unwrap().reclaim();
}

#[test]
fn memory_terminal_close_preserves_pending_settlement_and_explicit_snapshot_repair() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            batch_create("accepted-before-close"),
            request.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction.validate(request, &runtime).unwrap();
    let candidate = runtime
        .prepare_validated_proposal(request, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(request, candidate)
    else {
        panic!("genuine Native publication performs")
    };
    let mut close = Handle::immutable(Arc::new(runtime))
        .into_memory_close()
        .unwrap();
    assert!(matches!(
        close.advance(request),
        O::Refused(Denial::PublicationCustodyRetained)
    ));
    let handle = close.into_refused_handle().unwrap(); // Explicit unbudgeted preseal restoration.
    let result =
        handle.with_runtime(|runtime| runtime.settle_performed_publication(performed).unwrap());
    let mut close = handle.into_memory_close().unwrap();
    assert!(matches!(
        close.advance(request),
        O::Refused(Denial::ReadCustodyRetained)
    ));
    let handle = close.into_refused_handle().unwrap();
    handle.with_runtime(|runtime| release_test_commit_snapshot(runtime, &result));
    let mut close = handle.into_memory_close().unwrap();
    finish(&mut close, request);
    close.into_retired_memory().unwrap().reclaim();
}
