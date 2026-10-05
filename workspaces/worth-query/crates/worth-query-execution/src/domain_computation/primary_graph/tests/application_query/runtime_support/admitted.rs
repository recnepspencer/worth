use worth_query_admission::facade::{
    application_query::{admit_application_query_parameters, WorthQueryApplicationQueryLane},
    graph_read_access::WorthQueryGraphIndexInventoryAdmissionStop,
};
use worth_query_admission::integration::derive_graph_read_access_requirements_for_contract;

use super::super::super::fixture::{installed_authorization_world, live_account_parameters};
use crate::domain_computation::primary_graph::application_query::{
    primary_graph_support_inventory, primary_graph_support_inventory_admitted,
};

#[test]
fn real_layout_admitted_inventory_matches_ordinary_and_refuses_one_short() {
    let world = installed_authorization_world(true);
    let query = super::super::installed_live_query(&world);
    let parameters =
        admit_application_query_parameters(&query, live_account_parameters("account-1")).unwrap();
    let requirements = derive_graph_read_access_requirements_for_contract(
        query.read_family_binding().planning_contract(),
        WorthQueryApplicationQueryLane::Continuation,
        4,
        parameters.identity(),
        query.canonical_work_policy().admission_planning(),
    )
    .unwrap();
    let graph = world.application.runtime.primary_graph().unwrap();
    let ordinary = primary_graph_support_inventory(
        &graph.layout,
        query.continuation(),
        query.live(),
        &requirements,
    );

    let mut work = 0_u64;
    let admitted = primary_graph_support_inventory_admitted(
        &graph.layout,
        query.continuation(),
        query.live(),
        &requirements,
        &mut |charge, _| {
            work = work.checked_add(charge).unwrap();
            Ok::<(), ()>(())
        },
    )
    .unwrap();
    assert!(!admitted.rows().is_empty());
    assert_eq!(admitted, ordinary);

    let limit = work
        .checked_sub(1)
        .expect("real support traversal spends Work");
    let mut spent = 0_u64;
    let refusal = primary_graph_support_inventory_admitted(
        &graph.layout,
        query.continuation(),
        query.live(),
        &requirements,
        &mut |charge, _| {
            let next = spent.checked_add(charge).ok_or("overflow")?;
            if next > limit {
                return Err("one-short");
            }
            spent = next;
            Ok(())
        },
    );
    assert!(matches!(
        refusal,
        Err(WorthQueryGraphIndexInventoryAdmissionStop::Admission(
            "one-short"
        ))
    ));
}
