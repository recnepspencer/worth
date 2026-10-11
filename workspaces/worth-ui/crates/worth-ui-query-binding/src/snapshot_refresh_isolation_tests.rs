use worth_query::facade::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
use worth_query::facade::domain;

use crate::{
    worth_ui_domain_package, WorthUiQueryAllocationDetail, WorthUiQueryBindingPlan,
    WorthUiQueryConsumerRequirements, WorthUiQueryDenialPresentation,
    WorthUiQueryInspectionRelevance, WorthUiQueryViewShape, WorthUiQueryWorkspaceExt,
};

#[test]
fn refreshing_one_real_binding_preserves_the_unrelated_settlement_slot() {
    let mut fixture = refresh_isolation_fixture();
    let first_before = fixture
        .runtime
        .admit_settled_snapshot(settle(&fixture.first_reference, &mut fixture.workspace))
        .unwrap();
    let second_before = fixture
        .runtime
        .admit_settled_snapshot(settle(&fixture.second_reference, &mut fixture.workspace))
        .unwrap();
    let second_touch = fixture
        .runtime
        .settlement_touch_reference_for(&fixture.second_reference)
        .unwrap();

    let first_after = fixture
        .runtime
        .refresh_settled_snapshot(settle(&fixture.first_reference, &mut fixture.workspace))
        .unwrap();
    let second_after = fixture
        .runtime
        .settled_snapshot_fact_for(&fixture.second_reference)
        .unwrap();

    assert_eq!(
        first_after.binding_reference(),
        first_before.binding_reference()
    );
    assert_ne!(
        first_before.binding_reference(),
        second_before.binding_reference()
    );
    assert_ne!(
        first_after.settlement_reference(),
        first_before.settlement_reference()
    );
    assert_eq!(first_before.source_generation().unwrap().as_u64(), 1);
    assert_eq!(first_before.source_order().unwrap().as_u64(), 1);
    assert_eq!(first_after.source_generation().unwrap().as_u64(), 2);
    assert_eq!(first_after.source_order().unwrap().as_u64(), 3);
    assert_eq!(second_after, second_before.as_ref());
    assert_eq!(second_after.source_generation().unwrap().as_u64(), 1);
    assert_eq!(second_after.source_order().unwrap().as_u64(), 2);
    assert!(fixture
        .runtime
        .readmits_settlement_touch_reference(&fixture.second_reference, &second_touch)
        .unwrap());
}

struct RefreshIsolationFixture {
    workspace: worth_query::facade::runtime::WorthQueryWorkspace,
    runtime: crate::WorthUiRuntimeQueryBinding,
    first_reference: crate::WorthUiInstalledQueryBindingReference,
    second_reference: crate::WorthUiInstalledQueryBindingReference,
}

fn refresh_isolation_fixture() -> RefreshIsolationFixture {
    let workspace = installed_workspace();
    let first_view = workspace
        .worth_ui()
        .unwrap()
        .measurement_view("dashboard.primary")
        .unwrap();
    let second_view = workspace
        .worth_ui()
        .unwrap()
        .measurement_view("dashboard.secondary")
        .unwrap();
    let first_identity = first_view.definition().identity().clone();
    let second_identity = second_view.definition().identity().clone();
    let plan = WorthUiQueryBindingPlan::default()
        .register_view(first_view)
        .unwrap()
        .register_view(second_view)
        .unwrap();
    let first_reference = plan
        .resolve_definition(&first_identity, WorthUiQueryViewShape::Collection)
        .unwrap()
        .clone();
    let second_reference = plan
        .resolve_definition(&second_identity, WorthUiQueryViewShape::Collection)
        .unwrap()
        .clone();
    assert!(first_reference
        .installed_domain()
        .shares_authority_with(second_reference.installed_domain()));
    assert_eq!(
        first_reference.snapshot_operation(),
        second_reference.snapshot_operation()
    );
    assert_ne!(first_reference.definition(), second_reference.definition());
    RefreshIsolationFixture {
        workspace,
        runtime: plan.prepare_downstream_state(),
        first_reference,
        second_reference,
    }
}

pub(crate) fn settle(
    reference: &crate::WorthUiInstalledQueryBindingReference,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> crate::WorthUiSettledSnapshotProjection {
    reference
        .enter_snapshot_attempt(workspace)
        .unwrap()
        .prepare_snapshot_consumer(requirements())
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume()
        .unwrap()
        .settle()
        .unwrap()
}

fn requirements() -> WorthUiQueryConsumerRequirements {
    WorthUiQueryConsumerRequirements::new(
        domain::WorthQueryConsumerBoundaryRequirements {
            presentation: domain::WorthQueryConsumerPresentationPosture::Interactive,
            allocation: domain::WorthQueryConsumerAllocationPosture::Borrowed,
        },
        WorthUiQueryAllocationDetail::BorrowedFactSlice,
        WorthUiQueryViewShape::Collection,
        WorthUiQueryDenialPresentation::StructuredStatus,
        WorthUiQueryInspectionRelevance::Relevant,
    )
}

pub(crate) fn installed_workspace() -> worth_query::facade::runtime::WorthQueryWorkspace {
    let schema = WorthQueryTestBackendSchema::single_collection("WorthUiMeasurement")
        .aspect_contracts(crate::worth_ui_native_aspect_contracts())
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap()
        .aspect("measurement.value", "measurement.value")
        .unwrap();
    let mut workspace =
        crate::certification::request_custody::with_bootstrap_request(|execution| {
            crate::install_worth_ui_test_operation_executors(
                in_memory_test_runtime()
                    .with_schema(schema)
                    .domain_package(worth_ui_domain_package()),
            )
            .workspace(execution, "snapshot-refresh-isolation")
            .unwrap()
        });
    workspace.with_execution_request_mut(|execution, workspace| workspace
        .insert(execution, "WorthUiMeasurement", |measurement| {
            measurement
                .set_aspect(
                    worth_query::facade::runtime::WorthQueryAspectTouch::from_authoring_ingress_text(
                        "identity.id",
                    )
                    .unwrap(),
                    worth_query::facade::runtime::WorthQueryAuthoredAspectValue::string(
                        "measurement",
                    ),
                )
                .set_aspect(
                    worth_query::facade::runtime::WorthQueryAspectTouch::from_authoring_ingress_text(
                        "measurement.value",
                    )
                    .unwrap(),
                    worth_query::facade::runtime::WorthQueryAuthoredAspectValue::native(
                        worth_foundational::AspectValue::Float32(
                            worth_foundational::CanonicalF32::from_f32(240.0),
                        ),
                    ),
                )
        })
        .unwrap()).expect("workspace host call opens");
    workspace
}
