//! Retained authenticated handler candidates consume both actual source facts.
use super::{
    super::{program, schema::*, World},
    binding::*,
    identity,
};
use crate::domain_computation::primary_graph::{application_entry::mutation::HandlerResult, *};
use worth_execution::{ExecutionAllocationPolicy as Allocation, ExecutionRequest};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
type Schema = OrbitSchema<true>;
type Binding = CheckOrbitBinding<Schema>;
type Program = WorthQueryApplicationEffectProgram<Schema, CheckOrbit, CheckOrbitInput, Node>;

fn prepare(
    world: &World,
    request: &WorthQueryRequestScope,
    root: u64,
    key: &str,
) -> (Program, WorthQueryApplicationIdempotencyBinding) {
    world
        .application
        .with_application_advancement(request, |phase| {
            let host = phase.test_host_request();
            let execution = host
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            let (admission, _) = program::authorize(
                world,
                &phase,
                &execution,
                request,
                root,
                CheckOrbit::reference(),
            );
            let input = CheckOrbitInput { root };
            let key_text = key.to_owned();
            let identities =
                ApplicationMutationIdentities::<Schema, Binding>::encode(&key_text, &input)
                    .unwrap();
            let idempotency =
                WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities);
            let report = world
                .application
                .execute_mutation_handler_report::<Binding>(
                    &phase,
                    &identities,
                    &1_u64,
                    admission,
                    Allocation::SystemAllocation,
                );
            let WorthQueryMutationHandlerWork::Captured(work) = report.decision_work() else {
                panic!("actual handler contact required")
            };
            assert!(work.handler_contacted());
            let HandlerResult::Completed(completed) = report.into_outcome().unwrap() else {
                panic!("actual candidate must complete")
            };
            let (program, result) = completed.into_parts();
            assert_eq!(result, CheckOrbitOutcome(if root < 6 { 3 } else { 2 }));
            (program, idempotency)
        })
        .unwrap()
}
fn publish(
    world: &World,
    request: &WorthQueryRequestScope,
    program: Program,
    key: WorthQueryApplicationIdempotencyBinding,
) -> WorthQueryApplicationCommitOutcome {
    world
        .application
        .with_application_advancement(request, |phase| {
            world
                .application
                .compare_and_commit_application_in_advancement(
                    &phase,
                    program,
                    key,
                    Allocation::SystemAllocation,
                )
        })
        .unwrap()
}
#[test]
fn retained_orbit_candidate_stales_after_identity_change_and_commits_after_unrelated_edit() {
    let _isolation = super::super::super::super::fixture::isolated_request_owner();
    for changed in [true, false] {
        let world = identity::install();
        let host = crate::test_request_policy::serial_request();
        let execution = ExecutionRequest::serial(&host);
        let groups = [
            identity::read(&world, 0, &execution),
            identity::read(&world, 3, &execution),
        ];
        let loser = if groups[0].body.native().anchor() < groups[1].body.native().anchor() {
            3
        } else {
            0
        };
        let request = super::super::super::super::fixture::live_scope();
        let (candidate, key) = prepare(&world, &request, loser, "retained-orbit");
        let before = world.committed_version();
        if changed {
            for key in [233, 234] {
                let change = program::swap_successors(&world, &request, 0, 3);
                assert!(matches!(
                    super::super::commit(&world, &request, change, key),
                    WorthQueryApplicationCommitOutcome::Committed(_)
                ));
            }
            let current = identity::read(&world, loser, &execution);
            assert_ne!(
                groups[usize::from(loser == 3)].body.native().identity(),
                current.body.native().identity()
            );
        } else {
            let (other, idempotency) = prepare(&world, &request, 6, "unrelated-mark");
            assert!(matches!(
                publish(&world, &request, other, idempotency),
                WorthQueryApplicationCommitOutcome::Committed(_)
            ));
        }
        assert_ne!(
            world.committed_version(),
            before,
            "the ordinary edit really moved the head"
        );
        let outcome = publish(&world, &request, candidate, key);
        if changed {
            let WorthQueryApplicationCommitOutcome::Stale(stale) = outcome else {
                panic!("both complete facts must be stale: {outcome:?}")
            };
            assert_eq!(
                stale.stale_fact_count(),
                2,
                "repeated orbit/orbit_of retains one aggregate and one walk"
            );
        } else {
            assert!(matches!(
                outcome,
                WorthQueryApplicationCommitOutcome::Committed(_)
            ));
        }
    }
}
