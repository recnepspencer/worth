//! Real native-before-custom ordering and the two retained scope boundaries.
use super::*;
use worth_query_consumer_values::PlanarOperation;
type Request<'a, 'p, 's> = WorthQueryApplicationRequest<'a, 'p, 's, ExampleSchema>;
pub(super) struct Controls<'a> {
    pub mode: &'a AtomicUsize,
    pub calls: &'a AtomicUsize,
    pub events: &'a Mutex<Vec<PlanarInvariantReadEvent>>,
}
pub(super) fn verify(
    application: &Application,
    request: &Request<'_, '_, '_>,
    controls: Controls<'_>,
) {
    for (mode, key) in [
        (INVARIANT_PROBE_UNDECLARED_ACCESS, 31_u64),
        (INVARIANT_PROBE_EVALUATION_EXPANSION, 32),
    ] {
        controls.events.lock().unwrap().clear();
        let before = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        let version = before.receipt().inspect().basis().version();
        controls.mode.store(mode, Ordering::SeqCst);
        let outcome = request
            .advanced()
            .mutate(edit(3))
            .expect_source(before.observed_sources()[0].clone())
            .idempotency(&key)
            .execute_performed::<ExampleProgram<false>, Root>(
                application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome) = outcome else {
            panic!("scope escape must not publish")
        };
        let WorthQueryApplicationMutationOutcome::Commit(
            primary_graph::WorthQueryApplicationUncommitted::Denied(denial),
        ) = outcome
        else {
            panic!("the real invariant must refuse")
        };
        assert_eq!(
            denial.kind(),
            primary_graph::WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
        );
        assert_eq!(
            denial.stage(),
            primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution
        );
        assert!(matches!(
            denial.custom_invariant_denial(),
            Some(primary_graph::WorthQueryCustomInvariantDenial::Failure { .. })
        ));
        let events = std::mem::take(&mut *controls.events.lock().unwrap());
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, PlanarInvariantReadEvent::CompleteOrbit { .. })),
            "negative mode prepares no relation scope"
        );
        assert!(events
            .iter()
            .any(|event| if mode == INVARIANT_PROBE_UNDECLARED_ACCESS {
                matches!(
                    event,
                    PlanarInvariantReadEvent::Prepared {
                        succeeded: false,
                        ..
                    }
                )
            } else {
                matches!(
                    event,
                    PlanarInvariantReadEvent::Evaluated {
                        returned_verdict: false,
                        ..
                    }
                )
            }));
        let unchanged = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(unchanged.receipt().inspect().basis().version(), version);
        assert_eq!(unchanged.rows(), before.rows());
    }
    controls
        .mode
        .store(INVARIANT_PROBE_STANDARD, Ordering::SeqCst);
    controls.events.lock().unwrap().clear();
    let before = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    let version = before.receipt().inspect().basis().version();
    let calls = controls.calls.load(Ordering::SeqCst);
    let refused = retarget(application, request, "anchor-c", 41);
    let WorthQueryApplicationMutationOutcome::Commit(
        primary_graph::WorthQueryApplicationUncommitted::Denied(denial),
    ) = refused
    else {
        panic!("actual degree-breaking retarget must reach Native refusal")
    };
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryApplicationCommitDenialKind::ProviderRejected
    );
    assert_eq!(
        denial.stage(),
        primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    assert_eq!(
        controls.calls.load(Ordering::SeqCst),
        calls,
        "Native blocks before custom preparation"
    );
    assert!(controls.events.lock().unwrap().is_empty());
    let unchanged = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(unchanged.receipt().inspect().basis().version(), version);
    assert_eq!(unchanged.rows(), before.rows());
    // Same operation and physical unlink/link shape, preserving degree one.
    let repaired = retarget(application, request, "anchor-b", 42);
    assert!(
        matches!(
            repaired,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{repaired:?}"
    );
    assert!(
        controls.calls.load(Ordering::SeqCst) > calls,
        "lawful ABA reaches the actual custom preparation"
    );
    assert!(controls
        .events
        .lock()
        .unwrap()
        .iter()
        .any(|event| matches!(event, PlanarInvariantReadEvent::CompleteOrbit { .. })));
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-a".into()
            })
            .execute()
            .unwrap()
            .rows(),
        before.rows()
    );
    controls.events.lock().unwrap().clear();
}
fn retarget(
    application: &Application,
    request: &Request<'_, '_, '_>,
    target: &str,
    key: u64,
) -> WorthQueryApplicationMutationOutcome<
    worth_query_consumer_values::PlanarMutationDenial,
    worth_query_consumer_values::PlanarAdjustmentResult,
> {
    let before = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    request
        .advanced()
        .mutate(PlanarEdit(PlanarMutation {
            scope_key: "anchor-a".into(),
            operation: PlanarOperation::RetargetSuccessor {
                source_key: "anchor-a".into(),
                previous_target_key: "anchor-b".into(),
                replacement_target_key: target.into(),
            },
        }))
        .expect_source(before.observed_sources()[0].clone())
        .idempotency(&key)
        .execute_in_program(
            application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
}
