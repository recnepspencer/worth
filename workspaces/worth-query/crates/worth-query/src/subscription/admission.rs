use crate::basis_lifecycle::ScopedSubscriptionDeclarationBasis;
use crate::evidence_identity::WorthQueryEvidenceIdentity;
#[cfg(test)]
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_authority::{
    project_query_subscription_evidence, QueryProjectionIdentity, QuerySubscriptionIdentityKind,
};

use super::admission_budget::QuerySubscriptionAdmissionBudget;
use super::admission_diagnostics::{
    QuerySubscriptionAdmissionDiagnosticOutcome, QuerySubscriptionAdmissionDiagnosticStage,
    QuerySubscriptionAdmissionDiagnostics,
};
use super::admission_error::{
    QuerySubscriptionAdmissionDenialKind, QuerySubscriptionAdmissionError,
};
use super::bridge_lowering::BridgeSubscriptionLoweringPlan;
use super::counters::QuerySubscriptionDeclarationCounters;
use super::diagnostic::{QuerySubscriptionDiagnosticEvidence, QuerySubscriptionDiagnosticStage};
use super::evidence_identities::admission_artifact_identity;
use super::future_selection::QuerySubscriptionFutureSelection;
use super::support::QuerySubscriptionSupportProfile;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionAdmissionArtifact {
    admission_identity: WorthQueryEvidenceIdentity,
    query_declaration_identity: WorthQueryEvidenceIdentity,
    bridge_declaration_identity: WorthQueryEvidenceIdentity,
    future_selection: QuerySubscriptionFutureSelection,
    basis_binding_identity: WorthQueryEvidenceIdentity,
    scoped_declaration_basis: ScopedSubscriptionDeclarationBasis,
    signal_strategy_identity: WorthQueryEvidenceIdentity,
    admission_budget: QuerySubscriptionAdmissionBudget,
    diagnostics: QuerySubscriptionAdmissionDiagnostics,
    support_profile: QuerySubscriptionSupportProfile,
    counters: QuerySubscriptionDeclarationCounters,
}

impl QuerySubscriptionAdmissionArtifact {
    pub fn admission_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        project_query_subscription_evidence(&self.admission_identity)
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.admission_identity
    }

    pub fn query_declaration_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        project_query_subscription_evidence(&self.query_declaration_identity)
    }

    pub fn query_declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.query_declaration_identity
    }

    pub fn bridge_declaration_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        project_query_subscription_evidence(&self.bridge_declaration_identity)
    }

    pub fn bridge_declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.bridge_declaration_identity
    }

    pub fn future_selection(&self) -> &QuerySubscriptionFutureSelection {
        &self.future_selection
    }

    pub fn basis_binding_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        project_query_subscription_evidence(&self.basis_binding_identity)
    }

    pub fn basis_binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_binding_identity
    }

    pub fn scoped_declaration_basis(&self) -> &ScopedSubscriptionDeclarationBasis {
        &self.scoped_declaration_basis
    }

    pub fn signal_strategy_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        project_query_subscription_evidence(&self.signal_strategy_identity)
    }

    pub fn signal_strategy_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.signal_strategy_identity
    }

    #[cfg(test)]
    pub(crate) fn recomputed_evidence_identity(
        &self,
        counters_identity: &WorthQueryEvidenceIdentity,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(admission_artifact_identity(
            self.query_declaration_identity(),
            self.bridge_declaration_identity(),
            self.basis_binding_identity(),
            self.signal_strategy_identity(),
            self.diagnostics().diagnostics_identity(),
            self.support_profile().profile_identity(),
            self.admission_budget().declaration_input_width_limit(),
            self.admission_budget().bridge_plan_width_limit(),
            self.admission_budget().basis_binding_width_limit(),
            self.admission_budget().signal_strategy_width_limit(),
            self.admission_budget().activation_input_width_limit(),
            counters_identity,
        ))
    }

    pub fn admission_budget(&self) -> &QuerySubscriptionAdmissionBudget {
        &self.admission_budget
    }

    pub fn diagnostics(&self) -> &QuerySubscriptionAdmissionDiagnostics {
        &self.diagnostics
    }

    pub fn support_profile(&self) -> &QuerySubscriptionSupportProfile {
        &self.support_profile
    }

    pub fn counters(&self) -> &QuerySubscriptionDeclarationCounters {
        &self.counters
    }
}

pub fn admit_query_subscription(
    lowering: BridgeSubscriptionLoweringPlan,
    admission_budget: QuerySubscriptionAdmissionBudget,
) -> Result<QuerySubscriptionAdmissionArtifact, QuerySubscriptionAdmissionError> {
    let mut counters = lowering.counters().clone();
    let source_identity = lowering.bridge_declaration_identity();

    if exceeds_admission_budget(&admission_budget) {
        counters.admission_denial_count = 1;
        counters.work_budget_denial_count = 1;
        return Err(admission_error(
            QuerySubscriptionAdmissionDenialKind::AdmissionBudgetExceeded,
            "subscription admission exceeds its explicit admission budget",
            QuerySubscriptionAdmissionDiagnosticStage::AdmissionBudget,
            source_identity,
            counters,
        ));
    }

    if admission_budget.durable_reload_requested() {
        counters.admission_denial_count = 1;
        counters.durable_overclaim_denial_count = 1;
        counters.declaration_time_checkpoint_denial_count = 1;
        return Err(admission_error(
            QuerySubscriptionAdmissionDenialKind::DurableReloadOverclaim,
            "durable subscription reload remains later-milestone debt",
            QuerySubscriptionAdmissionDiagnosticStage::DurableReloadOverclaim,
            source_identity,
            counters,
        ));
    }

    if admission_budget.active_lifecycle_allocation_requested() {
        counters.admission_denial_count = 1;
        counters.active_state_allocation_denial_count = 1;
        return Err(admission_error(
            QuerySubscriptionAdmissionDenialKind::ActiveLifecycleAllocationForbidden,
            "Milestone 9.1 admission may not allocate active subscription lifecycle state",
            QuerySubscriptionAdmissionDiagnosticStage::ActiveLifecycleAllocation,
            source_identity,
            counters,
        ));
    }

    counters.admission_count = 1;
    let diagnostics = QuerySubscriptionAdmissionDiagnostics::new(
        QuerySubscriptionAdmissionDiagnosticStage::RuntimeBackedAdmission,
        QuerySubscriptionAdmissionDiagnosticOutcome::Admitted,
        "runtime-backed query subscription declaration admitted for activation handoff",
        source_identity,
    ).expect("admission diagnostics use closed stages, literal messages, sealed sources and fixed counters");
    let support_profile =
        QuerySubscriptionSupportProfile::admitted(lowering.bridge_declaration_identity());
    let admission_identity = admission_artifact_identity(
        lowering.query_declaration_identity(),
        lowering.bridge_declaration_identity(),
        lowering.basis_request().evidence_identity(),
        lowering.signal_strategy_request().evidence_identity(),
        diagnostics.diagnostics_identity(),
        support_profile.profile_identity(),
        admission_budget.declaration_input_width_limit(),
        admission_budget.bridge_plan_width_limit(),
        admission_budget.basis_binding_width_limit(),
        admission_budget.signal_strategy_width_limit(),
        admission_budget.activation_input_width_limit(),
        &counters.evidence_identity(),
    );

    Ok(QuerySubscriptionAdmissionArtifact {
        admission_identity,
        query_declaration_identity: lowering.query_declaration_identity().clone(),
        bridge_declaration_identity: lowering.bridge_declaration_identity().clone(),
        future_selection: lowering.future_selection().clone(),
        basis_binding_identity: lowering.basis_request().evidence_identity().clone(),
        scoped_declaration_basis: lowering.scoped_declaration_basis().clone(),
        signal_strategy_identity: lowering
            .signal_strategy_request()
            .evidence_identity()
            .clone(),
        admission_budget,
        diagnostics,
        support_profile,
        counters,
    })
}

fn admission_error(
    denial_kind: QuerySubscriptionAdmissionDenialKind,
    message: &'static str,
    stage: QuerySubscriptionAdmissionDiagnosticStage,
    source_identity: &WorthQueryEvidenceIdentity,
    counters: QuerySubscriptionDeclarationCounters,
) -> QuerySubscriptionAdmissionError {
    let diagnostics = QuerySubscriptionAdmissionDiagnostics::new(
        stage,
        QuerySubscriptionAdmissionDiagnosticOutcome::Denied,
        message,
        source_identity,
    ).expect("admission diagnostics use closed stages, literal messages, sealed sources and fixed counters");
    let pipeline_diagnostic = QuerySubscriptionDiagnosticEvidence::denied(
        admission_pipeline_stage(stage),
        message,
        source_identity,
        &counters.evidence_identity(),
    ).expect("admission diagnostics use closed stages, literal messages, sealed sources and fixed counters");
    let support_profile = QuerySubscriptionSupportProfile::denied(source_identity);
    QuerySubscriptionAdmissionError::new(
        denial_kind,
        message,
        diagnostics,
        pipeline_diagnostic,
        support_profile,
        counters,
    )
}

fn admission_pipeline_stage(
    stage: QuerySubscriptionAdmissionDiagnosticStage,
) -> QuerySubscriptionDiagnosticStage {
    match stage {
        QuerySubscriptionAdmissionDiagnosticStage::RuntimeBackedAdmission => {
            QuerySubscriptionDiagnosticStage::RuntimeBackedAdmission
        }
        QuerySubscriptionAdmissionDiagnosticStage::AdmissionBudget => {
            QuerySubscriptionDiagnosticStage::AdmissionBudget
        }
        QuerySubscriptionAdmissionDiagnosticStage::DurableReloadOverclaim => {
            QuerySubscriptionDiagnosticStage::DurableReloadOverclaim
        }
        QuerySubscriptionAdmissionDiagnosticStage::ActiveLifecycleAllocation => {
            QuerySubscriptionDiagnosticStage::ActiveLifecycleAllocation
        }
        QuerySubscriptionAdmissionDiagnosticStage::ActivationReadiness => {
            QuerySubscriptionDiagnosticStage::ActivationReadiness
        }
    }
}

fn exceeds_admission_budget(budget: &QuerySubscriptionAdmissionBudget) -> bool {
    budget.declaration_input_width_limit() < 1
        || budget.bridge_plan_width_limit() < 1
        || budget.basis_binding_width_limit() < 1
        || budget.signal_strategy_width_limit() < 1
        || budget.activation_input_width_limit() < 1
}
