pub use worth_foundational::facade::{FieldKey, InternedString};
pub use worth_query::facade::{domain, foundation, read};

pub use super::installed_operation_fixture::{
    configured_runtime, GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex,
};

pub type SettledProjection = domain::WorthQuerySettledDomainProjection<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;

pub fn settle(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> SettledProjection {
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();
    let consumer = bound.consumer_projection_contract().unwrap();
    bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(consumer, read::project_facts().entity_identities())
        .unwrap()
        .settle()
        .unwrap()
}

pub fn settle_native(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> (SettledProjection, domain::WorthQueryNativeAccessKey) {
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();
    let mut builder = bound
        .consumer_projection_contract()
        .unwrap()
        .projection_request();
    let selection = builder
        .select_display_native_field(FieldKey::new("id").unwrap())
        .unwrap();
    let request = builder.build().unwrap();
    let key = request.resolve_native_key(&selection).unwrap().into_key();
    let settled = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(request)
        .unwrap()
        .settle()
        .unwrap();
    (settled, key)
}

pub fn assert_no_attempt_or_planning(counters: domain::WorthQueryProjectionPromotionCounters) {
    assert_eq!(counters.lifecycle_attempts, 0);
    assert_eq!(counters.fresh_conditional_decisions, 0);
    assert_eq!(counters.planning_attempts, 0);
    assert_eq!(counters.lower_runtime_contacts, 0);
    assert_eq!(counters.managed_resource_registrations, 0);
}
