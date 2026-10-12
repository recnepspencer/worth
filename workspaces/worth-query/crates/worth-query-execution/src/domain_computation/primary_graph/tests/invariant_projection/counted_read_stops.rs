use super::*;
use crate::domain_computation::primary_graph::WorthQueryInvariantProjectionDenialKind as Kind;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
fn check_snapshot_door(door: u8) {
    let world = installed_authorization_world(true);
    let snapshot = world
        .application
        .with_host_advancement(|phase| world.invariant.snapshot(&phase))
        .unwrap()
        .unwrap();
    let host = crate::test_request_policy::serial_request();
    let request = worth_execution::ExecutionRequest::serial(&host);
    let account = snapshot
        .entities(&request, Account::reference())
        .unwrap()
        .remove(0);
    {
        crate::test_read_stops::every_consult(
            |execution| {
                match door {
                    0 => snapshot
                        .entities(execution, Account::reference())
                        .map(|rows| rows.len()),
                    1 => snapshot
                        .relations(execution, AccountOwner::reference())
                        .map(|rows| rows.len()),
                    _ => snapshot
                        .field(execution, &account, AccountStatus::reference())
                        .map(|value| usize::from(value.is_some())),
                }
                .map_err(|denial| denial.kind())
            },
            Kind::ExecutionRequest(Cause::Cancelled),
            Kind::ExecutionRequest(Cause::WorkExhausted),
        );
    }
}

#[test]
fn snapshot_entities_door_preserves_stops_at_every_consult() {
    check_snapshot_door(0);
}
#[test]
fn snapshot_relations_door_preserves_stops_at_every_consult() {
    check_snapshot_door(1);
}
#[test]
fn snapshot_field_door_preserves_stops_at_every_consult() {
    check_snapshot_door(2);
}
