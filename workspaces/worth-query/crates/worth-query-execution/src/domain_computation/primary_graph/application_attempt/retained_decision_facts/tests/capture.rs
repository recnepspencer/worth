//! Real installed projection exercises the retained-key owner. No commit or
//! downstream heap-closure claim is made by this proof.
use super::custody::{authority, isolated, request, SERIAL};
use crate::domain_computation::primary_graph::{
    tests::{
        application_attempt::{authenticated_principal, resolved_account},
        fixture::{
            installed_authorization_world, live_scope, AccountStatus, TouchAccountOperation,
        },
    },
    WorthQueryInvariantProjectionDenialKind,
};
use worth_execution::{
    ExecutionAllocationDenialKind, ExecutionAllocationPolicy as Policy, LeaseDenial,
};

#[test]
fn installed_absence_projection_refuses_before_retention_and_releases_snapshot() {
    if !isolated(
        "installed_absence_projection_refuses_before_retention_and_releases_snapshot",
        module_path!(),
    ) {
        return;
    }
    let _serial = SERIAL.lock().unwrap();
    let world = installed_authorization_world(true);
    let request_scope = live_scope();
    let actor = authenticated_principal(&world, &request_scope);
    let account = resolved_account(&world, "open", &request_scope);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &actor,
            &account,
            &operation,
            Default::default(),
            &request_scope,
        )
        .unwrap();
    let zero = authority().request_lease(request(0)).unwrap();
    let denial = match world.invariant.project_admitted_operation(
        &admission,
        |reader, _| {
            // Ignoring the local error must not seal a partial/empty projection.
            let _ = reader.decision_select_entities(
                AccountStatus::reference(),
                "unseen-retained-key".to_owned(),
                2,
            );
        },
        Policy::Execution(&zero),
    ) {
        Err(denial) => denial,
        Ok(_) => panic!("nonempty canonical key requires backing"),
    };
    let retained = denial.invariant_denial().unwrap();
    assert_eq!(
        retained.kind(),
        WorthQueryInvariantProjectionDenialKind::SourceRetentionDenied
    );
    assert_eq!(retained.projection_work().unwrap().equality_lookups(), 1);
    let allocation = retained.allocation_denial().unwrap();
    assert_eq!(
        allocation.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::ResourceExhausted)
    );
    assert!(allocation.requested_payload_bytes().unwrap() > 0);
    drop(zero);
    const PAYLOAD: u64 = 4 * 1024 * 1024;
    let parent = authority().request_lease(request(PAYLOAD)).unwrap();
    let child = parent.child(request(PAYLOAD)).unwrap();
    let (_, snapshot, work) = world
        .invariant
        .project_admitted_operation(
            &admission,
            |reader, _| {
                assert!(reader
                    .decision_select_entities(
                        AccountStatus::reference(),
                        "unseen-retained-key".to_owned(),
                        2
                    )
                    .unwrap()
                    .is_empty());
            },
            Policy::Execution(&child),
        )
        .unwrap()
        .into_parts();
    assert_eq!(work.equality_lookups(), 1);
    drop(child);
    assert!(
        matches!(
            parent.reserve_memory(PAYLOAD),
            Err(LeaseDenial::ResourceExhausted)
        ),
        "sealed snapshot retains its actual inline/key backings"
    );
    drop(snapshot);
    assert_eq!(
        parent.reserve_memory(PAYLOAD).unwrap().charged_bytes(),
        PAYLOAD
    );
}
