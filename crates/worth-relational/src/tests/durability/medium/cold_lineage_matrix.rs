//! Every node-consuming public operation shares the failure-class contract.
use super::cold_lineage_fixture::reopened;
use super::cold_lineage_nodes::NodeDoor;
use super::cold_read_matrix::FAILURES;
use super::cold_source::ColdRead;
use super::*;
use crate::execution::RelationalExecutionDenialCause as Execution;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};

#[test]
fn residency_failure_matrix_lineage_doors_preserve_every_failure_class() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in NodeDoor::ALL {
        for (cause, sticky) in FAILURES {
            let fixture = reopened(execution);
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            let offered = attached_test_medium(&fixture.runtime).offered().len();
            fixture
                .source
                .failures
                .lock()
                .unwrap()
                .insert(ColdRead::Member(fixture.page.member), cause);
            let expected = Execution::StorageUnavailable(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Lineage {
                    first: crate::identity::data::LineageId(fixture.page.first),
                    last: crate::identity::data::LineageId(fixture.page.last),
                },
                cause,
            });
            assert_eq!(
                door.read(&fixture, execution),
                Err(expected),
                "{door:?} x {cause:?}"
            );
            fixture.source.failures.lock().unwrap().clear();
            let reads = fixture.source.reads.load(Ordering::Relaxed);
            if sticky {
                assert_eq!(door.read(&fixture, execution), Err(expected));
                assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
            } else {
                door.read(&fixture, execution).unwrap();
                assert!(fixture.source.reads.load(Ordering::Relaxed) > reads);
                assert!(
                    fixture
                        .runtime
                        .lineage
                        .node(fixture.expected.lineage_id())
                        .is_some(),
                    "retry must admit the requested node"
                );
            }
            assert_eq!(
                fixture.runtime.history().latest_patch_stream_position(),
                frontier
            );
            assert_eq!(
                attached_test_medium(&fixture.runtime).offered().len(),
                offered
            );
        }
    }
}
