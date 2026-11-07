use super::super::lane::UnifiedFacadeLane;
use crate::application::WorthQueryApplicationFacade;
use crate::facade::foundation::{
    IdentityEvolutionQueryContext, LineageTraversalDescriptor, WorthQueryCapabilityFamily,
};
use crate::facade::policy::{PreviewEvaluationClass, PreviewSessionQueryContext};
use crate::facade::runtime::{
    WorkflowAuthorityTargetFamily, WorkflowBindingSource, WorkflowBudgetClass, WorkflowCostClass,
    WorkflowDeclarationFamily, WorkflowDeclarationRequest, WorkflowFreshnessPolicy,
};
use crate::harness::fixtures::{execution_preflights, preview_bridge::active_preview_artifacts};

pub(super) fn identity_evolution_lane() -> UnifiedFacadeLane {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let preflight = execution_preflights::direct_runtime_preflight();
    let support = facade.support_matrix();
    let report = facade.support_report().expect("fixture digest must fit");
    let identity = facade
        .identity_evolution_capability()
        .expect("identity evolution should admit");
    let admitted = identity
        .capability()
        .admit_query(IdentityEvolutionQueryContext::lineage_traversal_for_test(
            crate::identity::CanonicalQueryDigest::from_parts(&[format!(
                "unified-facade-identity-evolution:{}",
                preflight.plan().query().validated_query_digest().as_str()
            )]),
            crate::identity::BasisDigest::from_parts(&[
                "unified-facade-identity-evolution".to_string()
            ]),
            LineageTraversalDescriptor::direct_replacement("entity:replacement"),
        ))
        .expect("identity evolution should admit a direct replacement query");
    let execution = identity
        .capability()
        .execute_query(&admitted)
        .expect("identity evolution should execute");
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

pub(super) fn query_read_lane() -> UnifiedFacadeLane {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let preflight = execution_preflights::direct_runtime_preflight();
    let support = facade.support_matrix();
    let report = facade.support_report().expect("fixture digest must fit");
    let query_reads = facade
        .query_read_capability()
        .expect("query reads should admit");
    let _execution = query_reads
        .capability()
        .execute_preflight(&preflight)
        .expect("query reads should execute");
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
        query_reads.counters(),
        WorthQueryCapabilityFamily::QueryRead,
        query_reads.descriptor().status(),
        query_reads.descriptor().config_section(),
    )
    .with_report_digest(
        report.report_digest().to_string(),
        report.counters().support_report_generation_count(),
    )
}

pub(super) fn live_lane() -> UnifiedFacadeLane {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let preflight = execution_preflights::direct_runtime_preflight();
    let support = facade.support_matrix();
    let report = facade.support_report().expect("fixture digest must fit");
    let live = facade
        .live_query_capability()
        .expect("live queries should admit");
    let _plan = live
        .capability()
        .promote_preflight(&preflight)
        .expect("live capability should promote admitted preflight");
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
        live.counters(),
        WorthQueryCapabilityFamily::LiveQuery,
        live.descriptor().status(),
        live.descriptor().config_section(),
    )
    .with_report_digest(
        report.report_digest().to_string(),
        report.counters().support_report_generation_count(),
    )
}

pub(super) fn preview_lane() -> UnifiedFacadeLane {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let preflight = execution_preflights::direct_runtime_preflight();
    let support = facade.support_matrix();
    let report = facade.support_report().expect("fixture digest must fit");
    let preview = facade
        .preview_query_capability()
        .expect("preview sessions should admit");
    let (_runtime, active, execution_record) = active_preview_artifacts("unified-facade");
    let _preview_binding = preview
        .capability()
        .bind_preflight(
            preflight.clone(),
            PreviewSessionQueryContext::active(
                &active,
                &execution_record,
                PreviewEvaluationClass::promotion_eligible(),
            ),
        )
        .expect("preview capability should bind");
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
        preview.counters(),
        WorthQueryCapabilityFamily::PreviewSession,
        preview.descriptor().status(),
        preview.descriptor().config_section(),
    )
    .with_report_digest(
        report.report_digest().to_string(),
        report.counters().support_report_generation_count(),
    )
}

pub(super) fn workflow_lane() -> UnifiedFacadeLane {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let preflight = execution_preflights::direct_runtime_preflight();
    let support = facade.support_matrix();
    let report = facade.support_report().expect("fixture digest must fit");
    let preview = facade
        .preview_query_capability()
        .expect("preview sessions should admit");
    let workflow = facade
        .workflow_query_capability()
        .expect("workflow orchestration should admit");
    let (_runtime, active, execution_record) = active_preview_artifacts("unified-facade");
    let preview_binding = preview
        .capability()
        .bind_preflight(
            preflight.clone(),
            PreviewSessionQueryContext::active(
                &active,
                &execution_record,
                PreviewEvaluationClass::promotion_eligible(),
            ),
        )
        .expect("preview capability should bind");
    let foundation = crate::preview::admit_preview_workflow_foundation(&preview_binding)
        .expect("workflow foundation should admit");
    let workflow_binding = workflow
        .capability()
        .bind_context(WorkflowBindingSource::PreviewFoundation(&foundation))
        .expect("workflow binding should admit");
    let _declaration = workflow
        .capability()
        .admit_declaration(
            &workflow_binding,
            WorkflowDeclarationRequest::new(
                WorkflowDeclarationFamily::ConflictInspectionNarrow,
                WorkflowAuthorityTargetFamily::QueryInspection,
                WorkflowCostClass::InspectionNarrow,
                WorkflowBudgetClass::InspectionBounded,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
        )
        .expect("workflow declaration should admit");
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
        workflow.counters(),
        WorthQueryCapabilityFamily::WorkflowOrchestration,
        workflow.descriptor().status(),
        workflow.descriptor().config_section(),
    )
    .with_report_digest(
        report.report_digest().to_string(),
        report.counters().support_report_generation_count(),
    )
}

pub(super) fn historical_lane() -> UnifiedFacadeLane {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let preflight = execution_preflights::direct_runtime_preflight();
    let support = facade.support_matrix();
    let report = facade.support_report().expect("fixture digest must fit");
    let historical = facade
        .historical_query_capability()
        .expect("historical capability should be admitted in the unified facade");
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
        historical.counters(),
        WorthQueryCapabilityFamily::HistoricalEvaluation,
        historical.descriptor().status(),
        historical.descriptor().config_section(),
    )
    .with_report_digest(
        report.report_digest().to_string(),
        report.counters().support_report_generation_count(),
    )
}
