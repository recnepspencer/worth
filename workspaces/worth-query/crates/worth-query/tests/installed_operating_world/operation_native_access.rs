use worth_foundational::facade::{FieldKey, InternedString};
use worth_query::facade::{domain, foundation, read};

use super::installed_operation_fixture::{
    workflow_workspace, workspace, GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex,
    WorkflowRead,
};

#[test]
fn installed_projection_key_borrows_the_exact_foundational_value_in_constant_work() {
    let mut workspace = workspace("installed-native-access", false).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let (request, key) = native_id_request(bound.consumer_projection_contract().unwrap());

    let settled = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(request)
        .unwrap()
        .settle()
        .unwrap();
    let access = settled.native_value(&key, 0).unwrap();

    assert_eq!(
        access.fact().as_interned_string(),
        Ok(&InternedString::Raw("synthetic-anchor".into()))
    );
    assert_eq!(access.counters().indexed_accesses, 1);
    assert_eq!(access.counters().refinement_checks, 1);
    assert_eq!(access.counters().fact_scans, 0);
    assert_eq!(access.counters().row_scans, 0);
    assert_eq!(access.counters().path_parses, 0);
}

#[test]
fn key_from_an_equivalent_distinct_capability_is_denied_before_indexing() {
    let mut workspace = workspace("installed-native-affinity", false).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let owner = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let foreign = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    assert_eq!(owner.binding_identity(), foreign.binding_identity());
    let (owner_request, owner_key) =
        native_id_request(owner.consumer_projection_contract().unwrap());
    let (_foreign_request, foreign_key) =
        native_id_request(foreign.consumer_projection_contract().unwrap());

    let settled = owner
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(owner_request)
        .unwrap()
        .settle()
        .unwrap();
    assert!(settled.native_value(&owner_key, 0).is_ok());
    let denial = settled.native_value(&foreign_key, 0).unwrap_err();

    assert_eq!(
        denial.kind(),
        domain::WorthQueryNativeAccessDenialKind::CapabilityMismatch
    );
    assert_eq!(denial.counters().indexed_accesses, 0);
    assert_eq!(denial.counters().refinement_checks, 0);
    assert_eq!(denial.counters().fact_scans, 0);
    assert_eq!(denial.counters().path_parses, 0);
}

#[test]
fn out_of_bounds_row_is_distinct_and_does_no_fact_access() {
    let mut workspace = workspace("installed-native-row-bound", false).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let (request, key) = native_id_request(bound.consumer_projection_contract().unwrap());
    let settled = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(request)
        .unwrap()
        .settle()
        .unwrap();

    let denial = settled.native_value(&key, 1).unwrap_err();
    assert_eq!(
        denial.kind(),
        domain::WorthQueryNativeAccessDenialKind::RowOutOfBounds
    );
    assert_eq!(denial.counters().indexed_accesses, 0);
}

#[test]
fn access_key_requires_the_bound_native_layout() {
    let mut workspace = workspace("installed-native-layout-required", false).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let owner = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let foreign = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let (_foreign_request, key) =
        native_id_request(foreign.consumer_projection_contract().unwrap());
    let owner_consumer = owner.consumer_projection_contract().unwrap();
    let declaration = read::project_facts().entity_identities();
    let settled = owner
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(owner_consumer, declaration)
        .unwrap()
        .settle()
        .unwrap();
    let denial = settled.native_value(&key, 0).unwrap_err();
    assert_eq!(
        denial.kind(),
        domain::WorthQueryNativeAccessDenialKind::LayoutMismatch
    );
    assert_eq!(denial.counters().indexed_accesses, 0);
    assert_eq!(denial.counters().refinement_checks, 0);
}

#[test]
fn workflow_publication_uses_the_same_bound_native_access_contract() {
    let mut workspace = workflow_workspace("installed-workflow-native-access").unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, WorkflowRead)
        .unwrap();
    let (request, key) = native_id_request(bound.consumer_projection_contract().unwrap());
    let trace = bound
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            &mut workspace,
        )
        .unwrap()
        .advance(
            "right",
            domain::WorthQueryWorkflowValue::Text("start".into()),
            &mut workspace,
        )
        .unwrap()
        .advance(
            "left",
            domain::WorthQueryWorkflowValue::Text("start".into()),
            &mut workspace,
        )
        .unwrap()
        .advance(
            "publish",
            domain::WorthQueryWorkflowValue::Text("join".into()),
            &mut workspace,
        )
        .unwrap()
        .complete()
        .unwrap();
    let settled = trace
        .publish()
        .unwrap()
        .consume_bound(request)
        .unwrap()
        .settle()
        .unwrap();

    let access = settled.native_value(&key, 0).unwrap();
    assert_eq!(
        access.fact().as_interned_string(),
        Ok(&InternedString::Raw("synthetic-anchor".into()))
    );
    assert_eq!(access.counters().indexed_accesses, 1);
    assert_eq!(access.counters().fact_scans, 0);
}

fn native_id_request<D, O, F, L: foundation::BasisOperationLane>(
    consumer: domain::WorthQueryConsumerProjectionContract<D, O, F, L>,
) -> (
    domain::WorthQueryBoundProjectionRequest<D, O, F, L>,
    domain::WorthQueryNativeAccessKey,
) {
    let mut builder = consumer.projection_request();
    let selection = builder
        .select_display_native_field(FieldKey::new("id").unwrap())
        .unwrap();
    let request = builder.build().unwrap();
    let key = request.resolve_native_key(&selection).unwrap().into_key();
    (request, key)
}
