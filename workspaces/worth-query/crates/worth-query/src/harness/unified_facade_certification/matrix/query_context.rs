use super::super::lane::UnifiedFacadeLane;
use crate::application::WorthQueryApplicationFacade;
use crate::basis_lifecycle::basis_lifecycle;
use crate::facade::foundation::WorthQueryCapabilityFamily;
use crate::facade::policy::QueryContextBindingSource;
use crate::harness::fixtures::execution_preflights;

pub(super) fn query_context_lane() -> UnifiedFacadeLane {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let left_preflight = execution_preflights::direct_runtime_preflight();
    let right_preflight = execution_preflights::alternate_basis_runtime_preflight();
    let support = facade.support_matrix();
    let report = facade.support_report().expect("fixture digest must fit");
    let contexts = facade
        .query_context_capability()
        .expect("query context capability should admit");
    let left = contexts
        .capability()
        .admit_basis_context(
            basis_lifecycle().current_head(),
            QueryContextBindingSource::RuntimeCurrent(&left_preflight),
        )
        .expect("current context should admit");
    let right = contexts
        .capability()
        .admit_basis_context(
            basis_lifecycle().branch_head("branch:snapshot-2", true),
            QueryContextBindingSource::RuntimeBranch(&right_preflight),
        )
        .expect("branch context should admit");
    let _diff = contexts
        .capability()
        .bind_diff_context(&left, &right)
        .expect("query context capability should bind diff context");
    let basis_bundle = contexts
        .capability()
        .execute_basis_result_bundle(&left)
        .expect("query context basis bundle should shape");
    let left_execution = basis_bundle.execution().clone();
    let right_execution = contexts
        .capability()
        .execute_basis_context(&right)
        .expect("right branch context should execute");
    let diff = contexts
        .capability()
        .bind_diff_context(&left, &right)
        .expect("query context capability should bind diff context");
    let diff_bundle = contexts
        .capability()
        .shape_diff_result_bundle(&diff, &left_execution, &right_execution)
        .expect("query context diff bundle should shape");
    let report_profile = report
        .query_context_support_profile()
        .expect("query context support profile should be present");

    UnifiedFacadeLane::new(
        left_preflight
            .plan()
            .query()
            .validated_query_digest()
            .as_str()
            .to_string(),
        left_preflight
            .plan()
            .query()
            .plan_digest()
            .as_str()
            .to_string(),
        support.support_matrix_digest().to_string(),
        support.capability_registry().registry_digest().to_string(),
        contexts.counters(),
        WorthQueryCapabilityFamily::QueryContext,
        contexts.descriptor().status(),
        contexts.descriptor().config_section(),
    )
    .with_report_digest(
        report.report_digest().to_string(),
        report.counters().support_report_generation_count(),
    )
    .with_query_context_support_profile(
        report_profile.profile_digest().to_string(),
        report_profile
            .admitted_basis_families()
            .iter()
            .map(|family| family.as_str().to_string())
            .collect(),
        report_profile
            .admitted_comparison_families()
            .iter()
            .map(|family| family.as_str().to_string())
            .collect(),
        report_profile
            .deferred_scope_markers()
            .iter()
            .map(|marker| marker.as_str().to_string())
            .collect(),
    )
    .with_query_context_result_digests(
        basis_bundle.metadata().result_digest().to_string(),
        diff_bundle
            .metadata()
            .comparison_result_digest()
            .to_string(),
        diff_bundle.replay_digest().to_string(),
    )
}
