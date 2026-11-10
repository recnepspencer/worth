//! Metadata funding only; opaque scalar entries supply no typed managed READ.
use std::any::Any;
use std::sync::Arc;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
use worth_execution::ExecutionRequest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::managed_index::retention::association::ManagedIndexRegistrationKey as Key;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::derived_view::registry::index_directory::ManagedIndexDirectory;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::derived_view::registry::index_directory::RegisteredIndex;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::derived_view::registry::RegisteredManagedIndex;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use worth_execution::CancellationSource;
use worth_execution::SerialMemoryBudget;
use worth_execution::SerialRequest;

#[derive(Default)]
pub struct Cleanup(AtomicUsize);
impl RegisteredManagedIndex for Cleanup {
    fn dispose(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn registration_memory_refusal_preserves_old_pair_and_funded_retry_releases_backing() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let keys = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let selected = world.selected_product(&host);
            let scope = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::at("AccountStatus"),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let key = |purpose| {
                Key::storage_key_for_test(
                    (*world.application.runtime_for_test()).authority_identity_for_test(),
                    selected
                        .product()
                        .observation_for_test()
                        .lifecycle_incarnation(),
                    scope.entity_id_for_test(),
                    purpose,
                )
            };
            [key([1; 32]), key([2; 32])]
        })
        .unwrap();
    let memory = SerialMemoryBudget::new(None);
    let original =
        SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let execution = ExecutionRequest::serial(&original);
    let cleanup = Arc::new(Cleanup::default());
    let first: Arc<dyn Any + Send + Sync> = Arc::new(19u64);
    let second: Arc<dyn Any + Send + Sync> = Arc::new(23u64);
    let mut directory = ManagedIndexDirectory::default();
    directory.insert_for_test(
        1,
        RegisteredIndex::prepare_for_test(&execution, keys[0], first.clone(), cleanup.clone())
            .unwrap(),
    );
    let before = memory.charged_memory_bytes();
    assert!(
        before > 0,
        "both route and id backing belong to the original payer"
    );
    let denied_memory = SerialMemoryBudget::new(Some(0));
    let denied = SerialRequest::from_memory(
        denied_memory.clone(),
        CancellationSource::new().token(),
        None,
    );
    let refused = RegisteredIndex::prepare_for_test(
        &ExecutionRequest::serial(&denied),
        keys[1],
        second.clone(),
        cleanup.clone(),
    );
    assert!(matches!(
        refused,
        Err(Cause::PolicyMemoryExhausted { admitted: 0, .. })
    ));
    assert_eq!(directory.len_for_test(), 1);
    let (id, retained) = directory.get_for_test(&keys[0]).unwrap();
    assert_eq!(id, 1);
    assert!(Arc::ptr_eq(retained.state_for_test(), &first));
    assert!(directory.get_for_test(&keys[1]).is_none());
    assert_eq!(memory.charged_memory_bytes(), before);
    assert_eq!(denied_memory.charged_memory_bytes(), 0);
    directory.insert_for_test(
        2,
        RegisteredIndex::prepare_for_test(&execution, keys[1], second.clone(), cleanup.clone())
            .unwrap(),
    );
    cleanup.dispose(); // The public close receiver disposes before unregistering.
    drop(directory.remove_for_test(1));
    assert!(directory.get_for_test(&keys[0]).is_none());
    let (id, retained) = directory.get_for_test(&keys[1]).unwrap();
    assert_eq!(id, 2);
    assert!(Arc::ptr_eq(retained.state_for_test(), &second));
    assert!(memory.charged_memory_bytes() > 0);
    directory.dispose_for_test();
    assert_eq!(cleanup.0.load(Ordering::SeqCst), 2);
    assert_eq!(memory.charged_memory_bytes(), 0);
}
