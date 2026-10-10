use worth_foundational::facade::{AspectValue, CanonicalF32};
use worth_query::facade::{foundation::ObservationLaneWitness, installed, runtime};

use super::{aspect_touch, bound_snapshot, installed_builder, measurement_value_path};
use crate::{
    WorthUiQueryAllocationDetail, WorthUiQueryBindingPlan, WorthUiQueryConsumerRequirements,
    WorthUiQueryDenialPresentation, WorthUiQueryInspectionRelevance, WorthUiQueryWorkspaceExt,
    WorthUiSettledSnapshotProjection, WorthUiSnapshotConsumerPreparationDenial,
};

type QuerySettledSnapshot = installed::operation::WorthQuerySettledDomainProjection<
    crate::WorthUiDomainEntry,
    crate::WorthUiSnapshotMeasurement,
    crate::WorthUiSnapshotMeasurementFamily,
    ObservationLaneWitness,
>;

#[test]
fn ui_and_direct_consumers_converge_on_query_results_authority_and_counters() {
    let mut workspace = convergence_workspace();
    let reference = installed_reference(&workspace, "reference.convergence");
    let (direct, _) = settle_headless(&mut workspace);
    let ui = settle_ui(reference, &mut workspace);
    assert_exact_convergence(&ui, &direct);
}

fn convergence_workspace() -> runtime::WorthQueryWorkspace {
    let mut workspace =
        crate::certification::request_custody::with_bootstrap_request(|execution| {
            installed_builder()
                .workspace(execution, "worth-ui-reference-convergence")
                .expect("canonical Query world installs")
        });
    workspace
        .with_execution_request_mut(|execution, mut workspace| {
            workspace
                .insert(execution, "WorthUiMeasurement", |measurement| {
                    measurement
                        .set_aspect(
                            aspect_touch("identity.id"),
                            runtime::WorthQueryAuthoredAspectValue::string("measurement"),
                        )
                        .set_aspect(
                            aspect_touch("measurement.value"),
                            runtime::WorthQueryAuthoredAspectValue::native(AspectValue::Float32(
                                CanonicalF32::from_f32(240.0),
                            )),
                        )
                })
                .expect("canonical measurement delta applies")
        })
        .expect("workspace host call opens");
    workspace
}

fn settle_headless(workspace: &mut runtime::WorthQueryWorkspace) -> (QuerySettledSnapshot, String) {
    let direct = bound_snapshot(workspace);
    let installed_operation_identity = direct.definition().canonical_identity().to_owned();
    let direct_consumer = direct
        .consumer_projection_contract()
        .expect("direct consumer contract")
        .with_downstream_requirements(requirements().query_boundary());
    let support_counters = direct_consumer.query_contract().counters();
    assert_eq!(support_counters.installation_generation_checks, 1);
    assert_eq!(support_counters.mint_guard_checks, 1);
    assert_eq!(support_counters.dimensions_evaluated, 15);
    assert_eq!(support_counters.reporting_digest_comparisons, 0);
    assert_eq!(support_counters.downstream_hook_inspections, 0);
    let direct_operation_identity = direct_consumer
        .query_contract()
        .canonical_operation_identity()
        .to_owned();
    assert_eq!(direct_operation_identity, installed_operation_identity);
    let direct = direct
        .admit_execution_resources(
            (),
            crate::installed_domain::execution_resources::operation_execution_resource_request(),
            workspace,
        )
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(
            direct_consumer.into_query_contract(),
            worth_query::facade::read::project_facts().display_field(measurement_value_path()),
        )
        .unwrap()
        .settle()
        .unwrap();
    (direct, direct_operation_identity)
}

fn settle_ui(
    reference: crate::WorthUiInstalledQueryBindingReference,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> WorthUiSettledSnapshotProjection {
    let ui_prepared = reference
        .enter_snapshot_attempt(workspace)
        .expect("UI enters the owner-issued Query world")
        .prepare_snapshot_consumer(requirements())
        .expect("UI consumer requirements admit");
    assert_eq!(ui_prepared.installed_reference(), &reference);
    ui_prepared
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume()
        .unwrap()
        .settle()
        .unwrap()
}

fn assert_exact_convergence(ui: &WorthUiSettledSnapshotProjection, direct: &QuerySettledSnapshot) {
    assert_eq!(ui.result_state(), direct.result_state());
    assert_eq!(
        ui.result_state(),
        installed::operation::WorthQueryOperationResultState::Ready
    );
    assert_eq!(ui.execution_warnings(), direct.warnings());
    assert!(ui.execution_warnings().is_empty());
    assert_eq!(ui.counters(), direct.counters());
    assert_eq!(direct.counters().runtime_authority_checks, 1);
    assert_eq!(direct.resources().counters().input_contract_checks, 1);
    assert_eq!(direct.counters().primary_read_contacts, 1);
    assert_eq!(direct.counters().executor_contacts, 1);
    assert_eq!(direct.counters().terminal_posture_checks, 1);
    assert_eq!(direct.counters().publication_checks, 1);
    assert_eq!(direct.counters().consumption_contacts, 1);
    assert_eq!(
        ui.fact().measurement_facts().observations()[0].extent(),
        CanonicalF32::from_f32(240.0)
    );
}

#[test]
fn ui_and_direct_consumers_preserve_the_same_query_denial_and_cost() {
    let workspace = crate::certification::request_custody::with_bootstrap_request(|execution| {
        installed_builder()
            .consumer_support_posture(
                installed::support::WorthQueryConsumerSupportDimension::ProjectionConsumption,
                installed::support::WorthQueryConsumerSupportPosture::Unsupported,
            )
            .workspace(execution, "worth-ui-reference-denial-convergence")
            .expect("unsupported canonical Query world installs")
    });
    let reference = installed_reference(&workspace, "reference.denial.convergence");

    let direct_denial = match bound_snapshot(&workspace).consumer_projection_contract() {
        Ok(_) => panic!("unsupported direct consumer must deny"),
        Err(denial) => denial,
    };
    let ui_denial = match reference
        .enter_snapshot_attempt(&workspace)
        .expect("UI enters the owner-issued Query world")
        .prepare_snapshot_consumer(requirements())
    {
        Ok(_) => panic!("unsupported UI consumer must retain Query's denial"),
        Err(denial) => denial,
    };
    let WorthUiSnapshotConsumerPreparationDenial::ConsumerContract(ui_denial) = ui_denial else {
        panic!("UI denied before reaching the Query consumer boundary")
    };

    assert_eq!(ui_denial, direct_denial);
    assert_eq!(ui_denial.counters(), direct_denial.counters());
    let installed::operation::WorthQueryConsumerProjectionContractDenial::Compatibility(
        exact_denial,
    ) = direct_denial
    else {
        panic!("unsupported projection consumption must retain its exact compatibility denial")
    };
    assert_eq!(
        exact_denial.dimension(),
        installed::support::WorthQueryConsumerSupportDimension::ProjectionConsumption
    );
    assert_eq!(
        exact_denial.runtime_posture(),
        installed::support::WorthQueryConsumerSupportPosture::Unsupported
    );
    assert_eq!(exact_denial.counters().installation_generation_checks, 1);
    assert_eq!(exact_denial.counters().mint_guard_checks, 1);
    assert!(exact_denial.counters().dimensions_evaluated > 0);
    assert_eq!(exact_denial.counters().reporting_digest_comparisons, 0);
    assert_eq!(exact_denial.counters().downstream_hook_inspections, 0);
}

fn installed_reference(
    workspace: &runtime::WorthQueryWorkspace,
    local_name: &str,
) -> crate::WorthUiInstalledQueryBindingReference {
    let installed = workspace.worth_ui().expect("Worth UI domain is installed");
    let view = installed
        .measurement_view(local_name)
        .expect("installed domain issues the view identity");
    let view_identity = view.definition().identity().clone();
    WorthUiQueryBindingPlan::default()
        .register_view(view)
        .expect("installed view registers")
        .resolve_definition(&view_identity, crate::WorthUiQueryViewShape::Collection)
        .expect("registered identity resolves")
}

fn requirements() -> WorthUiQueryConsumerRequirements {
    WorthUiQueryConsumerRequirements::new(
        installed::operation::WorthQueryConsumerBoundaryRequirements {
            presentation: installed::operation::WorthQueryConsumerPresentationPosture::Interactive,
            allocation: installed::operation::WorthQueryConsumerAllocationPosture::Borrowed,
        },
        WorthUiQueryAllocationDetail::BorrowedFactSlice,
        crate::WorthUiQueryViewShape::Collection,
        WorthUiQueryDenialPresentation::StructuredStatus,
        WorthUiQueryInspectionRelevance::Relevant,
    )
}
