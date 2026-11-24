//! Resume retained export without demanding skipped cold journal bodies.
use super::envelope_source::reopened;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    config::RelationalRuntimeProfile, publication::PatchStreamPosition,
    storage::RelationalStorageUnavailableCause,
};
use worth_testkit_relational::durable_medium::immutable_members::ColdRead;

#[test]
fn seeking_past_an_acknowledged_index_prefix_never_demands_its_journal_bodies() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let fixture = reopened(
        execution,
        4,
        true,
        RelationalRuntimeProfile::CertificationCore,
    );
    let (position, bytes) = fixture.source.commits.last_key_value().unwrap();
    assert!(position.0 >= 4);
    {
        let mut failures = fixture.source.failures.lock().unwrap();
        for &prefix in fixture
            .source
            .commits
            .keys()
            .filter(|prefix| *prefix < position)
        {
            failures.insert(
                ColdRead::Commit(prefix),
                RelationalStorageUnavailableCause::SourceUnavailable,
            );
        }
    }
    let selected = fixture
        .runtime
        .durability()
        .canonical_commit_artifacts(execution)
        .after_position(PatchStreamPosition(position.0 - 1))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].as_bytes(), bytes.as_slice());
    assert_eq!(*fixture.source.commit_reads.lock().unwrap(), [*position]);
}
