use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_host::facade::{
    primary_graph::WorthQueryApplicationIdempotencyResolution as Resolution,
    WorthQueryApplicationHomeLocation,
};

#[test]
fn original_receipt_source_read_preserves_settled_effect_and_exposes_full_native_meaning() {
    let path = tempfile::tempdir().unwrap();
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    for (name, location) in [
        ("memory", WorthQueryApplicationHomeLocation::MemoryOnly),
        (
            "home",
            WorthQueryApplicationHomeLocation::Directory(path.path().to_path_buf()),
        ),
    ] {
        let contacts = Arc::new(AtomicUsize::new(0));
        let home = host_observation::opening::open(
            location,
            opening::build(),
            Arc::new(AtomicUsize::new(0)),
            contacts.clone(),
            None,
            Default::default(),
            &execution,
        );
        let outcome = host_observation::requests::set(home.runtime(), 8_822, 468);
        assert!(outcome.receipt().is_some());
        let before = host_observation::projection::observe(home.runtime(), &execution);
        let observed =
            host_observation::requests::resolve_original(home.runtime(), 8_822, 468).unwrap();
        let Resolution::RecoveredCommitted(original) = observed else {
            panic!("settled original was not read: {observed:?}")
        };
        assert_eq!(contacts.load(Ordering::SeqCst), 1);
        let after = host_observation::projection::observe(home.runtime(), &execution);
        assert_eq!(before.head, after.head);
        assert_eq!(host_observation::requests::balance(home.runtime()), 468);
        assert_eq!(
            original.commit_reference(),
            outcome.receipt().unwrap().commit_reference()
        );
        println!(
            "ORACLE_ORIGINAL_NATIVE {name} {}",
            serde_json::to_string(original.semantic_result().semantic()).unwrap()
        );
        println!(
            "ORACLE_GRAPH_VERSIONS {name} {}",
            serde_json::to_string(
                &after
                    .entities
                    .iter()
                    .map(|e| (&e.kind_name, e.created_at_version))
                    .collect::<Vec<_>>()
            )
            .unwrap()
        );
    }
}

#[test]
fn original_key_resolution_completes_pending_query_settlement_without_another_effect() {
    let path = tempfile::tempdir().unwrap();
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    for location in [
        WorthQueryApplicationHomeLocation::MemoryOnly,
        WorthQueryApplicationHomeLocation::Directory(path.path().to_path_buf()),
    ] {
        let contacts = Arc::new(AtomicUsize::new(0));
        let home = host_observation::opening::open(
            location,
            opening::build(),
            Arc::new(AtomicUsize::new(0)),
            contacts.clone(),
            None,
            Default::default(),
            &execution,
        );
        let before = host_observation::projection::observe(home.runtime(), &execution);
        let outcome =
            host_interruption::publication::settlement_failure(home.runtime(), 9_193, 469);
        assert!(outcome.receipt().is_none() && outcome.recovered_receipt().is_none());
        let pending = host_observation::projection::observe(home.runtime(), &execution);
        assert_eq!(pending.head.generation, before.head.generation + 1);
        let resolved =
            host_interruption::publication::continue_custody(home.runtime(), 9_193, 469).unwrap();
        let Resolution::RecoveredCommitted(original) = resolved else {
            panic!("original settlement remains unresolved: {resolved:?}")
        };
        assert_eq!(contacts.load(Ordering::SeqCst), 1);
        assert_eq!(
            host_observation::projection::observe(home.runtime(), &execution).head,
            pending.head
        );
        assert_eq!(host_observation::requests::balance(home.runtime()), 469);
        assert_eq!(
            host_observation::receipts::observe_original(&original).fate,
            host_observation::receipts::MutationFate::Committed
        );
    }
}
