use super::*;

pub(in crate::harness::milestone_eight_certification) fn durable_saved_query_deferred_rejection_bundle(
) -> MilestoneEightRejectionBundle {
    let canonical = direct_detail_canonical("Alice");
    let plan = view_plan(
        &canonical,
        detail_schema_view(),
        ViewShapeDescriptor::detail(),
    );
    let saved = freeze_direct_saved_query(
        &canonical,
        &plan,
        SavedQueryFreezeContext::new(
            crate::composition::runtime_backed_query_composition_support_profile().profile_digest(),
            "query_direct",
        ),
    )
    .unwrap();
    let error = saved
        .admit_persistence_claim(SavedQueryPersistenceClaim::DurableReload)
        .expect_err("durable reload should remain deferred debt in milestone eight");

    MilestoneEightRejectionBundle {
        failure_class: MilestoneEightFailureClass::DurableSavedQueryDeferredDebt,
        failure_digest: digest_parts(&[
            format!("{:?}", error.failure_class()),
            error.message().to_string(),
        ]),
        counter_snapshot_digest: digest_parts(&[format!(
            "durable_claim:{:?}",
            error.failure_class()
        )]),
    }
}

pub(in crate::harness::milestone_eight_certification) fn grouped_hidden_refresh_forbidden_rejection_bundle(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> MilestoneEightRejectionBundle {
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
    let live = lower_view_shape_plan_to_live(&plan, basis, Some(baseline), None).unwrap();
    let error = execute_live_view_shape_change(
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
    .expect_err("grouped hidden refresh should be forbidden on the ungrouped entrypoint");

    MilestoneEightRejectionBundle {
        failure_class: MilestoneEightFailureClass::GroupedHiddenRefreshForbidden,
        failure_digest: digest_parts(&[
            format!("{:?}", error.failure_class()),
            error.message().to_string(),
        ]),
        counter_snapshot_digest: digest_parts(&[format!(
            "grouped_hidden_refresh:{:?}",
            error.failure_class()
        )]),
    }
}
