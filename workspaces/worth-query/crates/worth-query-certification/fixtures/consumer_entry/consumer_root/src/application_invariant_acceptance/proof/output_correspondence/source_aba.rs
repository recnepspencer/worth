use super::*;

pub(super) fn require_aspect_aba_denial(request: &Request<'_>, application: &ProgramApplication) {
    let source = observed_source(request, "anchor-a");
    require_adjustment_commit(mutate(
        request,
        application,
        adjust("anchor-a", 3, 4096),
        407,
    ));
    require_adjustment_commit(mutate(
        request,
        application,
        adjust("anchor-a", 2, 4096),
        408,
    ));
    assert_eq!(read_y(request, "anchor-a"), 2);
    require_source_changed(
        request
            .mutate(replacement(
                "replacement-b",
                "aspect-aba-must-not-publish",
                11,
                1,
            ))
            .expect_source(source)
            .idempotency(&409_u64)
            .execute_in_program(application),
    );
    require_absent(request, "aspect-aba-must-not-publish");
}

pub(super) fn require_child_aspect_aba_denial(
    request: &Request<'_>,
    application: &ProgramApplication,
) {
    let source = observed_source(request, "anchor-a");
    require_adjustment_commit(mutate(
        request,
        application,
        adjust("replacement-b", 2, 4096),
        413,
    ));
    require_adjustment_commit(mutate(
        request,
        application,
        adjust("replacement-b", 1, 4096),
        414,
    ));
    assert_eq!(read_y(request, "replacement-b"), 1);
    require_source_changed(
        request
            .mutate(replacement(
                "replacement-b",
                "child-aspect-aba-must-not-publish",
                11,
                1,
            ))
            .expect_source(source)
            .idempotency(&415_u64)
            .execute_in_program(application),
    );
    require_absent(request, "child-aspect-aba-must-not-publish");
}

pub(super) fn require_nested_aspect_aba_denial(
    request: &Request<'_>,
    application: &ProgramApplication,
) {
    let source = observed_source(request, "anchor-a");
    require_adjustment_commit(mutate(
        request,
        application,
        adjust("anchor-c", 11, 4096),
        416,
    ));
    require_adjustment_commit(mutate(
        request,
        application,
        adjust("anchor-c", 10, 4096),
        417,
    ));
    assert_eq!(read_y(request, "anchor-c"), 10);
    require_source_changed(
        request
            .mutate(replacement(
                "replacement-b",
                "nested-aspect-aba-must-not-publish",
                11,
                1,
            ))
            .expect_source(source)
            .idempotency(&418_u64)
            .execute_in_program(application),
    );
    require_absent(request, "nested-aspect-aba-must-not-publish");
}

fn require_adjustment_commit(
    outcome: WorthQueryApplicationMutationOutcome<
        worth_query_consumer_values::PlanarMutationDenial,
        worth_query_consumer_values::PlanarAdjustmentResult,
    >,
) {
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "the ABA setup adjustment must publish: {outcome:?}"
    );
}

pub(super) fn require_adjacency_aba_denial(
    request: &Request<'_>,
    application: &ProgramApplication,
) {
    let source = observed_source(request, "anchor-a");
    let outcome = mutate(
        request,
        application,
        same_membership_retarget("anchor-a", "replacement-b"),
        410,
    );
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "same-membership retarget must publish: {outcome:?}"
    );
    require_source_changed(
        request
            .mutate(replacement(
                "replacement-b",
                "adjacency-aba-must-not-publish",
                11,
                1,
            ))
            .expect_source(source)
            .idempotency(&411_u64)
            .execute_in_program(application),
    );
    require_absent(request, "adjacency-aba-must-not-publish");
}

fn same_membership_retarget(
    source: &str,
    target: &str,
) -> worth_query_topology_entry::PlanarMutation {
    worth_query_topology_entry::PlanarMutation {
        scope_key: source.to_owned(),
        operation: worth_query_consumer_values::PlanarOperation::RetargetSuccessor {
            source_key: source.to_owned(),
            previous_target_key: target.to_owned(),
            replacement_target_key: target.to_owned(),
        },
        validator_work: 4096,
    }
}
