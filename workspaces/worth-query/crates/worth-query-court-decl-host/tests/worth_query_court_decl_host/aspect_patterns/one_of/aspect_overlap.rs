//! Candidate overlap is refused before provider lowering in either authored order.
use super::*;

#[test]
fn removing_and_writing_one_aspect_names_the_same_entity_and_aspect_in_both_orders() {
    let host = world::install();
    let before = world::read(&host);
    let selected = world::pin(&host);
    let mut subjects = Vec::new();
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
            subject.ends_with(&format!(".{}", First::reference().name())),
            "{subject}"
        );
        subjects.push(subject.to_owned());
        assert_eq!(
            selected.selected_commit(),
            world::pin(&host).selected_commit()
        );
        assert_eq!(world::read(&host), before);
    }
    assert_eq!(
        subjects[0], subjects[1],
        "both orders identify the exact same record and parent"
    );
}
