//! Public entry routing retains logical identity and original physical dependencies.
pub use super::super::{
    installation::{install, World},
    outcome, read,
};
pub use super::*;
pub use crate::facade::certification::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use std::{collections::BTreeSet, marker::PhantomData};
pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::WorthQueryApplicationUncommitted,
    runtime::ExecutionAllocationPolicy as Allocation,
};
pub fn checked<L: LawPolicy>(world: &World<L>, command: u64, root: u64) -> operation::Checked {
    let scope = request_scope();
    let principal = authenticate_operator(world.application.installed_schema(), &scope);
    let outcome = world
        .application
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.application.current_world())
        .advanced()
        .mutate(operation::Intent::<L> {
            input: operation::Input { root },
            marker: PhantomData,
        })
        .without_source()
        .idempotency(&command)
        .execute_in_program(&world.application, Allocation::SystemAllocation)
        .unwrap();
    outcome
        .result()
        .unwrap_or_else(|| panic!("logical decision must complete: {outcome:?}"))
        .clone()
}
pub fn count(world: &World<Logical>, command: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(world.application.installed_schema(), &scope);
    let outcome = world
        .application
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.application.current_world())
        .advanced()
        .mutate(derived::CountIntent::<Logical> {
            input: derived::CountInput {
                root: 0,
                bad_reference: false,
            },
            marker: PhantomData,
        })
        .without_source()
        .idempotency(&command)
        .execute_in_program(&world.application, Allocation::SystemAllocation)
        .unwrap();
    assert_eq!(
        outcome.result().map(|value| value.count),
        Some(2),
        "{outcome:?}"
    );
}
pub fn assert_pair(world: &World<Logical>, root: u64, successor: u64, answer: &operation::Checked) {
    let graph = read::committed(world);
    assert!(graph.edges.contains(&(root, successor)));
    assert_eq!(
        answer.members.iter().cloned().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            (graph.identities[&root], "Negative".into()),
            (graph.identities[&successor], "Positive".into())
        ])
    );
    assert_eq!(answer.distances, [0, 1]);
}
