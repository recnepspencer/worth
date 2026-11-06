use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_host::facade::WorthQueryApplicationHomeLocation;

#[test]
fn concurrent_same_key_retries_return_one_original_occurrence_without_reexecuting_its_retry() {
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
        let before = host_observation::projection::observe(home.runtime(), &execution).head;
        let outcome =
            host_observation::concurrent_mutation::execute(home.runtime(), 445, 987, &before);
        assert!(matches!(outcome, worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::AlreadyCommitted(_) | worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::RecoveredCommitted(_)));
        let evaluations = contacts.load(Ordering::SeqCst);
        assert!(
            (1..=2).contains(&evaluations),
            "only the two initial contenders may evaluate their pure candidates"
        );
        let retry = host_observation::requests::set(home.runtime(), 445, 987);
        assert_eq!(
            contacts.load(Ordering::SeqCst),
            evaluations,
            "an original-result retry never invokes the handler"
        );
        assert_eq!(
            serde_json::to_value(host_observation::receipts::observe(&outcome, &before)).unwrap(),
            serde_json::to_value(host_observation::receipts::observe(&retry, &before)).unwrap()
        );
        let after = host_observation::projection::observe(home.runtime(), &execution);
        assert_eq!(
            after.head.generation,
            before.generation + 1,
            "one original product occurrence"
        );
        assert_eq!(after.idempotency.len(), 1);
        assert_eq!(host_observation::requests::balance(home.runtime()), 987);
    }
}
