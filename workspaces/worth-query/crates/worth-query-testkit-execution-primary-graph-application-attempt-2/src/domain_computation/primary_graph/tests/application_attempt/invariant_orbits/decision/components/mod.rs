//! Original admitted roots, whole Native cohorts, sealing and physical invalidation.

use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::components::binding::Empty;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::components::binding::Shells;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::component_selection::ComponentSelection;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::components::binding::*;
use std::sync::Arc;
use worth_execution::ExecutionAllocationPolicy as Allocation;
pub struct Read {
    pub body: Arc<ComponentSelection>,
    pub lease: WorthQueryApplicationSnapshotLease,
}
#[derive(Clone, Copy)]
pub enum Selection {
    Empty,
    Shells,
}
pub fn read_selection(
    world: &crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::World,
    root: u64,
    selection: Selection,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Read {
    world
        .application
        .with_host_advancement(|phase| {
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
                CheckComponents::reference(),
            );
            let (_, snapshot, _) = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, _| match selection {
                        Selection::Empty => {
                            assert_eq!(reader.component_count(Empty::reference()).unwrap(), 0)
                        }
                        Selection::Shells => {
                            assert_eq!(reader.component_count(Shells::reference()).unwrap(), 1)
                        }
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let sealed = world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    snapshot,
                    Allocation::SystemAllocation,
                )
                .unwrap()
                .complete_projected_dependencies(&phase, Allocation::SystemAllocation)
                .unwrap();
            let body = sealed
                .decision_facts()
                .into_iter()
                .find_map(|fact| match fact {
                    WorthQueryApplicationObservedFact::ComponentSelection { observation } => {
                        Some(observation)
                    }
                    _ => None,
                })
                .unwrap();
            // A real independently issued selected lease, not a reconstructed snapshot.
            let selected = world
                .application
                .select_product_branch_for_test(
                    &execution,
                    world.application.product_runtime().default_branch_for_test(),
                )
                .unwrap();
            let projection = world
                .invariant
                .project(&phase, |_| (), Allocation::SystemAllocation)
                .unwrap();
            let (_, snapshot, _) = projection.into_parts();
            let (lease, _, _, _) =
                snapshot.into_lease_and_realized_scope_for_test(selected.product().retained_clone_for_test());
            Read { body, lease }
        })
        .unwrap()
}

#[path = "binding/mod.rs"]
pub mod binding;
#[path = "handler/mod.rs"]
pub mod handler;
