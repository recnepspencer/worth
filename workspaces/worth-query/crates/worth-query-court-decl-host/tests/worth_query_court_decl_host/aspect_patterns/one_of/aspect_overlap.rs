//! Candidate overlap is refused before provider lowering in either authored order.
use super::*;

#[test]
fn removal_and_write_overlap_refuses_in_both_orders() {
    let host = world::install();
    let before = world::read(&host);
    let selected = world::pin(&host);
    for (change, key) in [
        (authoring::Change::RemoveThenWriteFirst, "remove-then-write"),
        (authoring::Change::WriteFirstThenRemove, "write-then-remove"),
    ] {
        let result = world::try_edit(&host, change, key);
        let Err(WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Handler(denial),
        )) = result
        else {
            panic!("candidate authoring must refuse: {result:?}");
        };
        let denial = denial
            .downcast::<WorthQueryApplicationAttemptDenial>()
            .expect("the original typed candidate refusal survives the handler wrapper");
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep
        );
        let subject = denial.subject();
        assert!(
            subject.starts_with("Record RecordId {") || subject.starts_with("Record EntityId {"),
            "{subject}"
        );
        assert!(
            subject.contains(&format!(".{}", First::reference().name())),
            "{subject}"
        );
        assert_eq!(
            selected.selected_commit(),
            world::pin(&host).selected_commit()
        );
        assert_eq!(world::read(&host), before);
    }
}
