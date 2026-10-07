use crate::runtime::{
    WorthQueryGraphReadAccessAdmissionPosture, WorthQueryGraphReadAccessRequirementKind,
    WorthQueryReadFamily, WorthQueryWorkspace,
};

use crate::runtime::tests::graph_read_access::support;

use support::graph_index_inventory::runtime_profiles::{
    default_graph_support_workspace, profile_with_ephemeral_graph_support,
    profile_without_graph_support, workspace_with_graph_support,
};
use support::graph_read_access::hostile_graph_fixture::seed_hostile_frontier_graph;
use support::graph_read_access::read_surface_assertions::{
    assert_admitted_summary, assert_pre_execution_graph_access_denial,
    assert_success_counters_are_executor_observed, read_composition_denial,
};
use support::graph_read_access::read_surface_declarations::{
    dense_over_budget_family, execute_dense_over_budget_compose_read,
    execute_graph_access_compose_read, execute_unregistered_domain_operation_compose_read,
    graph_access_family, unregistered_domain_operation_family,
};
use support::public_bridge_runtime::workspace;

#[test]
fn graph_access_read_fronts_expose_same_admitted_access_evidence() {
    let mut execute_workspace = workspace("graph-read-access.read-surface.execute");
    let execute_family = graph_access_family(&mut execute_workspace, "read-surface-parity");
    let execute_result = execute_workspace
        .execute_read_family(&execute_family)
        .expect("execute_read_family should execute through admitted access plan");
    let execute_summary = execute_result
        .receipt()
        .graph_read_access_summary()
        .expect("execute_read_family receipt should expose access summary");

    let mut intent_workspace = workspace("graph-read-access.read-surface.intent");
    let intent_family = graph_access_family(&mut intent_workspace, "read-surface-parity");
    let intent_result = intent_workspace
        .read_family_intent(&intent_family)
        .execute()
        .expect("intent helper should execute through admitted access plan");
    let intent_summary = intent_result
        .receipt()
        .graph_read_access_summary()
        .expect("intent receipt should expose access summary");

    let mut compose_workspace = workspace("graph-read-access.read-surface.compose");
    let compose_result = execute_graph_access_compose_read(&mut compose_workspace)
        .expect("compose_read should execute through admitted access plan");
    let compose_summary = compose_result
        .receipt()
        .graph_read_access_summary()
        .expect("compose_read receipt should expose access summary");

    assert_admitted_summary(execute_summary);
    assert_admitted_summary(intent_summary);
    assert_admitted_summary(compose_summary);
    assert_eq!(
        execute_summary.admission_digest(),
        intent_summary.admission_digest()
    );
    assert_eq!(
        execute_summary.admission_digest(),
        compose_summary.admission_digest()
    );
    assert_eq!(execute_summary.plan_digest(), intent_summary.plan_digest());
    assert_eq!(execute_summary.plan_digest(), compose_summary.plan_digest());
    assert_eq!(
        execute_summary.requirement_set_digest(),
        compose_summary.requirement_set_digest()
    );
    assert_eq!(
        execute_summary.graph_index_inventory_match_report_digest(),
        compose_summary.graph_index_inventory_match_report_digest()
    );
    assert_eq!(
        execute_summary.execution_strategy(),
        compose_summary.execution_strategy()
    );
}

#[test]
fn one_call_intent_consumes_plan_available_through_explicit_review() {
    let mut workspace = workspace("graph-read-access.read-surface.helper-honesty");
    seed_hostile_frontier_graph(&mut workspace, "read-surface-helper-honesty");
    let family = graph_access_family(&mut workspace, "read-surface-helper-honesty");
    let (reviewed_plan_digest, reviewed_admission_digest) = {
        let review = workspace
            .read_family_intent(&family)
            .review()
            .expect("read review should be admitted");
        let planned = review
            .graph_read_access_plan()
            .expect("review should expose admitted graph access plan");
        (
            planned.digest().to_string(),
            planned.admission().digest().to_string(),
        )
    };
    let result = workspace
        .read_family_intent(&family)
        .execute()
        .expect("one-call intent should consume admitted access plan");
    let summary = result
        .receipt()
        .graph_read_access_summary()
        .expect("receipt should expose access summary");
    let counters = result
        .receipt()
        .graph_read_access_complexity_counters()
        .expect("receipt should expose access complexity counters");

    assert_eq!(summary.plan_digest(), reviewed_plan_digest);
    assert_eq!(summary.admission_digest(), reviewed_admission_digest);
    assert_success_counters_are_executor_observed(counters);
    assert_eq!(
        counters.planned_access_step_count(),
        counters.consumed_access_step_count()
    );
    assert!(
        result
            .receipt()
            .graph_read_access_plan_consumption()
            .expect("receipt should expose plan consumption")
            .execution_counters()
            .materialized_row_count()
            > 0
    );
}

#[test]
fn basis_context_intent_fronts_keep_access_plan_consumption_visible() {
    let mut workspace =
        default_graph_support_workspace("graph-read-access.read-surface.basis-context");
    let family = graph_access_family(&mut workspace, "read-surface-basis-context");
    let context = current_context_for_family(&workspace, &family);
    let reviewed_plan_digest = {
        let review = workspace
            .read_family_in_basis_context_intent(&family, &context)
            .review()
            .expect("basis-context review should admit");
        review
            .graph_read_access_plan()
            .expect("basis-context review should expose access plan")
            .digest()
            .to_string()
    };
    let explicit_result = workspace
        .read_family_in_basis_context_intent(&family, &context)
        .admit()
        .expect("basis-context intent should admit")
        .execute()
        .expect("basis-context admitted intent should execute");
    let one_call_result = workspace
        .read_family_in_basis_context_intent(&family, &context)
        .execute()
        .expect("basis-context one-call intent should execute");

    for result in [&explicit_result, &one_call_result] {
        let summary = result
            .receipt()
            .graph_read_access_summary()
            .expect("basis-context receipt should expose access summary");
        let counters = result
            .receipt()
            .graph_read_access_complexity_counters()
            .expect("basis-context receipt should expose complexity counters");

        assert_eq!(summary.plan_digest(), reviewed_plan_digest);
        assert_eq!(
            summary.admission_posture(),
            &WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed
        );
        assert_success_counters_are_executor_observed(counters);
    }
}

#[test]
fn over_budget_denials_publish_admission_envelopes_across_fronts() {
    let mut family_workspace = workspace("graph-read-access.read-surface.denial.family");
    let family = dense_over_budget_family(&mut family_workspace, "read-surface-denial");
    assert_pre_execution_graph_access_denial(&read_composition_denial(
        family_workspace
            .execute_read_family(&family)
            .expect_err("over-budget reusable family should deny"),
    ));

    let mut intent_workspace = workspace("graph-read-access.read-surface.denial.intent");
    let intent_family = dense_over_budget_family(&mut intent_workspace, "read-surface-denial");
    assert_pre_execution_graph_access_denial(&read_composition_denial(
        intent_workspace
            .read_family_intent(&intent_family)
            .execute()
            .expect_err("over-budget intent should deny"),
    ));

    let mut review_workspace = workspace("graph-read-access.read-surface.denial.review");
    let review_family = dense_over_budget_family(&mut review_workspace, "read-surface-denial");
    let review = review_workspace
        .read_family_intent(&review_family)
        .review()
        .expect("intent review should admit before graph access planning");
    assert_pre_execution_graph_access_denial(&read_composition_denial(
        review
            .graph_read_access_plan()
            .expect_err("over-budget reviewed plan should deny"),
    ));

    let mut compose_workspace = workspace("graph-read-access.read-surface.denial.compose");
    assert_pre_execution_graph_access_denial(&read_composition_denial(
        execute_dense_over_budget_compose_read(&mut compose_workspace)
            .expect_err("over-budget compose_read should deny"),
    ));
}

#[test]
fn missing_capability_denials_publish_admission_envelopes_across_fronts() {
    let support_profile = profile_without_graph_support(
        WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
    );
    let mut family_workspace = workspace_with_graph_support(
        "graph-read-access.read-surface.missing-capability.family",
        support_profile.clone(),
    );
    let family = graph_access_family(&mut family_workspace, "read-surface-missing-capability");
    assert_pre_execution_graph_access_denial(&read_composition_denial(
        family_workspace
            .execute_read_family(&family)
            .expect_err("missing graph support should deny"),
    ));

    let mut compose_workspace = workspace_with_graph_support(
        "graph-read-access.read-surface.missing-capability.compose",
        support_profile,
    );
    assert_pre_execution_graph_access_denial(&read_composition_denial(
        execute_graph_access_compose_read(&mut compose_workspace)
            .expect_err("missing graph support compose_read should deny"),
    ));
}

#[test]
fn unregistered_domain_operation_denials_preserve_capability_proof_across_fronts() {
    let mut family_workspace = workspace("graph-read-access.read-surface.unregistered.family");
    let family =
        unregistered_domain_operation_family(&mut family_workspace, "read-surface-unregistered");
    let family_denial = read_composition_denial(
        family_workspace
            .execute_read_family(&family)
            .expect_err("unregistered domain operation family should deny"),
    );
    assert_unregistered_operation_admission(assert_pre_execution_graph_access_denial(
        &family_denial,
    ));

    let mut compose_workspace = workspace("graph-read-access.read-surface.unregistered.compose");
    let compose_denial = read_composition_denial(
        execute_unregistered_domain_operation_compose_read(&mut compose_workspace)
            .expect_err("unregistered domain operation compose_read should deny"),
    );
    assert_unregistered_operation_admission(assert_pre_execution_graph_access_denial(
        &compose_denial,
    ));
}

#[test]
fn helper_fronts_prove_no_per_result_neighbor_lookup_loop_from_executor_counters() {
    let mut workspace = workspace_with_graph_support(
        "graph-read-access.read-surface.no-n-plus-one",
        profile_with_ephemeral_graph_support(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
    );
    seed_hostile_frontier_graph(&mut workspace, "read-surface-no-n-plus-one");
    let family = graph_access_family(&mut workspace, "read-surface-no-n-plus-one");
    let result = workspace
        .execute_read_family(&family)
        .expect("helper should execute through admitted bounded access plan");
    let counters = result
        .receipt()
        .graph_read_access_complexity_counters()
        .expect("receipt should expose complexity counters");

    assert_success_counters_are_executor_observed(counters);
    assert_eq!(counters.ephemeral_index_allocation_count(), 1);
    assert!(
        result
            .receipt()
            .graph_read_access_plan_consumption()
            .expect("receipt should expose plan consumption")
            .execution_counters()
            .materialized_row_count()
            > 0
    );
}

#[test]
fn explicit_access_plan_execution_denies_wrong_family_with_typed_binding_mismatch() {
    let mut plan_workspace = workspace("graph-read-access.read-surface.explicit-plan.source");
    let planned_family = graph_access_family(&mut plan_workspace, "explicit-plan-source");
    let plan = plan_workspace
        .read_family_intent(&planned_family)
        .review()
        .expect("source family should review")
        .graph_read_access_plan()
        .expect("source family should produce an admitted access plan");
    let planned_read_graph_digest = plan
        .admission()
        .requirement_set()
        .read_graph_digest()
        .render_hex();
    let provided_plan_digest = plan.digest().to_string();
    let provided_admission_digest = plan.admission().digest().to_string();

    let mut execution_workspace = workspace("graph-read-access.read-surface.explicit-plan.target");
    let target_family =
        unregistered_domain_operation_family(&mut execution_workspace, "explicit-plan-target");
    let target_read_graph_digest = target_family.read_graph().digest().to_string();
    let denial = read_composition_denial(
        execution_workspace
            .execute_read_family_with_access_plan(&target_family, plan)
            .expect_err("plan admitted for a different read graph should deny before execution"),
    );
    let mismatch = denial
        .access_plan_binding_mismatch()
        .expect("wrong-plan denial should carry typed binding mismatch");

    assert_eq!(
        mismatch.admitted_read_graph_digest(),
        planned_read_graph_digest
    );
    assert_eq!(
        mismatch.execution_read_graph_digest(),
        target_read_graph_digest
    );
    assert_eq!(mismatch.provided_plan_digest(), provided_plan_digest);
    assert_eq!(
        mismatch.provided_admission_digest(),
        provided_admission_digest
    );
    assert!(denial.graph_read_access_execution_counters().is_none());
}

fn current_context_for_family(
    workspace: &WorthQueryWorkspace,
    family: &WorthQueryReadFamily,
) -> crate::facade::policy::ScopedQueryBasisContext {
    use crate::facade::foundation::{
        basis_lifecycle, preflight_execution_basis, resolve_runtime_current_snapshot_basis,
    };
    use crate::facade::policy::{admit_query_basis_context, QueryContextBindingSource};

    let basis = resolve_runtime_current_snapshot_basis(
        workspace
            .snapshot_identity()
            .expect("the fixture owner remains open")
            .evidence_identity(),
        family.read_graph().schema_basis_authority(),
    )
    .expect("snapshot basis should resolve");
    let preflight = preflight_execution_basis(family.read_graph().execution_plan().clone(), basis)
        .expect("query basis should preflight");
    admit_query_basis_context(
        basis_lifecycle().current_head(),
        QueryContextBindingSource::RuntimeCurrent(&preflight),
    )
    .expect("current basis context should admit")
}

fn assert_unregistered_operation_admission(
    admission: &crate::runtime::WorthQueryGraphReadAccessAdmission,
) {
    assert!(admission
        .graph_index_inventory_match_report()
        .includes_admission_posture(
            &WorthQueryGraphReadAccessAdmissionPosture::AccessCapabilityRegistrationRequired,
        ));
    let operation_requirement = admission
        .requirement_set()
        .rows()
        .iter()
        .find(|row| {
            row.kind()
                == &WorthQueryGraphReadAccessRequirementKind::DomainOperationCapabilityRegistration
        })
        .and_then(|row| row.operation_capability_requirement())
        .expect("admission should preserve the missing domain-operation capability proof");
    assert_eq!(
        operation_requirement.operation_name(),
        "worth.geometry.visible_face_neighborhood"
    );
    assert_eq!(operation_requirement.domain_owner(), "worth.geometry");
    assert_eq!(
        operation_requirement.support_family(),
        "worth.geometry.visible_face_neighborhood.access"
    );
}
