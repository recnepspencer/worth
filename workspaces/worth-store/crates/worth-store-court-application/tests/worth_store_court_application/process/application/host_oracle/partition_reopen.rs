//! Private mutation namespaces survive ordinary owned Home close and reopen.
use super::*;
use std::sync::{atomic::AtomicUsize, Arc};
use worth_query_host::facade::{
    primary_graph::WorthQueryApplicationIdempotencyResolution as Resolution,
    WorthQueryApplicationHomeLocation,
};

#[test]
fn reopened_host_never_reuses_a_committed_private_mutation_partition() {
    let path = tempfile::tempdir().unwrap();
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let mut partitions = std::collections::HashSet::new();
    let initialized = Arc::new(AtomicUsize::new(0));
    for index in 0..3 {
        let home = host_observation::opening::open(
            WorthQueryApplicationHomeLocation::Directory(path.path().to_path_buf()),
            opening::build(),
            initialized.clone(),
            Arc::new(AtomicUsize::new(0)),
            None,
            Default::default(),
            &execution,
        );
        let key = 98_000 + index;
        let value = 840 + index;
        let outcome = host_observation::requests::set(home.runtime(), key, value);
        assert!(outcome.receipt().is_some());
        drop(outcome);
        let Resolution::RecoveredCommitted(original) =
            host_observation::requests::resolve_original(home.runtime(), key, value).unwrap()
        else {
            panic!("the actual original mutation receipt is required");
        };
        let created = original.semantic_result().semantic().created_entities();
        assert_eq!(
            created.len(),
            1,
            "this update creates its keyed registration only"
        );
        let partition = created[0].assigned.partition_id;
        assert!(
            partitions.insert(partition),
            "reopen reused private partition {partition:?}"
        );
        assert_eq!(created[0].key.partition, partition);
        drop(original);
        let closed = home
            .close(&execution)
            .unwrap_or_else(|refused| panic!("close: {refused:?}"));
        assert!(closed.is_closed());
        closed.reclaim();
    }
    assert_eq!(initialized.load(std::sync::atomic::Ordering::SeqCst), 1);
}
