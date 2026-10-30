//! Fresh authority and resource failures precede access to an actual retained evaluation.
use super::super::super::authentication::*;
use super::super::{
    distinct_subjects::{self, observation, query},
    journey as source_journey, SourceDisclosure,
};
use super::demand;
use super::journey::install;
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, causal_inspection::*,
    primary_graph::*, runtime::*,
};

pub fn evaluation_rechecks_each_subject_and_releases_request_funding() {
    for withdrawn in ["inspection-grant", "observer-decoy-inspection-grant"] {
        let application = install();
        let scope = request_scope();
        let principal = authenticate_operator(application.installed_schema(), &scope);
        let (source, _, source_read) =
            source_journey::perform(&application, 4, "evaluation-lifecycle-source");
        drop(source_read);
        let (_, read, contacts) = demand::settle(&application);
        assert_eq!(contacts, 1);
        let (original, _) =
            observation::read(&application, &principal, &scope, &read, "observer-decoy");
        drop(read);
        let readers = application.application_query_basis_observer();
        let baseline = readers.observe().active();
        let access = |subject| distinct_subjects::access(&application, &principal, &scope, subject);
        let cancellation = WorthQueryCancellationSource::new();
        let cancelled = WorthQueryRequestScope::new(None, cancellation.token());
        let (qm, qd, sm, sd) = (
            access("observer-decoy"),
            access("observer-decoy"),
            access("numeric"),
            access("numeric"),
        );
        cancellation.cancel();
        let denial = application
            .runtime()
            .request(&principal, &cancelled)
            .inspect::<query::Binding>(&original)
            .why_observed()
            .evaluated_by::<SourceDisclosure>(&source)
            .reference_only()
            .execute_governed(qm, qd, sm, sd)
            .err()
            .expect("actual cancellation refuses");
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationInspectionStopKind::Request
        );
        assert!(denial.producer_evaluation_refusal().is_none());
        drop(denial);
        let authority = ExecutionAuthority::serial();
        for bytes in [0, 32 * 1024 * 1024] {
            let lease = authority
                .request_lease(LeaseRequest {
                    policy: ExecutionRequestPolicy::new(
                        ExecutionPosture::Serial,
                        DeterminismContract::CanonicalBitwise,
                        worth_foundational::ExecutionBudget::new(
                            std::num::NonZeroUsize::MIN,
                            Some(bytes),
                            Some(100_000_000),
                        ),
                    ),
                    deadline: None,
                    cancellation: CancellationToken::new(),
                })
                .unwrap();
            let before = lease.charged_memory_bytes();
            let result = application
                .runtime()
                .request(&principal, &scope)
                .inspect::<query::Binding>(&original)
                .why_observed()
                .evaluated_by::<SourceDisclosure>(&source)
                .reference_only()
                .with_allocation_policy(ExecutionAllocationPolicy::Execution(&lease))
                .execute_governed(
                    access("observer-decoy"),
                    access("observer-decoy"),
                    access("numeric"),
                    access("numeric"),
                );
            if bytes == 0 {
                let denial = result
                    .err()
                    .expect("zero retained-memory budget cannot fund an evaluation report");
                assert_eq!(
                    denial.kind(),
                    WorthQueryApplicationInspectionStopKind::Request
                );
                assert!(denial.producer_evaluation_refusal().is_none());
                assert!(denial.resource_refusal().is_some());
                drop(denial);
            } else {
                let report = result.unwrap();
                assert!(lease.charged_memory_bytes() > before);
                assert_eq!(
                    report.producer_evaluation().source_commit(),
                    source.commit_reference()
                );
                report.close();
            }
            assert_eq!(
                lease.charged_memory_bytes(),
                before,
                "all request reservations release on report close or denial"
            );
            assert_eq!(readers.observe().active(), baseline);
        }
        let (qm, qd, sm, sd) = (
            access("observer-decoy"),
            access("observer-decoy"),
            access("numeric"),
            access("numeric"),
        );
        let outcome = application
            .runtime()
            .request(&principal, &scope)
            .advanced()
            .mutate(distinct_subjects::grant_status::Withdraw {
                grant: withdrawn.to_owned(),
            })
            .without_source()
            .idempotency(&format!("evaluation-withdraw-{withdrawn}"))
            .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        let denial = application
            .runtime()
            .request(&principal, &scope)
            .inspect::<query::Binding>(&original)
            .why_observed()
            .evaluated_by::<SourceDisclosure>(&source)
            .reference_only()
            .execute_governed(qm, qd, sm, sd)
            .err()
            .expect("either subject's actual withdrawal refuses");
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationInspectionStopKind::Admission
        );
        assert!(denial.producer_evaluation_refusal().is_none());
        assert!(denial.producer_connection_refusal().is_none());
        drop(denial);
        original.close();
        assert_eq!(readers.observe().active(), baseline);
    }
}
