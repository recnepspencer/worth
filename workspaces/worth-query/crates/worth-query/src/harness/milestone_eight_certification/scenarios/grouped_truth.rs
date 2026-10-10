use super::*;

pub(in crate::harness::milestone_eight_certification) fn grouped_truth_view_bundle(
    rows: &[GroupedRowFixture],
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> MilestoneEightCertificationBundle {
    let canonical = direct_collection_canonical();
    let plan = view_plan(
        &canonical,
        collection_schema_view(),
        ViewShapeDescriptor::kanban_grouped(aspect_key("status")),
    );
    let truth_view = grouped_truth_view_for_plan_with_rows(&plan, rows, resource_request);

    bundle_from_view_execution(
        canonical.query().digest().as_str().to_string(),
        plan.view_plan_digest().as_str().to_string(),
        canonical.result_shape().digest().as_str().to_string(),
        truth_view.digest().as_str().to_string(),
        vec![
            format!("members:{}", truth_view.members().len()),
            format!("grouping:{}", truth_view.contract().grouping_aspect()),
            format!("truth_view_digest:{}", truth_view.truth_view_digest()),
        ],
        truth_view.digest().as_str().to_string(),
        "support:none".to_string(),
    )
}

pub(in crate::harness::milestone_eight_certification) fn grouped_execution_surface_bundle(
    rows: &[GroupedRowFixture],
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> MilestoneEightCertificationBundle {
    let canonical = direct_collection_canonical();
    let plan = view_plan(
        &canonical,
        collection_schema_view(),
        ViewShapeDescriptor::kanban_grouped(aspect_key("status")),
    );
    let basis = runtime_basis(plan.validated().query().schema_basis().clone());
    let truth_view = grouped_truth_view_for_plan_with_rows(&plan, rows, resource_request);
    let grouped_execution =
        materialize_grouped_execution_surface_from_truth_view(&plan, basis, &truth_view).unwrap();

    bundle_from_view_execution(
        canonical.query().digest().as_str().to_string(),
        plan.view_plan_digest().as_str().to_string(),
        canonical.result_shape().digest().as_str().to_string(),
        grouped_execution.digest().to_string(),
        vec![
            format!("members:{}", grouped_execution.member_rows().len()),
            format!(
                "truth_view:{}",
                grouped_execution.truth_view_evidence_identity().as_str()
            ),
        ],
        grouped_execution.digest().to_string(),
        "support:none".to_string(),
    )
}

pub(in crate::harness::milestone_eight_certification) fn grouped_payload_rediscovery_free_bundle(
    rows: &[GroupedRowFixture],
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> MilestoneEightCertificationBundle {
    let canonical = direct_collection_canonical();
    let plan = view_plan(
        &canonical,
        collection_schema_view(),
        ViewShapeDescriptor::kanban_grouped(aspect_key("status")),
    );
    let basis = runtime_basis(plan.validated().query().schema_basis().clone());
    let truth_view = grouped_truth_view_for_plan_with_rows(&plan, rows, resource_request);
    let grouped_execution =
        materialize_grouped_execution_surface_from_truth_view(&plan, basis.clone(), &truth_view)
            .unwrap();
    let baseline =
        materialize_authoritative_grouped_baseline(&plan, basis, &grouped_execution).unwrap();

    bundle_from_view_execution(
        canonical.query().digest().as_str().to_string(),
        plan.view_plan_digest().as_str().to_string(),
        canonical.result_shape().digest().as_str().to_string(),
        baseline.desired_state().digest().to_string(),
        vec![
            format!("members:{}", baseline.desired_state().result().row_count()),
            format!("truth_view:{}", truth_view.digest().as_str()),
            format!("grouped_execution:{}", grouped_execution.digest()),
            format!("baseline:{}", baseline.desired_state().digest()),
        ],
        grouped_execution.digest().to_string(),
        "support:none".to_string(),
    )
}
