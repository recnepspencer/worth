use worth_query::facade::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
use worth_query::facade::runtime::{
    WorthQueryAspectTouch, WorthQueryAuthoredAspectValue, WorthQueryWorkspace,
};

use crate::{
    worth_ui_domain_package, worth_ui_native_aspect_contracts, WorthUiInstalledQueryDomain,
    WorthUiInstalledQueryView, WorthUiInstalledSnapshotQueryView, WorthUiQueryAllocationDetail,
    WorthUiQueryBindingPlan, WorthUiQueryConsumerRequirements, WorthUiQueryDenialPresentation,
    WorthUiQueryInspectionRelevance, WorthUiQueryViewShape, WorthUiQueryWorkspaceExt,
    WorthUiSettledSnapshotProjection,
};

pub fn worth_ui_installed_test_domain(label: &str) -> WorthUiInstalledQueryDomain {
    let schema = WorthQueryTestBackendSchema::single_collection("WorthUiMeasurement")
        .aspect_contracts(worth_ui_native_aspect_contracts())
        .expect("Worth UI aspect contracts")
        .aspect("identity.id", "identity.id")
        .expect("identity aspect")
        .aspect("measurement.value", "measurement.value")
        .expect("measurement aspect");
    crate::certification::request_custody::with_bootstrap_request(|execution| {
        crate::install_worth_ui_test_operation_executors(
            in_memory_test_runtime()
                .with_schema(schema)
                .domain_package(worth_ui_domain_package()),
        )
        .workspace(execution, label)
        .expect("installed Worth UI Query workspace")
        .worth_ui()
        .expect("Worth UI domain installed")
    })
}

/// Hostile integration fixture owned by the only crate allowed to translate
/// Query execution into Worth UI binding artifacts.
pub struct WorthUiInstalledQueryTestFixture {
    pub(super) workspace: WorthQueryWorkspace,
    view: WorthUiInstalledSnapshotQueryView,
    plan: WorthUiQueryBindingPlan,
    reference: crate::WorthUiInstalledQueryBindingReference,
    admitted_binding_reference: Option<crate::WorthUiAdmittedQueryBindingReference>,
}

impl WorthUiInstalledQueryTestFixture {
    pub fn new(label: &str) -> Self {
        let schema = WorthQueryTestBackendSchema::single_collection("WorthUiMeasurement")
            .aspect_contracts(worth_ui_native_aspect_contracts())
            .expect("Worth UI aspect contracts")
            .aspect("identity.id", "identity.id")
            .expect("identity aspect")
            .aspect("measurement.value", "measurement.value")
            .expect("measurement aspect");
        let mut workspace =
            crate::certification::request_custody::with_bootstrap_request(|execution| {
                crate::install_worth_ui_test_operation_executors(
                    in_memory_test_runtime()
                        .with_schema(schema)
                        .domain_package(worth_ui_domain_package()),
                )
                .workspace(execution, label)
                .expect("installed Worth UI Query workspace")
            });
        workspace
            .with_execution_request_mut(|execution, workspace| {
                workspace
                    .insert(execution, "WorthUiMeasurement", |measurement| {
                        measurement
                            .set_aspect(
                                WorthQueryAspectTouch::from_authoring_ingress_text("identity.id")
                                    .expect("identity touch"),
                                WorthQueryAuthoredAspectValue::string("measurement"),
                            )
                            .set_aspect(
                                WorthQueryAspectTouch::from_authoring_ingress_text(
                                    "measurement.value",
                                )
                                .expect("measurement touch"),
                                WorthQueryAuthoredAspectValue::native(
                                    worth_foundational::AspectValue::Float32(
                                        worth_foundational::CanonicalF32::from_f32(240.0),
                                    ),
                                ),
                            )
                    })
                    .expect("measurement insertion")
            })
            .expect("workspace host call opens");
        let installed = workspace.worth_ui().expect("Worth UI domain installed");
        let view = installed
            .measurement_view("inspector.measurements")
            .expect("measurement view");
        let plan = WorthUiQueryBindingPlan::default()
            .register_view(view.clone())
            .expect("installed view registration");
        let reference = plan
            .resolve_definition(
                view.definition().identity(),
                WorthUiQueryViewShape::Collection,
            )
            .expect("fixture resolves its canonical installed reference");
        let mut fixture = Self {
            workspace,
            view,
            plan,
            reference,
            admitted_binding_reference: None,
        };
        let admitted_binding_reference =
            fixture.settle_snapshot().fact().binding_reference().clone();
        fixture.admitted_binding_reference = Some(admitted_binding_reference);
        fixture
    }

    pub fn binding_plan(&self) -> WorthUiQueryBindingPlan {
        self.plan.clone()
    }

    /// Clone the installed view as input to a production registration facade.
    /// The fixture retains its workspace so later projections carry the same
    /// installed Query authority as the registered view.
    pub fn installed_view(&self) -> WorthUiInstalledQueryView {
        self.view.clone().into()
    }

    pub fn binding_reference(&self) -> &crate::WorthUiAdmittedQueryBindingReference {
        self.admitted_binding_reference
            .as_ref()
            .expect("fixture initialization admits one real settled binding")
    }

    pub fn settle_snapshot(&mut self) -> WorthUiSettledSnapshotProjection {
        self.reference
            .clone()
            .enter_snapshot_attempt(&self.workspace)
            .expect("fixture enters the exact Query operating world")
            .prepare_snapshot_consumer(WorthUiQueryConsumerRequirements::new(
                worth_query::facade::domain::WorthQueryConsumerBoundaryRequirements {
                    presentation: worth_query::facade::domain::WorthQueryConsumerPresentationPosture::Interactive,
                    allocation: worth_query::facade::domain::WorthQueryConsumerAllocationPosture::Borrowed,
                },
                WorthUiQueryAllocationDetail::BorrowedFactSlice,
                WorthUiQueryViewShape::Collection,
                WorthUiQueryDenialPresentation::StructuredStatus,
                WorthUiQueryInspectionRelevance::Relevant,
            ))
            .expect("Query mints one consumer contract")
            .execute(&mut self.workspace)
            .unwrap()
            .publish()
            .unwrap()
            .consume()
            .unwrap()
            .settle()
            .unwrap()
    }

    /// Builds an owned value only for isolated runtime unit fixtures whose
    /// subject begins after query-binding retention. Production scenarios
    /// should use the retained projection or its shared fact reference.
    pub fn clone_retained_fact_for_isolated_test(&mut self) -> crate::WorthUiSettledSnapshotFact {
        let plan = self.binding_plan();
        let mut downstream = plan.prepare_downstream_state();
        let retained = downstream
            .admit_settled_snapshot(self.settle_snapshot())
            .expect("fixture settlement belongs to its installed binding plan");
        retained.as_ref().clone()
    }

    pub fn close_retirement(
        &mut self,
        retirement: crate::WorthUiOperationLiveRetirement,
    ) -> crate::WorthUiOperationLiveRetirementCloseOutcome {
        retirement.close(&mut self.workspace)
    }
}
