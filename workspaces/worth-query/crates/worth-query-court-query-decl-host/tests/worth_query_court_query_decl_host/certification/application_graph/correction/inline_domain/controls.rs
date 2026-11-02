//! The actual retained Native pin borrows its caller's Work and cancellation.
use super::*;
use std::{cell::Cell, num::NonZeroUsize};
use worth_execution::{CancellationSource, ExecutionRequest, ExecutionWorkCeiling, LeaseRequest};
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

#[test]
fn inline_domain_at_its_wire_bound_uses_original_native_work_and_cancellation() {
    let (application, authority) = host::install_with_projection();
    let runtime = application.runtime();
    let scope = request_scope();
    let projected = runtime
        .with_application_advancement(&scope, |phase| {
            authority.project(
                &phase,
                |reader| {
                    reader
                        .resolve_entity(Key::reference(), "numeric".to_owned())
                        .unwrap()
                },
                ExecutionAllocationPolicy::SystemAllocation,
            )
        })
        .unwrap()
        .unwrap();
    let (identity, snapshot, _) = projected.into_parts();
    let version = snapshot.version();
    let original = CertifiedObservation::capture(&application, &authority, "numeric");
    assert_wire(&original.primary(&application), &FIVE);
    runtime.with_application_advancement(&scope, |phase| {
        let execution = phase.checked_execution_request_for(runtime.product_runtime()).unwrap();
        let run = |work, cancel_at_handoff| {
            let cancellation = CancellationSource::new();
            let lease = execution.in_scope(|parent| {
                let parent = parent.expect("the installed host lends its actual request lease");
                parent.child(LeaseRequest {
                    policy: ExecutionRequestPolicy::new(parent.policy().posture(), parent.policy().determinism(), ExecutionBudget::new(NonZeroUsize::MIN, Some(64 * 1024 * 1024), Some(1_000_000))),
                    cancellation: cancellation.token(), deadline: scope.deadline(),
                }).unwrap()
            }).unwrap();
            let request = ExecutionRequest::leased(&lease);
            let observed = Cell::new(None);
            let returned = Cell::new(false);
            let (result, report) = request.run(ExecutionWorkCeiling::new(Some(work)), |_| {
                snapshot.read_fields(&request, |reader| {
                    let value = reader.field(&identity, Primary::reference());
                    observed.set(value.as_ref().err().map(|denial| denial.kind()));
                    if cancel_at_handoff { cancellation.cancel(); }
                    returned.set(true);
                    // Deliberately swallow the field refusal: completion must veto it.
                    value.ok().flatten()
                })
            }).unwrap();
            (result, observed.get(), returned.get(), report)
        };
        let (funded, observed, returned, report) = run(1_000_000, false);
        assert_eq!(funded.unwrap(), Some(5));
        assert_eq!(observed, None);
        assert!(returned);
        // One work unit below the measured actual bill refuses through the
        // original execution request. No replacement allowance can complete it.
        let narrow = report.charged_work().checked_sub(1).unwrap();
        let (refused, observed, returned, _) = run(narrow, false);
        let expected = WorthQueryInvariantProjectionDenialKind::ExecutionRequest(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted);
        assert_eq!(observed, Some(expected));
        assert_eq!(refused.unwrap_err().kind(), expected);
        assert!(returned);
        let (cancelled, observed, returned, _) = run(1_000_000, true);
        assert_eq!(observed, None);
        assert!(returned);
        assert_eq!(cancelled.unwrap_err().kind(), WorthQueryInvariantProjectionDenialKind::ExecutionRequest(worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled));
        let (retried, _, _, retry_report) = run(1_000_000, false);
        assert_eq!(retried.unwrap(), Some(5));
        assert_eq!(retry_report.charged_work(), report.charged_work());
        assert_eq!(snapshot.version(), version);
        println!("inline native wire bound=9 original_work={} refused_at={narrow}", report.charged_work());
    }).unwrap();
    assert_eq!(host::read(&application)[0].primary, 5);
    assert_wire(&original.primary(&application), &FIVE);
}
