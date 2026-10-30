//! Audit lifecycle coverage is canonical history, not a physical adjacency edit.
use super::*;
use crate::config::data::CascadeDeletePolicy;
use crate::facade::runtime::{
    RelationalAdjacencyDirection as ReadDirection, RelationalAdjacencySupportDenial,
};
use crate::history::data::RelationalDescriptiveTouch as Touch;

fn durable_audit() -> RelationalRuntime {
    durable_runtime_with_declared_aspect_schema(CascadeDeletePolicy::RetainDanglingForAudit)
}

#[test]
fn audit_lifecycle_coverage_survives_unrelated_history_discard_and_durable_recovery() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = durable_audit();
    let a = create_entity(&execution, &runtime, "audit-source");
    let b = create_entity(&execution, &runtime, "audit-target");
    let c = create_entity(&execution, &runtime, "unaffected-source");
    let d = create_entity(&execution, &runtime, "unaffected-target");
    let relation = create_relation(&execution, &runtime, a, b, "audit-edge");
    let stable = create_relation_outcome(&execution, &runtime, c, d, "stable-edge");
    create_branch_from_main(&runtime, "audit-source");
    let source = test_owner_main_basis(&runtime).unwrap();
    let before = runtime
        .read_truth()
        .project_snapshot(&stable.snapshot)
        .unwrap();
    let retired = delete_entity(&execution, &runtime, b);
    let after = runtime
        .read_truth()
        .project_snapshot(&retired.snapshot)
        .unwrap();
    assert!(
        before
            .compare_adjacency_support(&execution, &after, c, KindId(2), ReadDirection::Outgoing)
            .unwrap(),
        "unrelated audit retention preserves the exact live relation population"
    );
    assert_eq!(before.compare_adjacency_support(&execution, &after, a, KindId(2), ReadDirection::Outgoing),
        Err(RelationalAdjacencySupportDenial::UnsupportedVisibility),
        "complete authored history does not turn retained audit visibility into a readable live list");
    let canonical = runtime
        .history_access()
        .demand_commit_envelope(&execution, retired.commit.commit_id)
        .unwrap()
        .unwrap();
    let touches = canonical.descriptive_touches().exact_touches().unwrap();
    assert!(touches.contains(&Touch::RelationLifecycle {
        relation,
        kind: KindId(2)
    }));
    assert!(touches.contains(&Touch::RelationMembership {
        relation,
        kind: KindId(2),
        source: a,
        target: b
    }));
    assert!(
        !touches
            .iter()
            .any(|touch| matches!(touch, Touch::AdjacencyRevision { .. })),
        "retention must not fabricate a physical adjacency delta/revision"
    );
    let digest = canonical.authoritative_payload_digest().unwrap();
    let encoded = canonical.encode_authoritative_payload().unwrap();
    // Grow genuinely unrelated history behind the same fixed selector. Neither
    // its completeness nor its exact occurrence identity may change.
    for index in 0..3 {
        let other = create_entity(&execution, &runtime, &format!("unrelated-{index}"));
        let unrelated = create_relation_outcome(&execution, &runtime, other, d, "outside-selector");
        let grown = runtime
            .read_truth()
            .project_snapshot(&unrelated.snapshot)
            .unwrap();
        assert!(
            before
                .compare_adjacency_support(
                    &execution,
                    &grown,
                    c,
                    KindId(2),
                    ReadDirection::Outgoing
                )
                .unwrap(),
            "unrelated relation population must not change this exact live support"
        );
        release_test_commit_snapshot(&runtime, &unrelated);
    }
    let target = test_owner_main_basis(&runtime).unwrap();
    for (anchor, direction) in [(a, Direction::Outgoing), (b, Direction::Incoming)] {
        assert_eq!(
            scan(&runtime, execution, &source, &target, anchor, direction),
            [retired.commit.commit_id]
        );
    }
    assert!(scan(
        &runtime,
        execution,
        &source,
        &target,
        c,
        Direction::Outgoing
    )
    .is_empty());
    runtime
        .discard_authored_write_generation(execution, selected(&runtime, &target))
        .unwrap();
    runtime
        .rebuild_authored_write_generation(execution, selected(&runtime, &target))
        .unwrap();
    assert_eq!(
        scan(
            &runtime,
            execution,
            &source,
            &target,
            a,
            Direction::Outgoing
        ),
        [retired.commit.commit_id]
    );
    let (_, recovered) = checkpoint_and_recover_with(&execution, &runtime, durable_audit);
    let recovered_source = recovered
        .admit_branch_basis(
            &recovered
                .branch_identity(&BranchId("audit-source".into()))
                .unwrap(),
        )
        .unwrap();
    let recovered_target = test_owner_main_basis(&recovered).unwrap();
    recovered
        .rebuild_authored_write_generation(execution, selected(&recovered, &recovered_target))
        .unwrap();
    for (anchor, direction) in [(a, Direction::Outgoing), (b, Direction::Incoming)] {
        assert_eq!(
            scan(
                &recovered,
                execution,
                &recovered_source,
                &recovered_target,
                anchor,
                direction
            ),
            [retired.commit.commit_id]
        );
    }
    assert!(scan(
        &recovered,
        execution,
        &recovered_source,
        &recovered_target,
        c,
        Direction::Outgoing
    )
    .is_empty());
    let cold = recovered
        .history_access()
        .demand_commit_envelope(&execution, retired.commit.commit_id)
        .unwrap()
        .unwrap();
    assert_eq!(cold.descriptive_touches(), canonical.descriptive_touches());
    assert_eq!(cold.authoritative_payload_digest().unwrap(), digest);
    assert_eq!(cold.encode_authoritative_payload().unwrap(), encoded);
    for outcome in [&stable, &retired] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}
