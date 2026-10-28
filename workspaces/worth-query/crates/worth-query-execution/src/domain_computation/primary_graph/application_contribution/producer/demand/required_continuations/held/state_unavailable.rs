//! A released typed successor refuses at the producer's resume boundary.

use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, retained_output_capacity::RetainedFamily,
    IdentityExecutionSchema,
};
use std::{cell::RefCell, time::Duration};

#[test]
fn absent_admitted_demand_refuses_with_required_successor_state_unavailable() {
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let principal = world.authenticate("alice", Duration::from_secs(60), &scope);
    // This is the emptied state after custody releases its admitted demand.
    // No fabricated interest, provenance or authority participates in the call.
    let mut successor = TypedRequiredSuccessor::<IdentityExecutionSchema, RetainedFamily> {
        demand: None,
        pending_claim: None,
        pending_provenance: None,
        predecessor: None,
    };
    world
        .application
        .with_host_advancement(|phase| {
            let runtime = &world.application;
            let execution = phase.execution_request_for(runtime.product_runtime()).unwrap();
            let denial = successor
                .run_again(
                    &execution,
                    &phase,
                    runtime,
                    &principal,
                    &scope,
                    runtime.current_world(),
                    &RefCell::new(PerformedMembers::start()),
                )
                .err()
                .expect("an absent successor demand must refuse");
            assert_eq!(
                denial.kind(),
                super::super::super::WorthQueryOutputDemandDenialKind::RequiredSuccessorStateUnavailable
            );
        })
        .expect("the installed host admits the resume call");
}
