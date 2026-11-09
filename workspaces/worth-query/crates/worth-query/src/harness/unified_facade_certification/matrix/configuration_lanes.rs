use super::super::lane::UnifiedFacadeLane;
use crate::application::WorthQueryApplicationFacade;
use crate::facade::foundation::{
    WorthQueryCapabilityFamily, WorthQueryCapabilityStatus, WorthQueryConfigSectionFamily,
};
use crate::harness::fixtures::execution_preflights;

pub(super) fn workflow_section_lane() -> UnifiedFacadeLane {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let preflight = execution_preflights::direct_runtime_preflight();
    let support = facade.support_matrix();
    let report = facade.support_report().expect("fixture digest must fit");
    let (section, counters) =
        facade.resolve_config_section(WorthQueryConfigSectionFamily::Relational);
    UnifiedFacadeLane::new(
        preflight
            .plan()
            .query()
            .validated_query_digest()
            .as_str()
            .to_string(),
        preflight.plan().query().plan_digest().as_str().to_string(),
        support.support_matrix_digest().to_string(),
        support.capability_registry().registry_digest().to_string(),
        &counters,
        WorthQueryCapabilityFamily::WorkflowOrchestration,
        if section.enabled() {
            WorthQueryCapabilityStatus::Admitted
        } else {
            WorthQueryCapabilityStatus::Unsupported
        },
        section.section(),
    )
    .with_report_digest(
        report.report_digest().to_string(),
        report.counters().support_report_generation_count(),
    )
}

pub(super) fn support_sync_lane() -> UnifiedFacadeLane {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let preflight = execution_preflights::direct_runtime_preflight();
    let support = facade.support_matrix();
    let report = facade.support_report().expect("fixture digest must fit");
    let query_context = facade
        .query_context_capability()
        .expect("query context capability should be admitted in the unified facade");
    let profile = report
        .query_context_support_profile()
        .expect("query context support profile should be present");
    UnifiedFacadeLane::new(
        preflight
            .plan()
            .query()
            .validated_query_digest()
            .as_str()
            .to_string(),
        preflight.plan().query().plan_digest().as_str().to_string(),
        support.support_matrix_digest().to_string(),
        support.capability_registry().registry_digest().to_string(),
        query_context.counters(),
        WorthQueryCapabilityFamily::QueryContext,
        query_context.descriptor().status(),
        query_context.descriptor().config_section(),
    )
    .with_report_digest(
        report.report_digest().to_string(),
        report.counters().support_report_generation_count(),
    )
    .with_query_context_support_profile(
        profile.profile_digest().to_string(),
        profile
            .admitted_basis_families()
            .iter()
            .map(|family| family.as_str().to_string())
            .collect(),
        profile
            .admitted_comparison_families()
            .iter()
            .map(|family| family.as_str().to_string())
            .collect(),
        profile
            .deferred_scope_markers()
            .iter()
            .map(|marker| marker.as_str().to_string())
            .collect(),
    )
}

pub(super) fn identity_evolution_support_sync_lane() -> UnifiedFacadeLane {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let preflight = execution_preflights::direct_runtime_preflight();
    let support = facade.support_matrix();
    let report = facade.support_report().expect("fixture digest must fit");
    let identity = facade
        .identity_evolution_capability()
        .expect("identity evolution capability should be admitted in the unified facade");
    let profile = report
        .identity_evolution_support_profile()
        .expect("identity evolution support profile should be present");
    let admitted = identity
        .capability()
        .admit_query(
            crate::facade::foundation::IdentityEvolutionQueryContext::correspondence_identity_comparison_for_test(
                crate::identity::CanonicalQueryDigest::from_parts(&[format!(
                    "identity-evolution-support-sync:{}",
                    preflight.plan().query().validated_query_digest().as_str()
                )]),
                crate::facade::foundation::IdentityEvolutionComparisonBasisFamily::BranchToBranch,
                crate::identity::BasisDigest::from_parts(&["identity-left".to_string()]),
                crate::identity::BasisDigest::from_parts(&["identity-right".to_string()]),
                crate::facade::foundation::CorrespondenceIdentityComparison::advisory_between(
                    "entity:left",
                    "entity:right",
                ),
            ),
        )
        .expect("identity evolution comparison should admit");
    let execution = identity
        .capability()
        .execute_query(&admitted)
        .expect("identity evolution comparison should execute");
    UnifiedFacadeLane::new(
        preflight
            .plan()
            .query()
            .validated_query_digest()
            .as_str()
            .to_string(),
        preflight.plan().query().plan_digest().as_str().to_string(),
        support.support_matrix_digest().to_string(),
        support.capability_registry().registry_digest().to_string(),
        identity.counters(),
        WorthQueryCapabilityFamily::IdentityEvolution,
        identity.descriptor().status(),
        identity.descriptor().config_section(),
    )
    .with_report_digest(
        report.report_digest().to_string(),
        report.counters().support_report_generation_count(),
    )
    .with_identity_evolution_support_profile(
        profile.profile_digest().to_string(),
        profile
            .admitted_traversal_families()
            .iter()
            .map(|family| family.as_str().to_string())
            .collect(),
        profile
            .admitted_comparison_basis_families()
            .iter()
            .map(|family| family.as_str().to_string())
            .collect(),
        profile
            .admitted_inspector_consumable_identity_classifications()
            .iter()
            .map(|classification| classification.as_str().to_string())
            .collect(),
        profile
            .deferred_scope_markers()
            .iter()
            .map(|marker| marker.as_str().to_string())
            .collect(),
    )
    .with_identity_evolution_result_digests(
        execution.result_digest().to_string(),
        execution
            .result_bundle()
            .metadata()
            .branch_locality_digest()
            .as_str()
            .to_string(),
    )
}
