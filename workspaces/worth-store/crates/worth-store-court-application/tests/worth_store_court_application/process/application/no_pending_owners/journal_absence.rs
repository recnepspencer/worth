use super::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_runtime_world::facade::RuntimeWorldCancellationSource;
use worth_store::physical_runtime::OrderedJournalResolution;
use worth_store_application::ApplicationHomeOwnerAdmissionDenial as Denial;

#[test]
fn empty_unsupported_journals_admit_the_same_home_with_real_native_and_world_history() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(directory.path(), &execution);
    let bootstrap = fixture.bootstrap_root(&execution);
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        &execution,
        bootstrap.product_branch().clone(),
        crate::batches::create(0),
        &cancellation.token(),
    );
    drop(
        fixture
            .publish(&execution, prepared, &cancellation.token())
            .consume(),
    );
    drop(bootstrap);
    let home = fixture.home;
    let identity = home.home_identity();
    let incarnation = home.incarnation();
    let native = home.relational().runtime().runtime_instance_id();
    let physical = home.physical_serving_for_test().runtime_identity();
    let admitted = home.readmit_no_pending_owners(&execution).unwrap();
    assert_contended(directory.path(), &execution);
    let home = admitted.into_no_pending_home();
    assert_eq!(home.home_identity(), identity);
    assert_eq!(home.incarnation(), incarnation);
    assert_eq!(home.relational().runtime().runtime_instance_id(), native);
    assert_eq!(
        home.physical_serving_for_test().runtime_identity(),
        physical
    );
    close(home, &execution);
}

#[test]
fn each_unsupported_journal_entry_refuses_admission_and_returns_the_same_home() {
    for name in [
        "home.transitions",
        "signal.definitions",
        "signal.advances",
        "query.obligations",
        "query.dispatch",
        "query.corrections",
    ] {
        let directory = tempfile::tempdir().unwrap();
        let request = crate::workload::serial_request();
        let execution = ExecutionRequest::serial(&request);
        let fixture = open(directory.path(), &execution);
        drop(fixture.bootstrap_root(&execution));
        let home = fixture.home;
        let identity = home.home_identity();
        let incarnation = home.incarnation();
        let physical = home.physical_serving_for_test().runtime_identity();
        let entry = publish_entry(home.physical_serving_for_test(), name)
            .unwrap()
            .entry();
        let journal = home
            .physical_serving_for_test()
            .ordered_journal(OrderedJournalName::new(name).unwrap())
            .unwrap();
        assert_eq!(journal.head().unwrap(), Some(entry));
        assert_eq!(
            journal.resolve(1, [0; 32]).unwrap(),
            OrderedJournalResolution::Conflict(entry)
        );
        let refused = home
            .readmit_no_pending_owners(&execution)
            .err()
            .expect("a genuine reserved journal entry must refuse admission");
        if name == "query.obligations" {
            assert!(matches!(refused.reason(), Denial::QueryObligations(worth_query_execution::facade::program_support::QueryProgramSupportDenial::SupportUnavailable)));
        } else {
            assert!(
                matches!(refused.reason(), Denial::JournalNotEmpty { journal, head } if *journal == name && *head == entry)
            );
        }
        let home = refused.into_home();
        assert_eq!(
            home.physical_serving_for_test()
                .ordered_journal(OrderedJournalName::new(name).unwrap())
                .unwrap()
                .head()
                .unwrap(),
            Some(entry),
            "refused owner admission cannot alter the original prefix"
        );
        assert_eq!(home.home_identity(), identity);
        assert_eq!(home.incarnation(), incarnation);
        assert_eq!(
            home.physical_serving_for_test().runtime_identity(),
            physical
        );
        close(home, &execution);
    }
}

#[test]
fn stopped_owner_admission_preserves_the_original_home_and_lease() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(directory.path(), &execution);
    drop(fixture.bootstrap_root(&execution));
    let home = fixture.home;
    let identity = home.home_identity();
    let physical = home.physical_serving_for_test().runtime_identity();
    let cancellation = CancellationSource::new();
    let stopped =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    cancellation.cancel();
    let refused = home
        .readmit_no_pending_owners(&ExecutionRequest::serial(&stopped))
        .err()
        .expect("the original stopped request refuses before admission");
    assert!(matches!(refused.reason(), Denial::Execution(_)));
    assert_contended(directory.path(), &execution);
    let home = refused.into_home();
    assert_eq!(home.home_identity(), identity);
    assert_eq!(
        home.physical_serving_for_test().runtime_identity(),
        physical
    );
    close(home, &execution);
}
