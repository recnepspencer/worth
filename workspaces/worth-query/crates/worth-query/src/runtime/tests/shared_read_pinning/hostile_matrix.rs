use crate::application::WorthQuerySharedReadPinningBoundaryPosture;

use super::*;

#[test]
fn shared_read_pinning_boundary_closes_only_with_all_phase_thirteen_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, derived) =
        shared_read_pinning_workspace(request_execution, "shared-read.phase13.matrix");
    let stable = workspace
        .shared_read_context(request_execution)
        .expect("stable context should mint");
    let stable_basis = stable.inspect_basis();
    let stable_binding = stable
        .published_derived_artifact(&derived)
        .expect("stable artifact should resolve")
        .published_binding()
        .expect("stable artifact should be published")
        .binding_for_reporting()
        .to_string();
    let stable_title = consume_display_title(
        &stable
            .published_derived_artifact(&derived)
            .expect("stable artifact should consume before pressure"),
    );

    for index in 0..4 {
        insert_task(
            request_execution,
            &mut workspace,
            &format!("task-phase13-{index}"),
            if index % 2 == 0 {
                "Task Two"
            } else {
                "Task One"
            },
        );
        let current = workspace
            .shared_read_context(request_execution)
            .expect("current context should mint under commit pressure");
        assert!(current.published_derived_artifact(&derived).is_ok());
        assert_eq!(
            stable
                .published_derived_artifact(&derived)
                .expect("stable context remains legal until explicit invalidation")
                .published_binding()
                .expect("stable artifact remains published")
                .binding_for_reporting(),
            stable_binding
        );
        assert_eq!(
            consume_display_title(
                &stable
                    .published_derived_artifact(&derived)
                    .expect("stable context should keep old facts")
            ),
            stable_title
        );
    }

    let portability = shared_read_portability_evidence(&stable, &derived, &stable_binding);
    workspace
        .runtime
        .invalidate_shared_read_snapshot_for_certification(stable_basis.snapshot_identity());
    let stale_denial = shared_read_stale_denial_evidence(&stable, &derived, &stable_basis);
    drop(stable);
    insert_task(
        request_execution,
        &mut workspace,
        "task-phase13-drain",
        "Task Two",
    );

    let inventory = shared_read_pinning_inventory_evidence();
    let counters = shared_read_counter_evidence(&workspace.runtime);
    let hostile_matrix = WorthQuerySharedReadPinningHostileMatrixEvidence::new(
        true,
        evidence_digest(
            "shared-read-pinning-hostile-matrix",
            [
                (
                    "stable_basis",
                    stable_basis
                        .snapshot_evidence_identity()
                        .as_str()
                        .to_string(),
                ),
                ("stable_binding", stable_binding),
                ("stable_title", stable_title),
                ("counter_digest", counters.counter_digest().to_string()),
            ],
        ),
    );
    let certification = WorthQuerySharedReadPinningCertification::from_evidence(
        inventory,
        hostile_matrix,
        portability,
        stale_denial,
        counters,
    );
    let closure = certification.closure();

    assert_eq!(
        closure.posture(),
        WorthQuerySharedReadPinningBoundaryPosture::Closed
    );
    assert_eq!(closure.inventory_failure_count(), 0);
    assert_eq!(closure.counter_residue_count(), 0);
    assert!(closure.hostile_matrix_green());
    assert!(closure.send_sync_proven());
    assert!(closure.stale_basis_denial_proven());
    assert!(!closure.closure_digest().is_empty());
    assert!(!certification.artifact_digest().is_empty());
    assert!(!certification.failure_digest().is_empty());
    assert!(pinning_phase_twelve_counters_are_closed(&workspace.runtime));
}

fn shared_read_portability_evidence(
    stable: &crate::runtime::WorthQuerySharedReadContext,
    derived: &crate::runtime::WorthQueryDerivedViewHandle<
        crate::runtime::WorthQueryUnrefinedLiveShape,
    >,
    stable_binding: &str,
) -> WorthQuerySharedReadPortabilityEvidence {
    let sibling = stable.clone();
    let (first_binding, second_binding) = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            stable
                .published_derived_artifact(derived)
                .expect("stable context should resolve in first scoped thread")
                .published_binding()
                .expect("stable artifact should be published")
                .binding_for_reporting()
                .to_string()
        });
        let second = scope.spawn(|| {
            sibling
                .published_derived_artifact(derived)
                .expect("stable clone should resolve in second scoped thread")
                .published_binding()
                .expect("stable clone artifact should be published")
                .binding_for_reporting()
                .to_string()
        });
        (
            first.join().expect("first scoped thread should finish"),
            second.join().expect("second scoped thread should finish"),
        )
    });
    assert_eq!(first_binding, stable_binding);
    assert_eq!(second_binding, stable_binding);
    WorthQuerySharedReadPortabilityEvidence::proven(evidence_digest(
        "shared-read-pinning-matrix-portability",
        [
            ("stable_binding", stable_binding.to_string()),
            ("first_binding", first_binding),
            ("second_binding", second_binding),
        ],
    ))
}

fn shared_read_stale_denial_evidence(
    stable: &crate::runtime::WorthQuerySharedReadContext,
    derived: &crate::runtime::WorthQueryDerivedViewHandle<
        crate::runtime::WorthQueryUnrefinedLiveShape,
    >,
    stable_basis: &crate::runtime::WorthQuerySharedReadBasisInspection,
) -> WorthQuerySharedReadStaleBasisDenialEvidence {
    let error = stable
        .published_derived_artifact(derived)
        .expect_err("invalidated matrix context must fail typed");
    match error {
        crate::runtime::WorthQueryRuntimeError::SharedReadStaleBasis { snapshot_identity } => {
            assert_eq!(&snapshot_identity, stable_basis.snapshot_identity());
            WorthQuerySharedReadStaleBasisDenialEvidence::proven(evidence_digest(
                "shared-read-pinning-matrix-stale-denial",
                [
                    (
                        "snapshot_identity",
                        snapshot_identity.evidence_identity().as_str().to_string(),
                    ),
                    (
                        "basis_identity",
                        stable_basis
                            .snapshot_evidence_identity()
                            .as_str()
                            .to_string(),
                    ),
                ],
            ))
        }
        other => panic!("expected shared-read stale basis error, got {other:?}"),
    }
}
