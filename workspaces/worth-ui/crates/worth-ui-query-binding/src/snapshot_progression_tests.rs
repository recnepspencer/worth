use worth_query::facade::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
use worth_query::facade::domain;

use crate::{
    worth_ui_domain_package, WorthUiQueryAllocationDetail, WorthUiQueryBindingPlan,
    WorthUiQueryConsumerRequirements, WorthUiQueryDenialPresentation, WorthUiQueryInspection,
    WorthUiQueryInspectionRelevance, WorthUiQueryViewIdentity, WorthUiQueryViewShape,
    WorthUiQueryWorkspaceExt, WorthUiSettledSnapshotAdmissionDenial,
    WorthUiSnapshotConsumerPreparationDenial,
};

mod cost;

#[test]
fn query_mints_support_while_ui_presentation_remains_adjacent() {
    let workspace = installed_workspace("consumer-boundary-adjacency");
    let reference = installed_reference(&workspace);
    let hidden = reference
        .enter_snapshot_attempt(&workspace)
        .unwrap()
        .prepare_snapshot_consumer(requirements(WorthUiQueryDenialPresentation::Hidden))
        .unwrap();
    let structured = reference
        .enter_snapshot_attempt(&workspace)
        .unwrap()
        .prepare_snapshot_consumer(requirements(
            WorthUiQueryDenialPresentation::StructuredStatus,
        ))
        .unwrap();

    assert_eq!(
        hidden.installed_reference(),
        structured.installed_reference()
    );
    assert_eq!(
        hidden.query_boundary_requirements(),
        structured.query_boundary_requirements()
    );
    assert_ne!(hidden.ui_requirements(), structured.ui_requirements());
}

#[test]
fn one_bound_operation_cannot_mint_a_second_consumer_contract() {
    let workspace = installed_workspace("single-consumer-contract");
    let reference = installed_reference(&workspace);
    let bound = reference
        .enter_snapshot_attempt(&workspace)
        .unwrap()
        .bind_snapshot()
        .unwrap();

    let _first = bound.consumer_projection_contract().unwrap();
    let second = match bound.consumer_projection_contract() {
        Ok(_) => panic!("Query must deny a second consumer contract"),
        Err(denial) => denial,
    };

    assert!(matches!(
        second,
        domain::WorthQueryConsumerProjectionContractDenial::AlreadyMinted { .. }
    ));
}

#[test]
fn unsupported_projection_consumption_is_a_query_denial() {
    let workspace = crate::certification::request_custody::with_bootstrap_request(|execution| {
        installed_builder()
            .consumer_support_posture(
                domain::WorthQueryConsumerSupportDimension::ProjectionConsumption,
                domain::WorthQueryConsumerSupportPosture::Unsupported,
            )
            .workspace(execution, "query-owned-support-denial")
            .unwrap()
    });
    let reference = installed_reference(&workspace);
    let denial = reference
        .enter_snapshot_attempt(&workspace)
        .unwrap()
        .prepare_snapshot_consumer(requirements(
            WorthUiQueryDenialPresentation::StructuredStatus,
        ));

    let denial = match denial {
        Err(denial) => denial,
        Ok(_) => panic!("unsupported projection must keep its exact Query denial"),
    };
    assert!(matches!(
        &denial,
        WorthUiSnapshotConsumerPreparationDenial::ConsumerContract(
            domain::WorthQueryConsumerProjectionContractDenial::Compatibility(_)
        )
    ));
    let inspection =
        WorthUiQueryInspection::exact_artifact(&denial, WorthUiQueryInspectionRelevance::Relevant);
    assert!(std::ptr::eq(inspection.exact_artifact(), &denial));
    assert_eq!(inspection.counters().rich_evidence_section_count(), 0);
}

#[test]
fn exact_settled_projection_is_retained_once_and_derives_ui_fact() {
    let mut workspace = installed_workspace("exact-settlement-retention");
    let (plan, view_identity) = binding_plan(&workspace);
    let reference = plan
        .resolve_definition(&view_identity, WorthUiQueryViewShape::Collection)
        .unwrap();
    let settled = settle(&reference, &mut workspace);
    let derived_fact_address = settled.fact() as *const crate::WorthUiSettledSnapshotFact;
    let batch = settled.fact().measurement_facts();
    let resolution = batch.key_resolution_counters();
    assert_eq!(resolution.declaration_checks(), 1);
    assert_eq!(resolution.indexed_slot_lookups(), 2);
    assert_eq!(resolution.key_scans(), 0);
    assert_eq!(resolution.path_parses(), 0);
    let access = batch.native_access_counters();
    assert_eq!(access.indexed_accesses, 1);
    assert_eq!(access.refinement_checks, 1);
    assert_eq!(access.fact_scans, 0);
    assert_eq!(access.row_scans, 0);
    assert_eq!(access.path_parses, 0);
    let binding = batch.native_access_binding_counters().unwrap();
    assert_eq!(binding.declared_key_routes(), 1);
    assert_eq!(binding.declared_key_layout_checks(), 1);
    assert_eq!(binding.lane_shape_checks(), 2);
    assert_eq!(binding.fact_scans(), 0);
    assert_eq!(binding.row_scans(), 0);
    assert_eq!(binding.path_parses(), 0);
    let expected_binding = settled.fact().binding_reference().clone();
    let expected_settlement = settled.fact().settlement_reference().clone();
    let mut runtime = plan.prepare_downstream_state();
    let fact = runtime
        .admit_settled_snapshot(settled)
        .expect("exact settled projection should admit once");

    assert_eq!(
        std::sync::Arc::as_ptr(&fact),
        derived_fact_address,
        "admission must share the projection's one derived UI fact"
    );
    assert_eq!(fact.binding_reference(), &expected_binding);
    assert_eq!(fact.settlement_reference(), &expected_settlement);
    assert_eq!(fact.source_generation().unwrap().as_u64(), 1);
    assert_eq!(fact.source_order().unwrap().as_u64(), 1);
    assert_eq!(
        runtime.settled_snapshot_fact_for(&reference).unwrap(),
        fact.as_ref()
    );
    let duplicate = settle(&reference, &mut workspace);
    let stop = runtime
        .admit_settled_snapshot(duplicate)
        .expect_err("one runtime slot retains one exact settled projection");
    assert_eq!(
        stop.denial(),
        WorthUiSettledSnapshotAdmissionDenial::DuplicateSettlement
    );
    let returned = stop.into_projection();
    assert_eq!(returned.installed_reference(), &reference);
}

#[test]
fn foreign_equal_installation_cannot_admit_exact_settlement() {
    let mut owner = installed_workspace("exact-settlement-equal-label");
    let mut foreign = installed_workspace("exact-settlement-equal-label");
    let (owner_plan, owner_identity) = binding_plan(&owner);
    let owner_reference = owner_plan
        .resolve_definition(&owner_identity, WorthUiQueryViewShape::Collection)
        .unwrap();
    let (foreign_plan, foreign_identity) = binding_plan(&foreign);
    let foreign_reference = foreign_plan
        .resolve_definition(&foreign_identity, WorthUiQueryViewShape::Collection)
        .unwrap();
    let owner_projection = settle(&owner_reference, &mut owner);
    let foreign_projection = settle(&foreign_reference, &mut foreign);
    assert_eq!(owner_reference.definition(), foreign_reference.definition());
    assert_eq!(
        owner_projection.fact().measurement_facts().observations(),
        foreign_projection.fact().measurement_facts().observations()
    );
    assert_ne!(
        owner_projection.fact().binding_reference(),
        foreign_projection.fact().binding_reference()
    );
    assert_ne!(
        owner_projection.fact().settlement_reference(),
        foreign_projection.fact().settlement_reference()
    );

    let mut owner_runtime = owner_plan.prepare_downstream_state();
    owner_runtime
        .admit_settled_snapshot(owner_projection)
        .unwrap();
    let mut foreign_runtime = foreign_plan.prepare_downstream_state();
    foreign_runtime
        .admit_settled_snapshot(foreign_projection)
        .unwrap();
    let foreign_fact = foreign_runtime
        .settled_snapshot_fact_for(&foreign_reference)
        .unwrap()
        .clone();
    assert_eq!(
        owner_runtime
            .readmit_settled_snapshot_fact(&foreign_fact)
            .err(),
        Some(crate::WorthUiSettledSnapshotReadmissionDenial::ForeignInstalledReference)
    );
    let foreign_touch = foreign_runtime
        .settlement_touch_reference_for(&foreign_reference)
        .unwrap();
    assert!(!owner_runtime
        .readmits_settlement_touch_reference(&owner_reference, &foreign_touch)
        .unwrap());

    let stop = owner_runtime
        .refresh_settled_snapshot(settle(&foreign_reference, &mut foreign))
        .expect_err("equal labels cannot replace exact Query authority");

    assert_eq!(
        stop.denial(),
        WorthUiSettledSnapshotAdmissionDenial::ForeignInstalledReference
    );
}

#[test]
fn in_generation_refresh_atomically_advances_source_coordinates() {
    let mut workspace = installed_workspace("exact-settlement-refresh");
    let (plan, view_identity) = binding_plan(&workspace);
    let reference = plan
        .resolve_definition(&view_identity, WorthUiQueryViewShape::Collection)
        .unwrap();
    let mut runtime = plan.prepare_downstream_state();
    let first = runtime
        .admit_settled_snapshot(settle(&reference, &mut workspace))
        .unwrap();
    let predecessor_touch = runtime.settlement_touch_reference_for(&reference).unwrap();
    let second = runtime
        .refresh_settled_snapshot(settle(&reference, &mut workspace))
        .unwrap();
    let current_touch = runtime.settlement_touch_reference_for(&reference).unwrap();

    assert_eq!(first.source_generation().unwrap().as_u64(), 1);
    assert_eq!(first.source_order().unwrap().as_u64(), 1);
    assert_eq!(second.source_generation().unwrap().as_u64(), 2);
    assert_eq!(second.source_order().unwrap().as_u64(), 2);
    assert_eq!(first.binding_reference(), second.binding_reference());
    assert_ne!(first.settlement_reference(), second.settlement_reference());
    assert_eq!(
        runtime.readmit_settled_snapshot_fact(&first).err(),
        Some(crate::WorthUiSettledSnapshotReadmissionDenial::StaleSettlementReference)
    );
    let readmitted = runtime
        .readmit_settled_snapshot_fact(&second)
        .expect("the current settlement fact must readmit");
    assert_eq!(readmitted.fact(), second.as_ref());
    assert!(!runtime
        .readmits_settlement_touch_reference(&reference, &predecessor_touch)
        .unwrap());
    assert!(runtime
        .readmits_settlement_touch_reference(&reference, &current_touch)
        .unwrap());
    assert_eq!(
        runtime.settled_snapshot_fact_for(&reference).unwrap(),
        second.as_ref()
    );

    let mut foreign_workspace = installed_workspace("exact-settlement-refresh-foreign");
    let foreign_reference = installed_reference(&foreign_workspace);
    let stop = runtime
        .refresh_settled_snapshot(settle(&foreign_reference, &mut foreign_workspace))
        .expect_err("foreign refresh must return its projection");
    assert_eq!(
        stop.denial(),
        WorthUiSettledSnapshotAdmissionDenial::ForeignInstalledReference
    );
    assert_eq!(
        runtime.settled_snapshot_fact_for(&reference).unwrap(),
        second.as_ref()
    );
}

fn settle(
    reference: &crate::WorthUiInstalledQueryBindingReference,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> crate::WorthUiSettledSnapshotProjection {
    reference
        .enter_snapshot_attempt(workspace)
        .unwrap()
        .prepare_snapshot_consumer(requirements(
            WorthUiQueryDenialPresentation::StructuredStatus,
        ))
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

fn requirements(denial: WorthUiQueryDenialPresentation) -> WorthUiQueryConsumerRequirements {
    WorthUiQueryConsumerRequirements::new(
        domain::WorthQueryConsumerBoundaryRequirements {
            presentation: domain::WorthQueryConsumerPresentationPosture::Interactive,
            allocation: domain::WorthQueryConsumerAllocationPosture::Borrowed,
        },
        WorthUiQueryAllocationDetail::BorrowedFactSlice,
        WorthUiQueryViewShape::Collection,
        denial,
        WorthUiQueryInspectionRelevance::Relevant,
    )
}

fn installed_reference(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
) -> crate::WorthUiInstalledQueryBindingReference {
    let (plan, view_identity) = binding_plan(workspace);
    plan.resolve_definition(&view_identity, WorthUiQueryViewShape::Collection)
        .unwrap()
}

fn binding_plan(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
) -> (WorthUiQueryBindingPlan, WorthUiQueryViewIdentity) {
    let view = workspace
        .worth_ui()
        .unwrap()
        .measurement_view("dashboard.measurements")
        .unwrap();
    let view_identity = view.definition().identity().clone();
    (
        WorthUiQueryBindingPlan::default()
            .register_view(view)
            .unwrap(),
        view_identity,
    )
}

fn installed_workspace(label: &str) -> worth_query::facade::runtime::WorthQueryWorkspace {
    let mut workspace =
        crate::certification::request_custody::with_bootstrap_request(|execution| {
            installed_builder().workspace(execution, label).unwrap()
        });
    workspace.with_execution_request_mut(|execution, mut workspace| workspace
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

fn installed_builder() -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    let schema = WorthQueryTestBackendSchema::single_collection("WorthUiMeasurement")
        .aspect_contracts(crate::worth_ui_native_aspect_contracts())
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap()
        .aspect("measurement.value", "measurement.value")
        .unwrap();
    crate::install_worth_ui_test_operation_executors(
        in_memory_test_runtime()
            .with_schema(schema)
            .domain_package(worth_ui_domain_package()),
    )
}
