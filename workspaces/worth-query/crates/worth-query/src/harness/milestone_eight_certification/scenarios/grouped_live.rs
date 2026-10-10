use super::*;

pub(in crate::harness::milestone_eight_certification) fn table_live_bundle(
    canonical: &crate::canonicalization::CanonicalQueryBundle,
) -> MilestoneEightCertificationBundle {
    let plan = view_plan(
        canonical,
        collection_schema_view(),
        ViewShapeDescriptor::table(),
    );
    let live = lower_view_shape_plan_to_live(
        &plan,
        runtime_basis(plan.validated().query().schema_basis().clone()),
        None,
        None,
    )
    .unwrap();
    let execution = execute_live_view_shape_change(
        &live,
        &crate::live::BridgeChangeSummary::default().with_field_delta(
            crate::live::BridgeFieldDelta::new(
                "profile",
                "display_name",
                Some("Ada"),
                Some("Ada Lovelace"),
            ),
        ),
    )
    .unwrap();
    bundle_from_view_execution(
        canonical.query().digest().as_str().to_string(),
        plan.view_plan_digest().as_str().to_string(),
        canonical.result_shape().digest().as_str().to_string(),
        execution.patch_envelope().delivery_digest().to_string(),
        vec![
            format!(
                "view_patch_width:{}",
                execution.counters().view_patch_width()
            ),
            format!(
                "table_ordering_keys:{}",
                execution.counters().table_ordering_key_count()
            ),
        ],
        "artifact:none".to_string(),
        "support:none".to_string(),
    )
}

pub(in crate::harness::milestone_eight_certification) fn grouped_live_bundle(
    delta_bound: bool,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> MilestoneEightCertificationBundle {
    let canonical = direct_collection_canonical();
    let plan = view_plan(
        &canonical,
        collection_schema_view(),
        ViewShapeDescriptor::kanban_grouped(aspect_key("status")),
    );
    let basis = runtime_basis(plan.validated().query().schema_basis().clone());
    let truth_view = grouped_truth_view_for_plan(&plan, resource_request);
    let grouped_execution =
        materialize_grouped_execution_surface_from_truth_view(&plan, basis.clone(), &truth_view)
            .unwrap();
    let baseline =
        materialize_authoritative_grouped_baseline(&plan, basis.clone(), &grouped_execution)
            .unwrap();
    let member_key = baseline.desired_state().result().member_states()[0]
        .member_key()
        .to_string();
    let live = lower_view_shape_plan_to_live(&plan, basis, Some(baseline), None).unwrap();
    let change = if delta_bound {
        crate::live::BridgeChangeSummary::default()
            .with_field_delta(crate::live::BridgeFieldDelta::new(
                "identity",
                "id",
                Some(member_key.as_str()),
                Some(member_key.as_str()),
            ))
            .with_field_delta(crate::live::BridgeFieldDelta::new(
                "status",
                "lane",
                Some("todo"),
                Some("doing"),
            ))
            .with_membership_transition(true, true)
    } else {
        let mut change = crate::live::BridgeChangeSummary::default();
        for _ in 0..128 {
            change = change.with_field_delta(crate::live::BridgeFieldDelta::new(
                "profile",
                "display_name",
                Some("Ada"),
                Some("Ada Lovelace"),
            ));
        }
        change
    };
    let next_grouped_execution = if delta_bound {
        let next_truth_view = grouped_truth_view_for_plan_with_rows(
            &plan,
            &[
                grouped_row("task-1", "Ada", "doing"),
                grouped_row("task-2", "Bea", "doing"),
            ],
            resource_request,
        );
        materialize_grouped_execution_surface_from_truth_view(
            &plan,
            live.basis().clone(),
            &next_truth_view,
        )
        .unwrap()
    } else {
        materialize_grouped_execution_surface_from_truth_view(
            &plan,
            live.basis().clone(),
            &truth_view,
        )
        .unwrap()
    };
    let execution = execute_grouped_live_view_shape_change(
        admit_grouped_live_view(&live).unwrap(),
        &change,
        &next_grouped_execution,
    )
    .unwrap();

    bundle_from_view_execution(
        canonical.query().digest().as_str().to_string(),
        plan.view_plan_digest().as_str().to_string(),
        canonical.result_shape().digest().as_str().to_string(),
        execution.patch_envelope().delivery_digest().to_string(),
        vec![
            format!(
                "grouped_delta_rows:{}",
                execution.counters().grouped_delta_row_count()
            ),
            format!(
                "grouped_membership_transitions:{}",
                execution.counters().grouped_membership_transition_count()
            ),
            format!(
                "grouped_lane_count:{}",
                execution.counters().grouped_lane_count()
            ),
            format!(
                "view_family_refresh_admission_count:{}",
                execution.counters().view_family_refresh_admission_count()
            ),
            format!(
                "complexity_status_debt_count:{}",
                execution.counters().complexity_status_debt_count()
            ),
        ],
        grouped_execution.digest().to_string(),
        "support:none".to_string(),
    )
}
