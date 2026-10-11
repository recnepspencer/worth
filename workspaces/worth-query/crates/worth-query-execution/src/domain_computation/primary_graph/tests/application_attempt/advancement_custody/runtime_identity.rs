//! A public phase cannot fund another installation's reads or mutation.
use super::*;

#[test]
fn foreign_runtime_phase_refuses_query_and_commit_before_source_read() {
    // Two changed rows (account and idempotency), 21 selected/reserved indexes,
    // eight before/after reads, twelve routing visits, three edits, one old-row
    // comparison, and three nonempty plus three empty retained paths.
    let index_work = 2 * 21 + 2 + 8 + 12 + 3 + 1 + (3 * 64 + 3);
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let mut control_work = None;
        for foreign in [false, true] {
            let a = installed_authorization_world(true);
            let b = installed_authorization_world(true);
            let request = live_scope();
            let a_principal = a
                .application
                .with_host_advancement(|phase| {
                    authenticated_principal(&phase.test_host_request(), &a, &request)
                })
                .expect("installed host call opens");
            let b_principal = b
                .application
                .with_host_advancement(|phase| {
                    authenticated_principal(&phase.test_host_request(), &b, &request)
                })
                .expect("installed host call opens");
            let a_account = a
                .application
                .with_host_advancement(|phase| {
                    resolved_account(&phase.test_host_request(), &a, "open", &request)
                })
                .expect("installed host call opens");
            let b_account = b
                .application
                .with_host_advancement(|phase| {
                    resolved_account(&phase.test_host_request(), &b, "open", &request)
                })
                .expect("installed host call opens");
            let a_program = a
                .application
                .with_host_advancement(|phase| {
                    admitted_program(&phase, &a, &a_principal, &a_account, &request, "closed")
                })
                .expect("installed host call opens");
            let b_program = b
                .application
                .with_host_advancement(|phase| {
                    admitted_program(&phase, &b, &b_principal, &b_account, &request, "closed")
                })
                .expect("installed host call opens");
            let a_product = a
                .application
                .with_host_advancement(|phase| a.selected_product(&phase.test_host_request()))
                .expect("installed host call opens");
            let b_product = b
                .application
                .with_host_advancement(|phase| b.selected_product(&phase.test_host_request()))
                .expect("installed host call opens");
            reports();
            a.application.with_application_advancement(&request, |phase| {
            let before = reads();
            if foreign {
            let denial = b_product.require_current_output_settlements(
                &phase,
                std::iter::empty(),
                NonZeroUsize::MIN,
            ).unwrap_err();
            assert_eq!(
                denial.kind(),
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ExecutionRequest(Denial::Resource(crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::ForeignAdvancementPhase)),
            );
            assert_eq!(reads(), before, "foreign query cannot reach B's reader");
            }
            a_product.require_current_output_settlements(
                &phase,
                std::iter::empty(),
                NonZeroUsize::MIN,
            ).unwrap();
            assert!(reads() > before, "A's same query door reaches its reader");
            let before = reads();
            if foreign {
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
            }
            let admitted = a.application.compare_and_commit_application_in_advancement(
                &phase, a_program, idempotency(178, 178),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            );
            let WorthQueryApplicationCommitOutcome::Committed(receipt) = &admitted else {
                panic!("the same-installation candidate must commit: {admitted:?}");
            };
            let evidence = receipt.mutation_work().unwrap();
            let work = evidence.index_maintenance_work();
            assert_eq!((work.patch_records, work.generation_publications_reserved), (2, 21));
            assert_eq!(work.work_units as u64, index_work);
            assert!(reads() > before, "A's same commit door reaches its reader");
        }).unwrap();
            let requests = reports();
            assert_eq!(requests.len(), 1, "B never opens or charges a request");
            let charged = requests[0].as_ref().unwrap().charged_work();
            assert!(
                charged >= index_work,
                "the aggregate includes A's exact native index work"
            );
            if foreign {
                assert_eq!(
                    Some(charged),
                    control_work,
                    "foreign doors charge no work beside the real same-installation control"
                );
            } else {
                control_work = Some(charged);
            }
        }
    }
}

#[test]
fn mutable_principal_reader_moves_the_installed_source_counter() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", std::time::Duration::from_secs(60), &request);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request()))
        .expect("installed host call opens");
    world
        .application
        .with_application_advancement(&request, |_phase| {
            let before = reads();
            selected.resolve_authenticated_principal(
            &_phase, &world.binding, &external, &request,
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
