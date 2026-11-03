//! Retained authenticated handler candidates consume both actual source facts.

use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::binding::CheckOrbitInput;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::binding::CheckOrbitOutcome;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::OrbitSchema;

use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::World;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::binding::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity;
use worth_query_execution::facade::domain_computation::HandlerResult;
use worth_execution::ExecutionRequest;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
use worth_query_execution::facade::domain_computation::WorthQueryMutationHandlerWork;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_entry;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::root;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
pub type Schema = OrbitSchema<true>;
pub type Binding = CheckOrbitBinding<Schema>;
pub type Program = WorthQueryApplicationEffectProgram<Schema, CheckOrbit, CheckOrbitInput, Node>;
pub fn prepare(
    world: &World,
    request: &WorthQueryRequestScope,
    root: u64,
    key: &str,
) -> (Program, WorthQueryApplicationIdempotencyBinding) {
    world
        .application
        .with_application_advancement(request, |phase| {
            let host = phase.test_host_request_for_test();
            let execution = host
                .execution_request_for_for_test(world.application.product_runtime())
                .unwrap();
            let (admission, _) = crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::authorize(
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
pub fn publish(
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
