use crate::application::{
    ConfigurationAdmissionFailureClass, WorthQueryApplicationFacade, WorthQueryCapabilityFamily,
    WorthQueryCapabilityStatus, WorthQueryConfig, WorthQueryConfigSectionFamily,
    WorthQueryFacadeFailureClass, WorthQueryQueryConfig, WorthQueryRelationalConfig,
    WorthQuerySignalConfig,
};
use crate::harness::fixtures::{execution_preflights, preview_bridge::active_preview_artifacts};
use crate::preview::{PreviewEvaluationClass, PreviewSessionQueryContext};
use crate::workflow::{
    WorkflowAuthorityTargetFamily, WorkflowBindingSource, WorkflowBudgetClass, WorkflowCostClass,
    WorkflowDeclarationFamily, WorkflowDeclarationRequest, WorkflowFreshnessPolicy,
};

mod historical;

#[test]
fn facade_support_matrix_stays_in_sync_with_capability_admission() {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let support = facade.support_matrix();
    let report = facade.support_report().expect("fixture digest must fit");

    let live_descriptor = support
        .descriptor(WorthQueryCapabilityFamily::LiveQuery)
        .expect("support matrix should include live query capability");
    assert_eq!(
        live_descriptor.status(),
        WorthQueryCapabilityStatus::Admitted
    );
    assert!(facade.live_query_capability().is_ok());

    let durability_descriptor = support
        .descriptor(WorthQueryCapabilityFamily::DurableArtifacts)
        .expect("support matrix should include durability capability");
    assert_eq!(
        durability_descriptor.status(),
        WorthQueryCapabilityStatus::DeferredDebt
    );
    let deferred = facade
        .durable_artifact_capability()
        .expect_err("durable artifacts should remain deferred debt");
    assert_eq!(
        deferred.failure_class(),
        WorthQueryFacadeFailureClass::DeferredCapabilityFamily
    );
    assert_eq!(
        deferred.counters().unsupported_composition_denial_count(),
        0
    );
    assert_eq!(deferred.counters().deferred_capability_denial_count(), 1);
    assert_eq!(
        report.support_matrix().support_matrix_digest(),
        support.support_matrix_digest()
    );
    assert_eq!(
        report.validated_config_digest(),
        facade.validated_config().validated_digest()
    );
    assert_eq!(report.counters().support_report_generation_count(), 1);
    assert_eq!(report.deferred_capability_count(), 1);
    assert!(report.admitted_capability_count() >= 1);
    assert!(report
        .admitted_capability_families()
        .contains(&WorthQueryCapabilityFamily::QueryRead));
    assert!(report
        .admitted_capability_families()
        .contains(&WorthQueryCapabilityFamily::QueryComposition));
    assert_eq!(
        report.deferred_capability_families(),
        &[WorthQueryCapabilityFamily::DurableArtifacts]
    );
    assert_eq!(
        support.admitted_capability_count(),
        report.admitted_capability_count()
    );
    assert_eq!(
        support.deferred_capability_count(),
        report.deferred_capability_count()
    );
    assert_eq!(
        support.unsupported_capability_count(),
        report.unsupported_capability_count()
    );
    assert_eq!(report.section_postures().len(), 5);
    assert!(report.section_postures().iter().any(|posture| {
        posture.section() == WorthQueryConfigSectionFamily::Signal
            && posture.owner() == crate::application::WorthQuerySubsystemOwner::Signal
            && posture.enabled()
    }));
    assert!(!report.report_digest().is_empty());
}

#[test]
fn query_composition_capability_is_admitted_with_query_section() {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let composition = facade
        .query_composition_capability()
        .expect("runtime-backed facade should admit query composition");

    assert_eq!(
        composition.admission().descriptor().family(),
        WorthQueryCapabilityFamily::QueryComposition
    );
    assert_eq!(composition.counters().capability_lookup_count(), 1);
}

#[test]
fn query_read_capability_executes_runtime_preflight() {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let query_reads = facade
        .query_read_capability()
        .expect("runtime-backed facade should admit query reads");
    let admission = query_reads.admission();
    let preflight = execution_preflights::direct_runtime_preflight();
    let execution = query_reads
        .capability()
        .execute_preflight(&preflight)
        .expect("query read capability should execute admitted preflight");

    assert_eq!(
        execution.report().query_digest().as_str(),
        preflight.plan().query().validated_query_digest().as_str()
    );
    assert_eq!(
        admission.descriptor().family(),
        WorthQueryCapabilityFamily::QueryRead
    );
    assert!(!admission.decision_digest().is_empty());
    assert_eq!(
        admission.validated_config_digest(),
        facade.validated_config().validated_digest()
    );
    assert_eq!(query_reads.counters().capability_lookup_count(), 1);
    assert_eq!(
        query_reads
            .counters()
            .configuration_section_resolution_count(),
        1
    );
}

#[test]
fn durable_artifact_admission_fails_as_deferred_decision() {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let error = facade
        .durable_artifact_capability()
        .expect_err("durable artifact admission should remain deferred");

    assert_eq!(
        error.failure_class(),
        crate::application::CapabilityAdmissionFailureClass::DeferredCapabilityFamily
    );
    assert_eq!(
        error.capability_family(),
        Some(WorthQueryCapabilityFamily::DurableArtifacts)
    );
    assert_eq!(error.counters().deferred_capability_denial_count(), 1);
    assert_eq!(error.counters().unsupported_composition_denial_count(), 0);
}

#[test]
fn preview_workflow_and_historical_decisions_preserve_family_and_validated_config() {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();

    let preview = facade
        .preview_query_capability()
        .expect("preview capability should admit")
        .admission()
        .clone();
    let workflow = facade
        .workflow_query_capability()
        .expect("workflow capability should admit")
        .admission()
        .clone();
    let historical = facade
        .historical_query_capability()
        .expect("historical capability should admit")
        .admission()
        .clone();

    assert_eq!(
        preview.descriptor().family(),
        WorthQueryCapabilityFamily::PreviewSession
    );
    assert_eq!(
        workflow.descriptor().family(),
        WorthQueryCapabilityFamily::WorkflowOrchestration
    );
    assert_eq!(
        historical.descriptor().family(),
        WorthQueryCapabilityFamily::HistoricalEvaluation
    );
    assert_eq!(
        preview.validated_config_digest(),
        facade.validated_config().validated_digest()
    );
    assert_eq!(
        workflow.validated_config_digest(),
        facade.validated_config().validated_digest()
    );
    assert_eq!(
        historical.validated_config_digest(),
        facade.validated_config().validated_digest()
    );
}

#[test]
fn preview_and_workflow_capabilities_compose_without_bypassing_authority() {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let preview = facade
        .preview_query_capability()
        .expect("runtime-backed facade should admit preview sessions");
    let workflow = facade
        .workflow_query_capability()
        .expect("runtime-backed facade should admit workflow orchestration");
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, execution_record) = active_preview_artifacts("app-facade");
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
        .expect("preview capability should bind admitted preview context");
    let foundation = crate::preview::admit_preview_workflow_foundation(&preview_binding)
        .expect("preview workflow foundation should admit");
    let workflow_binding = workflow
        .capability()
        .bind_context(WorkflowBindingSource::PreviewFoundation(&foundation))
        .expect("workflow capability should bind preview workflow foundation");
    let declaration = workflow
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
        .expect("workflow capability should admit a query inspection declaration");

    assert_eq!(
        declaration.report().declaration_family(),
        &WorkflowDeclarationFamily::ConflictInspectionNarrow
    );
}

#[test]
fn live_capability_disables_typed_and_early_when_config_section_is_off() {
    let facade = WorthQueryApplicationFacade::new(
        WorthQueryConfig::runtime_backed_default().with_signal(WorthQuerySignalConfig::disabled()),
    )
    .expect("disabling live alone should preserve a valid facade config");
    let support = facade.support_matrix();
    assert_eq!(
        support
            .descriptor(WorthQueryCapabilityFamily::LiveQuery)
            .expect("live capability descriptor should exist")
            .status(),
        WorthQueryCapabilityStatus::Unsupported
    );

    let error = facade
        .live_query_capability()
        .expect_err("disabled live section should deny live capability admission");
    assert_eq!(
        error.failure_class(),
        WorthQueryFacadeFailureClass::MissingOwningSection
    );
    assert_eq!(
        error.capability_family(),
        Some(WorthQueryCapabilityFamily::LiveQuery)
    );
    assert_eq!(error.counters().capability_lookup_count(), 1);
    assert_eq!(error.counters().unsupported_composition_denial_count(), 1);
    assert_eq!(error.counters().deferred_capability_denial_count(), 0);
}

#[test]
fn workflow_capability_can_be_disabled_inside_enabled_relational_section() {
    let facade = WorthQueryApplicationFacade::new(
        WorthQueryConfig::runtime_backed_default().with_relational(
            WorthQueryRelationalConfig::enabled()
                .with_workflow_orchestration(false)
                .with_historical_evaluation(true),
        ),
    )
    .expect("disabling workflow within relational should preserve a valid facade config");

    let error = facade
        .workflow_query_capability()
        .expect_err("disabled workflow capability should deny workflow admission");
    assert_eq!(
        error.failure_class(),
        WorthQueryFacadeFailureClass::InvalidComposedSupportPosture
    );
    assert_eq!(
        error.capability_family(),
        Some(WorthQueryCapabilityFamily::WorkflowOrchestration)
    );
    assert_eq!(error.counters().unsupported_composition_denial_count(), 1);
    assert_eq!(error.counters().deferred_capability_denial_count(), 0);
}

#[test]
fn config_section_resolution_remains_owned_by_subsystem() {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let (relational_section, counters) =
        facade.resolve_config_section(WorthQueryConfigSectionFamily::Relational);

    assert_eq!(
        relational_section.section(),
        WorthQueryConfigSectionFamily::Relational
    );
    assert_eq!(
        relational_section.owner(),
        crate::application::WorthQuerySubsystemOwner::Relational
    );
    assert!(relational_section.enabled());
    assert_eq!(counters.configuration_section_resolution_count(), 1);
}

#[test]
fn invalid_config_rejects_composed_capabilities_without_query_execution() {
    let error = WorthQueryConfig::runtime_backed_default()
        .with_query(WorthQueryQueryConfig::disabled())
        .with_signal(WorthQuerySignalConfig::enabled())
        .validate()
        .expect_err("signal without query execution should reject config");

    assert_eq!(
        error.failure_class(),
        ConfigurationAdmissionFailureClass::MissingRequiredSection
    );
    assert_eq!(error.section(), Some(WorthQueryConfigSectionFamily::Query));
}

#[test]
fn invalid_store_config_rejects_before_facade_construction() {
    let error = WorthQueryApplicationFacade::new(
        WorthQueryConfig::runtime_backed_default()
            .with_relational(
                WorthQueryRelationalConfig::disabled().with_historical_evaluation(false),
            )
            .with_store(crate::application::WorthQueryStoreConfig::enabled()),
    )
    .expect_err("store-backed config should reject before facade construction");

    assert_eq!(
        error.failure_class(),
        ConfigurationAdmissionFailureClass::ContradictorySectionPosture
    );
}

#[test]
fn historical_capability_can_be_disabled_typed_and_early() {
    let facade = WorthQueryApplicationFacade::new(
        WorthQueryConfig::runtime_backed_default().with_relational(
            WorthQueryRelationalConfig::enabled().with_historical_evaluation(false),
        ),
    )
    .expect("disabling historical alone should preserve a valid facade config");

    let error = facade
        .historical_query_capability()
        .expect_err("disabled historical section should deny historical capability admission");
    assert_eq!(
        error.failure_class(),
        WorthQueryFacadeFailureClass::InvalidComposedSupportPosture
    );
    assert_eq!(
        error.capability_family(),
        Some(WorthQueryCapabilityFamily::HistoricalEvaluation)
    );
    assert_eq!(error.counters().unsupported_composition_denial_count(), 1);
    assert_eq!(error.counters().deferred_capability_denial_count(), 0);
}
