#[cfg(test)]
use super::errors::{CapabilityAdmissionError, WorthQueryFacadeCounters};
#[cfg(test)]
use super::resolution::{
    deny_capability, CapabilityAdmissionDecision, WorthQueryCapabilityResolution,
};
#[cfg(test)]
use super::witnesses::{
    HistoricalEvaluationCapability, IdentityEvolutionCapability, LiveQueryCapability,
    PreviewSessionCapability, QueryCompositionCapability, QueryContextCapability,
    QueryReadCapability, WorkflowOrchestrationCapability,
};
use crate::application::config::{
    ConfigurationAdmissionError, ValidatedWorthQueryConfig, WorthQueryConfig,
};
#[cfg(test)]
use crate::application::config::{
    WorthQueryConfigSectionFamily, WorthQueryConfigSectionResolution,
};
#[cfg(test)]
use crate::application::support::{WorthQueryCapabilityFamily, WorthQueryCapabilityStatus};
use crate::application::support::{
    WorthQueryCapabilityRegistry, WorthQuerySupportMatrix, WorthQuerySupportReport,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity::hash_parts;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationFacade {
    config: ValidatedWorthQueryConfig,
    support_matrix: WorthQuerySupportMatrix,
    facade_digest: String,
}

impl WorthQueryApplicationFacade {
    pub fn new(config: WorthQueryConfig) -> Result<Self, ConfigurationAdmissionError> {
        let config = config.validate()?;
        let registry = WorthQueryCapabilityRegistry::from_validated_config(&config);
        let support_matrix = WorthQuerySupportMatrix::new(registry);
        let facade_digest = hash_parts(&[
            format!("config:{}", config.validated_digest()),
            format!(
                "registry:{}",
                support_matrix.capability_registry().registry_digest()
            ),
            format!("support:{}", support_matrix.support_matrix_digest()),
        ]);
        Ok(Self {
            config,
            support_matrix,
            facade_digest,
        })
    }

    pub fn runtime_backed_default() -> Self {
        Self::new(WorthQueryConfig::runtime_backed_default())
            .expect("runtime-backed default config should be valid")
    }

    pub fn validated_config(&self) -> &ValidatedWorthQueryConfig {
        &self.config
    }

    pub fn support_matrix(&self) -> WorthQuerySupportMatrix {
        self.support_matrix.clone()
    }

    pub fn support_report(&self) -> Result<WorthQuerySupportReport, IdentityDerivationDenial> {
        Ok(WorthQuerySupportReport::from_validated_config_and_matrix(
            &self.config,
            self.support_matrix.clone(),
        )?)
    }

    #[cfg(test)]
    pub fn resolve_config_section(
        &self,
        section: WorthQueryConfigSectionFamily,
    ) -> (WorthQueryConfigSectionResolution, WorthQueryFacadeCounters) {
        (
            self.config.resolve_section(section),
            WorthQueryFacadeCounters::config_resolution(),
        )
    }

    #[cfg(test)]
    pub fn query_read_capability(
        &self,
    ) -> Result<WorthQueryCapabilityResolution<QueryReadCapability>, CapabilityAdmissionError> {
        let admission = self.query_read_admission_decision()?;
        Ok(WorthQueryCapabilityResolution::new(
            QueryReadCapability::new(self.facade_digest.clone()),
            admission,
        ))
    }

    #[cfg(test)]
    pub(crate) fn query_read_admission_decision(
        &self,
    ) -> Result<CapabilityAdmissionDecision, CapabilityAdmissionError> {
        self.require_capability(WorthQueryCapabilityFamily::QueryRead)
    }

    #[cfg(test)]
    pub fn query_composition_capability(
        &self,
    ) -> Result<WorthQueryCapabilityResolution<QueryCompositionCapability>, CapabilityAdmissionError>
    {
        let admission = self.query_composition_admission_decision()?;
        Ok(WorthQueryCapabilityResolution::new(
            QueryCompositionCapability::new(self.facade_digest.clone()),
            admission,
        ))
    }

    #[cfg(test)]
    pub(crate) fn query_composition_admission_decision(
        &self,
    ) -> Result<CapabilityAdmissionDecision, CapabilityAdmissionError> {
        self.require_capability(WorthQueryCapabilityFamily::QueryComposition)
    }

    #[cfg(test)]
    pub fn live_query_capability(
        &self,
    ) -> Result<WorthQueryCapabilityResolution<LiveQueryCapability>, CapabilityAdmissionError> {
        let admission = self.live_query_admission_decision()?;
        Ok(WorthQueryCapabilityResolution::new(
            LiveQueryCapability::new(self.facade_digest.clone()),
            admission,
        ))
    }

    #[cfg(test)]
    pub(crate) fn live_query_admission_decision(
        &self,
    ) -> Result<CapabilityAdmissionDecision, CapabilityAdmissionError> {
        self.require_capability(WorthQueryCapabilityFamily::LiveQuery)
    }

    #[cfg(test)]
    pub fn preview_query_capability(
        &self,
    ) -> Result<WorthQueryCapabilityResolution<PreviewSessionCapability>, CapabilityAdmissionError>
    {
        let admission = self.preview_query_admission_decision()?;
        Ok(WorthQueryCapabilityResolution::new(
            PreviewSessionCapability::new(self.facade_digest.clone()),
            admission,
        ))
    }

    #[cfg(test)]
    pub(crate) fn preview_query_admission_decision(
        &self,
    ) -> Result<CapabilityAdmissionDecision, CapabilityAdmissionError> {
        self.require_capability(WorthQueryCapabilityFamily::PreviewSession)
    }

    #[cfg(test)]
    pub fn workflow_query_capability(
        &self,
    ) -> Result<
        WorthQueryCapabilityResolution<WorkflowOrchestrationCapability>,
        CapabilityAdmissionError,
    > {
        let admission = self.workflow_query_admission_decision()?;
        Ok(WorthQueryCapabilityResolution::new(
            WorkflowOrchestrationCapability::new(self.facade_digest.clone()),
            admission,
        ))
    }

    #[cfg(test)]
    pub(crate) fn workflow_query_admission_decision(
        &self,
    ) -> Result<CapabilityAdmissionDecision, CapabilityAdmissionError> {
        self.require_capability(WorthQueryCapabilityFamily::WorkflowOrchestration)
    }

    #[cfg(test)]
    pub fn historical_query_capability(
        &self,
    ) -> Result<
        WorthQueryCapabilityResolution<HistoricalEvaluationCapability>,
        CapabilityAdmissionError,
    > {
        let admission = self.historical_query_admission_decision()?;
        Ok(WorthQueryCapabilityResolution::new(
            HistoricalEvaluationCapability::new(self.facade_digest.clone()),
            admission,
        ))
    }

    #[cfg(test)]
    pub(crate) fn historical_query_admission_decision(
        &self,
    ) -> Result<CapabilityAdmissionDecision, CapabilityAdmissionError> {
        self.require_capability(WorthQueryCapabilityFamily::HistoricalEvaluation)
    }

    #[cfg(test)]
    pub fn query_context_capability(
        &self,
    ) -> Result<WorthQueryCapabilityResolution<QueryContextCapability>, CapabilityAdmissionError>
    {
        let admission = self.query_context_admission_decision()?;
        Ok(WorthQueryCapabilityResolution::new(
            QueryContextCapability::new(self.facade_digest.clone()),
            admission,
        ))
    }

    #[cfg(test)]
    pub(crate) fn query_context_admission_decision(
        &self,
    ) -> Result<CapabilityAdmissionDecision, CapabilityAdmissionError> {
        self.require_capability(WorthQueryCapabilityFamily::QueryContext)
    }

    #[cfg(test)]
    pub fn identity_evolution_capability(
        &self,
    ) -> Result<WorthQueryCapabilityResolution<IdentityEvolutionCapability>, CapabilityAdmissionError>
    {
        let admission = self.identity_evolution_admission_decision()?;
        Ok(WorthQueryCapabilityResolution::new(
            IdentityEvolutionCapability::new(self.facade_digest.clone()),
            admission,
        ))
    }

    #[cfg(test)]
    pub(crate) fn identity_evolution_admission_decision(
        &self,
    ) -> Result<CapabilityAdmissionDecision, CapabilityAdmissionError> {
        self.require_capability(WorthQueryCapabilityFamily::IdentityEvolution)
    }

    #[cfg(test)]
    pub fn durable_artifact_capability(
        &self,
    ) -> Result<CapabilityAdmissionDecision, CapabilityAdmissionError> {
        self.require_capability(WorthQueryCapabilityFamily::DurableArtifacts)
    }

    #[cfg(test)]
    fn require_capability(
        &self,
        family: WorthQueryCapabilityFamily,
    ) -> Result<CapabilityAdmissionDecision, CapabilityAdmissionError> {
        let descriptor = self
            .support_matrix
            .descriptor(family)
            .expect("capability registry should contain every known capability family")
            .clone();
        match descriptor.status() {
            WorthQueryCapabilityStatus::Admitted => Ok(CapabilityAdmissionDecision::admitted(
                descriptor,
                self.config.validated_digest(),
            )),
            WorthQueryCapabilityStatus::DeferredDebt | WorthQueryCapabilityStatus::Unsupported => {
                Err(deny_capability(&descriptor, &self.config))
            }
        }
    }
}
