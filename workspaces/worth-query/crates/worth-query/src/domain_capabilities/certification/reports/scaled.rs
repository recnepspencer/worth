use crate::domain_capabilities::certification::reports::fixtures::{
    admitted_basis_observation_plan, admitted_projection_consumption_plan, admitted_ready,
    intent_declaration, lower_runtime_envelope, plan_support_requested,
    projection_contract_request, store_backed_replay_gap_request, success,
};
use crate::domain_capabilities::identity::{
    compose_scaled_category_digest, compose_scaled_contribution_digest,
    compose_scaled_support_digest, compose_scaled_trace_digest,
};
use crate::domain_capabilities::{
    materialize_intent_admission_support_traceability_report, WorthQueryDomainCapabilityCategory,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData;
use crate::intent_admission::WorthQueryIntentAdmissionCoveredEntrypoint;

pub(crate) struct WorthQueryDomainCapabilityScaledEvidence {
    contribution_width: usize,
    trace_width: usize,
    category_width: usize,
    support_width: usize,
    contribution_digest: String,
    trace_digest: String,
    category_digest: String,
    support_digest: String,
}

impl WorthQueryDomainCapabilityScaledEvidence {
    pub(crate) fn contribution_width(&self) -> usize {
        self.contribution_width
    }

    pub(crate) fn trace_width(&self) -> usize {
        self.trace_width
    }

    pub(crate) fn category_width(&self) -> usize {
        self.category_width
    }

    pub(crate) fn support_width(&self) -> usize {
        self.support_width
    }

    pub(crate) fn contribution_digest(&self) -> &str {
        &self.contribution_digest
    }

    pub(crate) fn trace_digest(&self) -> &str {
        &self.trace_digest
    }

    pub(crate) fn category_digest(&self) -> &str {
        &self.category_digest
    }

    pub(crate) fn support_digest(&self) -> &str {
        &self.support_digest
    }
}

pub(crate) fn worth_query_domain_capability_scaled_evidence_in(
    domain: &crate::domain_capabilities::WorthQueryInstalledDomainContributionSurface,
) -> Result<[WorthQueryDomainCapabilityScaledEvidence; 3], IdentityDerivationDenial> {
    Ok([
        scaled_evidence(1, domain)?,
        scaled_evidence(2, domain)?,
        scaled_evidence(3, domain)?,
    ])
}

fn scaled_evidence(
    scale: usize,
    domain: &crate::domain_capabilities::WorthQueryInstalledDomainContributionSurface,
) -> Result<WorthQueryDomainCapabilityScaledEvidence, IdentityDerivationDenial> {
    let declaration = intent_declaration(&format!("domain-capability-scale-{scale}"));
    let admitted_plan = admitted_basis_observation_plan()?;
    let projection_plan = admitted_projection_consumption_plan()?;
    let lower_runtime = lower_runtime_envelope(&format!("domain-capability-scale-{scale}"))?;
    let categories = category_set_for_scale(scale);
    let admitted_plan_target = domain
        .admitted_plan_target(&admitted_plan)
        .expect("installed contribution authority must remain current");

    let mut contribution_digests = Vec::new();
    let mut trace_digests = Vec::new();
    let mut support_digests = Vec::new();
    let mut support_width = 0usize;
    let mut trace_width = 0usize;

    if categories.contains(&WorthQueryDomainCapabilityCategory::Admission) {
        let admission = domain
            .for_admitted_intent_plan(&admitted_plan)
            .expect("installed contribution authority must remain current")
            .advises(format!("admission.scale_{scale}"))
            .because("scaled admission evidence should remain canonical")
            .materialize()
            .expect("scaled admission should materialize");
        contribution_digests.push(admission_digest(&admission));
    }

    if categories.contains(&WorthQueryDomainCapabilityCategory::SupportTraceability) {
        let support = domain
            .for_intent(&declaration)
            .expect("installed contribution authority must remain current")
            .supports_traceability(format!("traceability.scale_{scale}"))
            .because("scaled support evidence should remain declaration scoped")
            .materialize()
            .expect("scaled support should materialize");
        contribution_digests.push(support.materialization_digest().to_string());

        for _ in 0..scale {
            let report = success(materialize_intent_admission_support_traceability_report(
                admitted_ready(plan_support_requested(admitted_plan_target.clone())?)?,
            )?);
            support_width += report.rows().len();
            support_digests.push(report.decision_support_traceability_digest().to_string());
        }
    }

    if categories.contains(&WorthQueryDomainCapabilityCategory::WorkflowPreview) {
        let workflow = domain
            .for_intent(&declaration)
            .expect("installed contribution authority must remain current")
            .plans_preview_mutation(
                format!("workflow.scale_{scale}"),
                crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(format!(
                    "preview-session:scale-{scale}"
                )),
            )
            .because("scaled workflow evidence should remain canonical")
            .materialize()
            .expect("scaled workflow should materialize");
        contribution_digests.push(
            workflow
                .workflow_declaration()
                .report()
                .declaration_digest()
                .to_string(),
        );
    }

    if categories.contains(&WorthQueryDomainCapabilityCategory::ContinuityLineage) {
        let continuity = domain
            .for_admitted_intent_plan(&admitted_plan)
            .expect("installed contribution authority must remain current")
            .preserves_continuity(
                format!("continuity.scale_{scale}"),
                format!("edge:{scale}:before"),
                format!("edge:{scale}:after"),
            )
            .because("scaled continuity evidence should remain canonical")
            .materialize()
            .expect("scaled continuity should materialize");
        contribution_digests.push(
            continuity
                .continuity_resolution_digest()
                .as_str()
                .to_string(),
        );
    }

    if categories.contains(&WorthQueryDomainCapabilityCategory::ConsequenceAftermath) {
        let aftermath = domain
            .for_admitted_intent_plan(&projection_plan)
            .expect("installed contribution authority must remain current")
            .consumes_projection_contract(
                format!("aftermath.scale_{scale}"),
                projection_contract_request()?,
            )
            .because("scaled aftermath evidence should remain canonical")
            .materialize()
            .expect("scaled aftermath should materialize");
        contribution_digests.push(aftermath.contract_digest().to_string());
    }

    if categories.contains(&WorthQueryDomainCapabilityCategory::ExplanationInspection) {
        let explanation = domain
            .for_lower_runtime_boundary_envelope(&lower_runtime)
            .expect("installed contribution authority must remain current")
            .explains_store_backed_replay_gap(
                format!("explanation.scale_{scale}"),
                store_backed_replay_gap_request()?,
            )
            .because("scaled explanation evidence should remain canonical")
            .materialize_artifact()
            .expect("scaled explanation should materialize");
        contribution_digests.push(explanation.artifact_for_reporting().to_string());
    }

    if categories.contains(&WorthQueryDomainCapabilityCategory::InvariantCapability) {
        let invariant = domain
            .for_intent(&declaration)
            .expect("installed contribution authority must remain current")
            .register_invariant_catalog(
                format!("invariant.scale_{scale}"),
                worth_relational::facade::runtime::InvariantCatalog::default(),
            )
            .because("scaled invariant evidence should remain canonical")
            .materialize()
            .expect("scaled invariant should materialize");
        contribution_digests.push(invariant.materialization_digest().to_string());
    }

    for index in 0..scale {
        let review = WorthQueryRuntimeIntentAdmissionReviewData::from_request(
            crate::intent_admission::WorthQueryRawIntentAdmissionRequest::deferred_neighbor(
                WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteInspectionNeighborDeferred,
                intent_declaration(&format!("trace.scale.{scale}.{index}")),
            )
            .expect("scaled deferred-neighbor request should build"),
        )?;
        let trace = review
            .decision_trace_envelope()
            .expect("scaled review should preserve decision trace")
            .clone();
        trace_width += trace.rows().len();
        trace_digests.push(trace.trace_digest().to_string());
    }

    Ok(WorthQueryDomainCapabilityScaledEvidence {
        contribution_width: contribution_digests.len(),
        trace_width,
        category_width: categories.len(),
        support_width,
        contribution_digest: compose_scaled_contribution_digest(&contribution_digests)?,
        trace_digest: compose_scaled_trace_digest(&trace_digests)?,
        category_digest: compose_scaled_category_digest(&categories)?,
        support_digest: compose_scaled_support_digest(&support_digests)?,
    })
}

fn category_set_for_scale(scale: usize) -> Vec<WorthQueryDomainCapabilityCategory> {
    match scale {
        1 => vec![
            WorthQueryDomainCapabilityCategory::Admission,
            WorthQueryDomainCapabilityCategory::SupportTraceability,
            WorthQueryDomainCapabilityCategory::WorkflowPreview,
        ],
        2 => vec![
            WorthQueryDomainCapabilityCategory::Admission,
            WorthQueryDomainCapabilityCategory::SupportTraceability,
            WorthQueryDomainCapabilityCategory::WorkflowPreview,
            WorthQueryDomainCapabilityCategory::ContinuityLineage,
            WorthQueryDomainCapabilityCategory::ConsequenceAftermath,
        ],
        _ => vec![
            WorthQueryDomainCapabilityCategory::Admission,
            WorthQueryDomainCapabilityCategory::SupportTraceability,
            WorthQueryDomainCapabilityCategory::WorkflowPreview,
            WorthQueryDomainCapabilityCategory::ContinuityLineage,
            WorthQueryDomainCapabilityCategory::ConsequenceAftermath,
            WorthQueryDomainCapabilityCategory::ExplanationInspection,
            WorthQueryDomainCapabilityCategory::InvariantCapability,
        ],
    }
}

fn admission_digest(decision: &crate::runtime::WorthQueryIntentAdmissionDecision) -> String {
    match decision {
        crate::runtime::WorthQueryIntentAdmissionDecision::Admitted(plan) => {
            plan.decision_digest().to_string()
        }
        crate::runtime::WorthQueryIntentAdmissionDecision::Advisory(advisory) => {
            advisory.decision_digest().to_string()
        }
        crate::runtime::WorthQueryIntentAdmissionDecision::Violation(violation) => {
            violation.decision_digest().to_string()
        }
    }
}
