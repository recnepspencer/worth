//! After a graceful close, one changed Signal input refuses the next Home open
//! before service: physical damage, or Signal's own typed cause.
use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::{OrderedJournalName, OrderedJournalPublication};
use worth_store_testkit_application_oracle::home_opening;
use worth_store_testkit_application_oracle::world_home::{
    self, ChangedSignalAdvance, SignalOpenRefusal,
};

#[derive(Clone, Copy, Debug)]
enum Change {
    /// One byte of a Signal owner frame on disk.
    DamagedFrame,
    /// A durable Signal advance with a changed recorded input.
    Advance(ChangedSignalAdvance),
}

#[test]
fn each_changed_signal_input_refuses_the_next_home_open_before_service() {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    for (change, expected) in [
        (Change::DamagedFrame, SignalOpenRefusal::Physical),
        (
            Change::Advance(ChangedSignalAdvance::ForeignOwner),
            SignalOpenRefusal::ForeignOwner,
        ),
        (
            Change::Advance(ChangedSignalAdvance::UnknownVersion),
            SignalOpenRefusal::UnsupportedVersion,
        ),
        (
            Change::Advance(ChangedSignalAdvance::DefinitionBasis),
            SignalOpenRefusal::MissingDefinition,
        ),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let opening = || {
            home_opening::request(
                directory.path(),
                crate::workload::runtime_builder(crate::schema_transition::SOURCE),
            )
        };
        let fixture = world_home::open_with_derived_signal(opening(), &execution).unwrap();
        drop(fixture.bootstrap_root(&execution));
        let row = match change {
            Change::DamagedFrame => None,
            Change::Advance(advance) => Some(fixture.changed_signal_advance(advance, &execution)),
        };
        let closed = fixture
            .home
            .close(&execution)
            .unwrap_or_else(|cause| panic!("graceful close: {:?}", cause.reason()));
        assert!(matches!(
            closed,
            worth_store_application::ApplicationHomeCloseOutcome::Closed {
                checkpoint: Ok(_),
                ..
            }
        ));
        match row {
            None => assert!(
                crate::open_refusals::flip_after(directory.path(), b"WSIGOWNR", 60, &|_| true) > 0,
                "a Signal owner frame is on disk"
            ),
            Some(row) => append_signal_advance(directory.path(), row, &execution),
        }
        let refused = match world_home::open_with_derived_signal(opening(), &execution) {
            Ok(_) => panic!("{change:?}: no Home is issued"),
            Err(refused) => refused,
        };
        assert_eq!(
            world_home::signal_open_refusal(&refused),
            expected,
            "{change:?}"
        );
    }
}

/// The Store holds the changed row with valid physical framing, so only
/// Signal's own readmission can refuse it.
fn append_signal_advance(
    location: &std::path::Path,
    row: Vec<u8>,
    execution: &ExecutionRequest<'_, '_>,
) {
    let home = crate::durable_run::open(location, execution).unwrap();
    let journal = home
        .physical_serving_for_test()
        .ordered_journal(OrderedJournalName::new("signal.advances").unwrap())
        .unwrap();
    let next = journal
        .head()
        .unwrap()
        .map_or(1, |entry| entry.position() + 1);
    journal
        .publish(OrderedJournalPublication::new(
            next,
            row,
            crate::open_refusals::home_placement(),
            crate::open_refusals::deadline(),
        ))
        .unwrap();
    home.abort_for_test();
}
