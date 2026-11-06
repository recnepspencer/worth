use crate::authorized_projection::AuthorizedProjectionArtifact;
use crate::canonicalization::CanonicalResultShapeArtifact;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::query_context::QueryContextExecutionArtifact;
use crate::runtime::{
    WorthQueryDerivedArtifactBinding, WorthQueryLiveArtifactBinding, WorthQueryLiveReadResult,
    WorthQueryReadResult, WorthQueryWriteReceipt,
};

use super::contracts::MaterializedProjectionContract;
use super::declaration::{ProjectionConsumptionDeclaration, ProjectionConsumptionDeclarationError};
use super::eligibility::{
    evaluate_projection_consumption_eligibility, DeferredProjectionConsumption,
    DeniedProjectionConsumption, ProjectionConsumptionEligibility, ProjectionConsumptionWarnings,
    SourceMismatchedProjectionConsumption,
};
#[cfg(test)]
use super::envelope::SelfDescribingProjectionConsumptionEnvelope;
use super::extraction::ProjectionFactExtractionError;
use super::facts::ProjectMaterializedFacts;
use super::receipt::ProjectionConsumptionReceipt;
use super::source::ProjectionConsumptionSource;
use super::support::{discover_projection_consumption_support, ProjectionConsumptionSupportReport};
use super::ConsumedProjectionFactSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectionFactConsumptionPathError {
    Declaration(ProjectionConsumptionDeclarationError),
    Extraction(ProjectionFactExtractionError),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompletedProjectionFactConsumption {
    declaration: ProjectionConsumptionDeclaration,
    contract: MaterializedProjectionContract,
    facts: ConsumedProjectionFactSet,
    receipt: ProjectionConsumptionReceipt,
}

impl CompletedProjectionFactConsumption {
    pub(super) fn certification(
        declaration: ProjectionConsumptionDeclaration,
        contract: MaterializedProjectionContract,
        facts: ConsumedProjectionFactSet,
        receipt: ProjectionConsumptionReceipt,
    ) -> Self {
        Self {
            declaration,
            contract,
            facts,
            receipt,
        }
    }

    pub fn declaration(&self) -> &ProjectionConsumptionDeclaration {
        &self.declaration
    }

    pub fn contract(&self) -> &MaterializedProjectionContract {
        &self.contract
    }

    pub fn facts(&self) -> &ConsumedProjectionFactSet {
        &self.facts
    }

    pub fn receipt(&self) -> &ProjectionConsumptionReceipt {
        &self.receipt
    }

    pub fn source_family(&self) -> super::source::ProjectionSourceFamily {
        self.receipt.source_family()
    }

    #[cfg(test)]
    pub fn source_identity(&self) -> &str {
        self.receipt.source_identity()
    }

    #[cfg(test)]
    pub fn support_posture(&self) -> &super::contracts::ProjectionContractSupportPosture {
        self.receipt.support_posture()
    }

    #[cfg(test)]
    pub fn materialized_fact_posture(&self) -> Option<&super::ProjectionMaterializedFactPosture> {
        self.receipt.materialized_fact_posture()
    }

    #[cfg(test)]
    pub fn warning_kinds(&self) -> &[super::eligibility::ProjectionConsumptionWarningKind] {
        self.receipt.warning_kinds()
    }

    #[cfg(test)]
    pub fn admitted_fact_family_count(&self) -> usize {
        self.receipt.admitted_fact_family_count()
    }

    #[cfg(test)]
    pub fn extracted_fact_count(&self) -> usize {
        self.receipt.extracted_fact_count()
    }

    #[cfg(test)]
    pub fn authority_reopen_count(&self) -> usize {
        self.receipt.authority_reopen_count()
    }

    #[cfg(test)]
    pub fn deferred_neighbors(
        &self,
    ) -> &[super::receipt_transitions::ProjectionConsumptionDeferredNeighborFamily] {
        self.receipt.deferred_neighbors()
    }

    #[cfg(test)]
    pub fn transition_rules(
        &self,
    ) -> super::receipt_transitions::ProjectionConsumptionTransitionRules {
        self.receipt.transition_rules()
    }

    #[cfg(test)]
    pub fn projection_consumption_envelope(&self) -> SelfDescribingProjectionConsumptionEnvelope {
        self.receipt.projection_consumption_envelope()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProjectionFactConsumptionAttempt {
    Admitted(CompletedProjectionFactConsumption),
    AdmittedWithWarnings(
        CompletedProjectionFactConsumption,
        ProjectionConsumptionWarnings,
    ),
    Denied(DeniedProjectionConsumption),
    Deferred(DeferredProjectionConsumption),
    SourceMismatch(SourceMismatchedProjectionConsumption),
}

impl ProjectionFactConsumptionAttempt {
    #[cfg(test)]
    pub fn completed(&self) -> Option<&CompletedProjectionFactConsumption> {
        match self {
            Self::Admitted(completed) | Self::AdmittedWithWarnings(completed, _) => Some(completed),
            Self::Denied(_) | Self::Deferred(_) | Self::SourceMismatch(_) => None,
        }
    }

    #[cfg(test)]
    pub fn warnings(&self) -> Option<&ProjectionConsumptionWarnings> {
        match self {
            Self::AdmittedWithWarnings(_, warnings) => Some(warnings),
            Self::Admitted(_) | Self::Denied(_) | Self::Deferred(_) | Self::SourceMismatch(_) => {
                None
            }
        }
    }

    #[cfg(test)]
    pub fn denied(&self) -> Option<&DeniedProjectionConsumption> {
        match self {
            Self::Denied(denied) => Some(denied),
            Self::Admitted(_)
            | Self::AdmittedWithWarnings(_, _)
            | Self::Deferred(_)
            | Self::SourceMismatch(_) => None,
        }
    }

    #[cfg(test)]
    pub fn deferred(&self) -> Option<&DeferredProjectionConsumption> {
        match self {
            Self::Deferred(deferred) => Some(deferred),
            Self::Admitted(_)
            | Self::AdmittedWithWarnings(_, _)
            | Self::Denied(_)
            | Self::SourceMismatch(_) => None,
        }
    }

    #[cfg(test)]
    pub fn source_mismatch(&self) -> Option<&SourceMismatchedProjectionConsumption> {
        match self {
            Self::SourceMismatch(mismatch) => Some(mismatch),
            Self::Admitted(_)
            | Self::AdmittedWithWarnings(_, _)
            | Self::Denied(_)
            | Self::Deferred(_) => None,
        }
    }
}

impl WorthQueryReadResult {
    pub(crate) fn consume_projection_facts(
        &self,
        result_shape: &CanonicalResultShapeArtifact,
        authorized_projection: &AuthorizedProjectionArtifact,
        requested: ProjectMaterializedFacts,
    ) -> Result<ProjectionFactConsumptionAttempt, ProjectionFactConsumptionPathError> {
        let declaration = self
            .receipt()
            .declare_projection_fact_consumption(result_shape, authorized_projection, requested)
            .map_err(ProjectionFactConsumptionPathError::Declaration)?;
        consume_attempt_from_declaration(declaration, |contract| {
            contract.extract_from_read_result(self)
        })
    }
}

impl WorthQueryLiveReadResult {
    pub(crate) fn consume_projection_facts_with_binding(
        &self,
        binding: super::declaration::ProjectionConsumptionBindingContext,
        requested: ProjectMaterializedFacts,
    ) -> Result<ProjectionFactConsumptionAttempt, ProjectionFactConsumptionPathError> {
        let declaration = super::declare_projection_consumption(
            ProjectionConsumptionSource::from_live_read_receipt(self.receipt()),
            binding,
            requested,
        )
        .map_err(ProjectionFactConsumptionPathError::Declaration)?;
        consume_attempt_from_declaration(declaration, |contract| {
            contract.extract_from_live_read_result(self)
        })
    }
}

impl crate::runtime::WorthQueryReadReceipt {
    pub fn discover_projection_fact_consumption_support(
        &self,
        result_shape: &CanonicalResultShapeArtifact,
    ) -> Result<ProjectionConsumptionSupportReport, IdentityDerivationDenial> {
        Ok(discover_projection_consumption_support(
            &ProjectionConsumptionSource::from_read_receipt(self, result_shape),
        )?)
    }
}

impl WorthQueryWriteReceipt {
    pub(crate) fn consume_projection_facts(
        &self,
        result_shape_digest: &str,
        authorized_projection: &AuthorizedProjectionArtifact,
        requested: ProjectMaterializedFacts,
    ) -> Result<ProjectionFactConsumptionAttempt, ProjectionFactConsumptionPathError> {
        let declaration = self
            .declare_projection_fact_consumption(
                result_shape_digest,
                authorized_projection,
                requested,
            )
            .map_err(ProjectionFactConsumptionPathError::Declaration)?;
        consume_attempt_from_declaration(declaration, |contract| {
            contract.extract_from_write_receipt(self)
        })
    }

    pub fn discover_projection_fact_consumption_support(
        &self,
    ) -> Result<ProjectionConsumptionSupportReport, IdentityDerivationDenial> {
        Ok(discover_projection_consumption_support(
            &ProjectionConsumptionSource::from_write_receipt(self),
        )?)
    }
}

impl QueryContextExecutionArtifact {
    pub(crate) fn consume_projection_facts(
        &self,
        authorized_projection: &AuthorizedProjectionArtifact,
        requested: ProjectMaterializedFacts,
    ) -> Result<ProjectionFactConsumptionAttempt, ProjectionFactConsumptionPathError> {
        let declaration = self
            .declare_projection_fact_consumption(authorized_projection, requested)
            .map_err(ProjectionFactConsumptionPathError::Declaration)?;
        consume_attempt_from_declaration(declaration, |contract| {
            contract.extract_from_query_context_execution(self)
        })
    }

    pub fn discover_projection_fact_consumption_support(
        &self,
    ) -> Result<ProjectionConsumptionSupportReport, IdentityDerivationDenial> {
        Ok(discover_projection_consumption_support(
            &ProjectionConsumptionSource::from_query_context_execution(self),
        )?)
    }
}

impl WorthQueryDerivedArtifactBinding {
    pub(crate) fn consume_projection_facts(
        &self,
        result_shape: &CanonicalResultShapeArtifact,
        authorized_projection: &AuthorizedProjectionArtifact,
        requested: ProjectMaterializedFacts,
    ) -> Result<ProjectionFactConsumptionAttempt, ProjectionFactConsumptionPathError> {
        let declaration = self
            .declare_projection_fact_consumption(result_shape, authorized_projection, requested)
            .map_err(ProjectionFactConsumptionPathError::Declaration)?;
        consume_attempt_from_declaration(declaration, |contract| {
            contract.extract_from_retained_derived_artifact_binding(self)
        })
    }

    pub fn discover_projection_fact_consumption_support(
        &self,
    ) -> Result<ProjectionConsumptionSupportReport, IdentityDerivationDenial> {
        Ok(discover_projection_consumption_support(
            &ProjectionConsumptionSource::from_retained_derived_artifact_binding(self)?,
        )?)
    }
}

impl WorthQueryLiveArtifactBinding {
    pub(crate) fn consume_projection_facts(
        &self,
        result_shape_identity: &crate::evidence_identity::WorthQueryEvidenceIdentity,
        authorized_projection: &AuthorizedProjectionArtifact,
        requested: ProjectMaterializedFacts,
    ) -> Result<ProjectionFactConsumptionAttempt, ProjectionFactConsumptionPathError> {
        let declaration = self
            .declare_projection_fact_consumption(
                result_shape_identity,
                authorized_projection,
                requested,
            )
            .map_err(ProjectionFactConsumptionPathError::Declaration)?;
        consume_attempt_from_declaration(declaration, |contract| {
            contract.extract_from_live_artifact_binding(self)
        })
    }

    pub fn discover_projection_fact_consumption_support(
        &self,
    ) -> Result<ProjectionConsumptionSupportReport, IdentityDerivationDenial> {
        Ok(discover_projection_consumption_support(
            &ProjectionConsumptionSource::from_live_artifact_binding(self)?,
        )?)
    }
}

fn consume_attempt_from_declaration(
    declaration: ProjectionConsumptionDeclaration,
    extract: impl FnOnce(
        &MaterializedProjectionContract,
    ) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError>,
) -> Result<ProjectionFactConsumptionAttempt, ProjectionFactConsumptionPathError> {
    match evaluate_projection_consumption_eligibility(&declaration) {
        ProjectionConsumptionEligibility::Admitted(admitted) => {
            let contract = admitted.bind_contract();
            let facts =
                extract(&contract).map_err(ProjectionFactConsumptionPathError::Extraction)?;
            let receipt = facts.issue_receipt();
            Ok(ProjectionFactConsumptionAttempt::Admitted(
                CompletedProjectionFactConsumption {
                    declaration,
                    contract,
                    facts,
                    receipt,
                },
            ))
        }
        ProjectionConsumptionEligibility::AdmittedWithWarnings(admitted, warnings) => {
            let contract = admitted.bind_contract();
            let facts =
                extract(&contract).map_err(ProjectionFactConsumptionPathError::Extraction)?;
            let receipt = facts.issue_receipt();
            Ok(ProjectionFactConsumptionAttempt::AdmittedWithWarnings(
                CompletedProjectionFactConsumption {
                    declaration,
                    contract,
                    facts,
                    receipt,
                },
                warnings,
            ))
        }
        ProjectionConsumptionEligibility::Denied(denied) => {
            Ok(ProjectionFactConsumptionAttempt::Denied(denied))
        }
        ProjectionConsumptionEligibility::Deferred(deferred) => {
            Ok(ProjectionFactConsumptionAttempt::Deferred(deferred))
        }
        ProjectionConsumptionEligibility::SourceMismatch(mismatch) => {
            Ok(ProjectionFactConsumptionAttempt::SourceMismatch(mismatch))
        }
    }
}
