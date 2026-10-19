//! Native context storage/read evidence; real guarded authorization is covered by supplier journeys.
mod fixture;
use fixture::Context;

#[test]
fn native_output_references_keep_existing_created_and_later_retired_identity() {
    let context = Context::publish(None, false);
    context.check().unwrap();
    context.assert_protected_context();
    assert_eq!(
        context.targets(),
        vec![
            ("created".into(), context.created),
            ("source".into(), context.source)
        ]
    );
    context.retire(context.created);
    context.check().unwrap();
    assert_eq!(
        context.targets(),
        vec![
            ("created".into(), context.created),
            ("source".into(), context.source)
        ]
    );
}

#[test]
fn retire_output_can_reference_a_target_deleted_in_the_same_publication() {
    let context = Context::publish(None, true);
    context.check().unwrap();
    assert_eq!(
        context.targets(),
        vec![
            ("created".into(), context.created),
            ("source".into(), context.source)
        ]
    );
}

#[test]
fn cold_output_context_refuses_missing_downgraded_and_incoherent_metadata() {
    for defect in [
        "missing",
        "missing-target",
        "count",
        "duplicate",
        "posture",
        "empty",
        "downgrade",
        "overflow",
    ] {
        let context = Context::publish(Some(defect), false);
        assert!(
            context.check().is_err(),
            "{defect} must fail native reconstruction"
        );
    }
}
