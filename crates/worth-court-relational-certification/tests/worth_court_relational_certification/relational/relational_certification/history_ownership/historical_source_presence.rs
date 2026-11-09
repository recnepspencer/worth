//! Source identity is checked at the capsule's own D, independently of C.
use super::read_evidence_fixture::SelectedWorld;
use super::*;
use worth_relational::facade::runtime::RelationalHistoricalSourcePresence as Presence;

#[test]
fn a_source_absent_before_restoration_is_live_only_in_the_exact_restored_domain() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = SelectedWorld::new(execution);
    let source = member(execution, &world.runtime, "later-source");
    create_relation(execution, &world.runtime, world.model, source, "owns-later");
    let selected = transfer_fixture::basis(&world.runtime);
    let absent =
        transfer_fixture::select(execution, &world.runtime, world.model, &world.selected).unwrap();
    release_test_commit_snapshot(&world.runtime, &absent);
    assert!(begin(&world.runtime)
        .read_entity(execution, source)
        .unwrap()
        .base()
        .is_none());
    let restored =
        transfer_fixture::select(execution, &world.runtime, world.model, &selected).unwrap();
    release_test_commit_snapshot(&world.runtime, &restored);
    let restored = transfer_fixture::basis(&world.runtime);
    let evidence = world
        .runtime
        .read_truth()
        .read_historical_selection(execution, &selected.observation(), &restored.observation())
        .unwrap();
    assert_eq!(
        evidence
            .source_presence(execution, source, KindId(3))
            .unwrap(),
        Presence::LiveInDomain
    );
    assert_eq!(
        evidence
            .source_presence(execution, world.outside, KindId(1))
            .unwrap(),
        Presence::OutsideDomain
    );
    assert_eq!(
        evidence
            .source_presence(execution, source, KindId(1))
            .unwrap(),
        Presence::KindMismatch { actual: KindId(3) }
    );
    // A different selection legitimately retires this exact generation. Its
    // own capsule must not accept the earlier live D, even though it is pinned.
    let absent =
        transfer_fixture::select(execution, &world.runtime, world.model, &world.selected).unwrap();
    release_test_commit_snapshot(&world.runtime, &absent);
    let absent = transfer_fixture::basis(&world.runtime);
    let absent_evidence = world
        .runtime
        .read_truth()
        .read_historical_selection(
            execution,
            &world.selected.observation(),
            &absent.observation(),
        )
        .unwrap();
    assert_eq!(
        absent_evidence
            .source_presence(execution, source, KindId(3))
            .unwrap(),
        Presence::NotLive
    );
    assert_eq!(
        evidence
            .source_presence(execution, source, KindId(3))
            .unwrap(),
        Presence::LiveInDomain
    );
}
