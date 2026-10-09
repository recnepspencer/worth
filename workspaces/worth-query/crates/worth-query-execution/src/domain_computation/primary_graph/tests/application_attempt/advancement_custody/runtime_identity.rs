//! A public phase cannot fund another installation's reads or mutation.
use super::*;

#[test]
fn foreign_runtime_phase_refuses_query_and_commit_before_source_read() {
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let a = installed_authorization_world(true);
        let b = installed_authorization_world(true);
        let request = live_scope();
        let a_principal = authenticated_principal(&a, &request);
        let b_principal = authenticated_principal(&b, &request);
        let a_account = resolved_account(&a, "open", &request);
        let b_account = resolved_account(&b, "open", &request);
        let a_program = admitted_program(&a, &a_principal, &a_account, &request, "closed");
        let b_program = admitted_program(&b, &b_principal, &b_account, &request, "closed");
        let a_product = a.selected_product();
        let b_product = b.selected_product();
        reports();
        a.application.with_application_advancement(&request, |phase| {
            let before = reads();
            let denial = b_product.require_current_output_settlements(
                &phase,
                std::iter::empty(),
                NonZeroUsize::MIN,
            ).unwrap_err();
            assert_eq!(
                denial.kind(),
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ExecutionRequest(Denial::ForeignPhase),
            );
            assert_eq!(reads(), before, "foreign query cannot reach B's reader");
            a_product.require_current_output_settlements(
                &phase,
                std::iter::empty(),
                NonZeroUsize::MIN,
            ).unwrap();
            assert!(reads() > before, "A's same query door reaches its reader");
            let before = reads();
            let refused = b.application.compare_and_commit_application_in_advancement(
                &phase, b_program, idempotency(179, 179),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            );
            let WorthQueryApplicationCommitOutcome::Denied(denial) = refused else {
                panic!("foreign commit must refuse before revalidation");
            };
            assert!(matches!(
                denial.kind(),
                WorthQueryApplicationCommitDenialKind::ExecutionResource {
                    denial: Resource::ForeignAdvancementPhase,
                    partition_identity: None,
                    policy_ancestor: None,
                }
            ));
            assert_eq!(reads(), before, "foreign mutation cannot reach B's reader");
            let admitted = a.application.compare_and_commit_application_in_advancement(
                &phase, a_program, idempotency(178, 178),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            );
            assert!(matches!(admitted, WorthQueryApplicationCommitOutcome::Committed(_)));
            assert!(reads() > before, "A's same commit door reaches its reader");
        }).unwrap();
        let requests = reports();
        assert_eq!(requests.len(), 1, "B never opens or charges a request");
        assert_eq!(
            requests[0].as_ref().unwrap().charged_work(),
            0,
            "these inline reads and mutations charge no execution work in part one"
        );
    }
}

#[test]
fn mutable_principal_reader_moves_the_installed_source_counter() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", std::time::Duration::from_secs(60), &request);
    let selected = world.selected_product();
    world
        .application
        .with_application_advancement(&request, |_phase| {
            let before = reads();
            selected.resolve_authenticated_principal(
            &world.binding, &external, &request,
            crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        ).unwrap();
            assert_eq!(
                reads(),
                before + 1,
                "principal resolution enters its mutable source door once"
            );
        })
        .unwrap();
    // This lower public read door takes no phase yet; it closes in part two.
    // The assertion pins the observer's actual read boundary, not that carriage.
}
