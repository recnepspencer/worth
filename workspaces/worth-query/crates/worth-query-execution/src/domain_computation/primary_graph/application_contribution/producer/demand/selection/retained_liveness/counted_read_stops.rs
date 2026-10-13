use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, Account, AccountStatus,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
#[test]
fn retained_output_liveness_door_preserves_stops_at_every_consult() {
    let world = installed_authorization_world(true);
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.borrowed_request_for_test();
            let snapshot = world.invariant.snapshot(&phase).unwrap();
            let account = snapshot
                .entities(&execution, Account::reference())
                .unwrap()
                .remove(0);
            let entity = account.entity_id();
            assert!(snapshot
                .field(&execution, &account, AccountStatus::reference())
                .unwrap()
                .is_some());
            let selected = world.selected_product(&phase.test_host_request());
            world
                .application
                .runtime
                .primary_graph()
                .unwrap()
                .integration_handle()
                .with_runtime(&execution, |runtime| {
                    let view = runtime
                        .read_truth()
                        .project_snapshot(selected.application_basis().snapshot_handle())
                        .unwrap();
                    crate::test_read_stops::every_consult_under(
                        &execution,
                        |request| read(request, &view, entity).map_err(|denied| denied.kind()),
                        denial(Cause::Cancelled).kind(),
                        denial(Cause::WorkExhausted).kind(),
                    );
                });
        })
        .unwrap();
}
