use super::*;

pub(in crate::harness::milestone_eight_certification) fn rejection_rows(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Vec<MilestoneEightRejectionRow> {
    let control_lane = detail_live_bundle(&direct_detail_canonical("Alice"));
    let saved_control = saved_query_bundle(false);

    let unsupported_scope = GuidedCompositionPath::expand_detail_scopes(
        crate::authoring::RawAuthoredQuery::detail_builder(RootEntityKey::new("user").unwrap())
            .project(AspectFieldSelector::new("identity", "id").unwrap())
            .project(AspectFieldSelector::new("profile", "display_name").unwrap())
            .build()
            .unwrap(),
        detail_shape(),
        [QueryScopeDescriptor::unsupported_for_test("nope")],
    )
    .expect_err("unsupported scope should deny");
    let unsupported_template = GuidedCompositionPath::instantiate_detail_template(
        QueryTemplateDescriptor::observed_inspector_deferred_for_test(
            crate::authoring::RawAuthoredQuery::detail_builder(RootEntityKey::new("user").unwrap())
                .project(AspectFieldSelector::new("identity", "id").unwrap())
                .project(AspectFieldSelector::new("profile", "display_name").unwrap())
                .build()
                .unwrap(),
            detail_shape(),
        ),
        TemplateBindingSet::new(),
    )
    .expect_err("unsupported template family should deny");

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
    let descriptor = SavedQueryReuseDescriptor::new(
        saved.metadata().schema_basis_digest().clone(),
        saved.metadata().basis_family().cloned(),
        saved.metadata().template_binding_digest().cloned(),
        saved.metadata().template_slot_count(),
        saved.metadata().view_shape_digest().clone(),
        saved.metadata().view_shape_family(),
        saved.metadata().result_shape_family().clone(),
        saved.metadata().composition_digest().clone(),
        saved.metadata().scope_lineage_digest().cloned(),
        "different-support",
        saved.metadata().capability_family_identity().to_string(),
    );
    let saved_drift = evaluate_saved_query_reuse(&saved, &descriptor);
    let SavedQueryReuseOutcome::Denied(saved_denial) = saved_drift else {
        panic!("saved query support drift should deny");
    };

    vec![
        MilestoneEightRejectionRow {
            row_name: "unsupported-scope-family",
            perturbation_class: MilestoneEightPerturbationClass::UnsupportedScopeFamily,
            control_lane: control_lane.clone(),
            hostile_lane: MilestoneEightRejectionBundle {
                failure_class: MilestoneEightFailureClass::UnsupportedScopeFamily,
                failure_digest: digest_parts(&[
                    format!("{:?}", unsupported_scope.failure_class()),
                    unsupported_scope.message().to_string(),
                ]),
                counter_snapshot_digest: digest_parts(&[format!(
                    "scope_denial:{:?}",
                    unsupported_scope.failure_class()
                )]),
            },
            parity_lane: control_lane.clone(),
        },
        MilestoneEightRejectionRow {
            row_name: "unsupported-template-family",
            perturbation_class: MilestoneEightPerturbationClass::UnsupportedTemplateFamily,
            control_lane: control_lane.clone(),
            hostile_lane: MilestoneEightRejectionBundle {
                failure_class: MilestoneEightFailureClass::UnsupportedTemplateFamily,
                failure_digest: digest_parts(&[
                    format!("{:?}", unsupported_template.failure_class()),
                    unsupported_template.message().to_string(),
                ]),
                counter_snapshot_digest: digest_parts(&[format!(
                    "template_denial:{:?}",
                    unsupported_template.failure_class()
                )]),
            },
            parity_lane: control_lane,
        },
        MilestoneEightRejectionRow {
            row_name: "saved-query-support-profile-drift",
            perturbation_class: MilestoneEightPerturbationClass::SavedQuerySupportProfileDrift,
            control_lane: saved_control.clone(),
            hostile_lane: MilestoneEightRejectionBundle {
                failure_class: MilestoneEightFailureClass::SavedQuerySupportProfileDrift,
                failure_digest: digest_parts(&[
                    format!("{:?}", saved_denial.failure_class()),
                    format!("{:?}", saved_denial.overall()),
                ]),
                counter_snapshot_digest: digest_parts(
                    &saved_denial
                        .matrix()
                        .rows()
                        .iter()
                        .map(|row| format!("{:?}:{:?}", row.dimension(), row.legality()))
                        .collect::<Vec<_>>(),
                ),
            },
            parity_lane: saved_control,
        },
        MilestoneEightRejectionRow {
            row_name: "durable-saved-query-deferred-debt",
            perturbation_class: MilestoneEightPerturbationClass::DurableSavedQueryDeferredDebt,
            control_lane: saved_query_bundle(false),
            hostile_lane: durable_saved_query_deferred_rejection_bundle(),
            parity_lane: saved_query_bundle(false),
        },
        MilestoneEightRejectionRow {
            row_name: "grouped-hidden-refresh-forbidden",
            perturbation_class: MilestoneEightPerturbationClass::GroupedHiddenRefreshForbidden,
            control_lane: grouped_live_bundle(true, resource_request),
            hostile_lane: grouped_hidden_refresh_forbidden_rejection_bundle(resource_request),
            parity_lane: grouped_live_bundle(true, resource_request),
        },
    ]
}
