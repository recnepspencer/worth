use super::InvariantCheckResult;
use serde::{Deserialize, Serialize};

use crate::contracts::validation::data::execution::{
    InvariantExecutionPoint, InvariantFailureEffect, InvariantReportedRule, InvariantVerdict,
    InvariantWitnessKey,
};
use crate::contracts::validation::data::results::InvariantViolationFields;
use crate::validation::custom_rule::CustomInvariantProvenance;

use super::{InvariantExecutionResult, InvariantFailure, InvariantProofBoundarySummary};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantProofBoundaryArtifact {
    scope_class: crate::validation::engine::result::metadata::InvariantPlanScopeClass,
    widened_causes: Vec<crate::validation::engine::result::metadata::InvariantScopeWideningCause>,
    packet_count: usize,
    touched_partition_count: usize,
}

impl InvariantProofBoundaryArtifact {
    fn from_summary(summary: &InvariantProofBoundarySummary) -> Self {
        Self {
            scope_class: summary.scope_class(),
            widened_causes: summary.widened_causes().to_vec(),
            packet_count: summary.packet_count(),
            touched_partition_count: summary.touched_partition_count(),
        }
    }

    pub fn scope_class(
        &self,
    ) -> crate::validation::engine::result::metadata::InvariantPlanScopeClass {
        self.scope_class
    }

    pub fn widened_causes(
        &self,
    ) -> &[crate::validation::engine::result::metadata::InvariantScopeWideningCause] {
        &self.widened_causes
    }

    pub fn packet_count(&self) -> usize {
        self.packet_count
    }

    pub fn touched_partition_count(&self) -> usize {
        self.touched_partition_count
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantFailureArtifact {
    execution_point: InvariantExecutionPoint,
    failure_effect: InvariantFailureEffect,
    witness: InvariantWitnessKey,
    proof_boundary: Option<InvariantProofBoundaryArtifact>,
    violation: InvariantViolationFields,
    custom_provenance: Option<CustomInvariantProvenance>,
}

impl InvariantFailureArtifact {
    fn new(
        execution_point: InvariantExecutionPoint,
        failure_effect: InvariantFailureEffect,
        witness: InvariantWitnessKey,
        proof_boundary: Option<InvariantProofBoundaryArtifact>,
        violation: InvariantViolationFields,
        custom_provenance: Option<CustomInvariantProvenance>,
    ) -> Self {
        Self {
            execution_point,
            failure_effect,
            witness,
            proof_boundary,
            violation,
            custom_provenance,
        }
    }

    pub fn execution_point(&self) -> InvariantExecutionPoint {
        self.execution_point
    }

    pub fn failure_effect(&self) -> InvariantFailureEffect {
        self.failure_effect
    }

    pub fn witness(&self) -> &InvariantWitnessKey {
        &self.witness
    }

    pub fn proof_boundary(&self) -> Option<&InvariantProofBoundaryArtifact> {
        self.proof_boundary.as_ref()
    }

    pub fn violation(&self) -> &InvariantViolationFields {
        &self.violation
    }

    pub fn custom_provenance(&self) -> Option<&CustomInvariantProvenance> {
        self.custom_provenance.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomInvariantTraceArtifact {
    rule_id: String,
    semantic_version_major: u16,
    semantic_version_minor: u16,
    execution_point: InvariantExecutionPoint,
    verdict: String,
    provenance: CustomInvariantProvenance,
}

impl CustomInvariantTraceArtifact {
    fn from_result(
        result: &InvariantCheckResult,
        provenance: &CustomInvariantProvenance,
        identity: &crate::contracts::validation::data::rule_id::CustomInvariantSemanticIdentity,
    ) -> Self {
        Self {
            rule_id: identity.rule_id.as_str().to_string(),
            semantic_version_major: identity.semantic_version.major,
            semantic_version_minor: identity.semantic_version.minor,
            execution_point: result.execution_point,
            verdict: result.verdict.diagnostic_label().to_string(),
            provenance: provenance.clone(),
        }
    }

    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }

    pub fn semantic_version_major(&self) -> u16 {
        self.semantic_version_major
    }

    pub fn semantic_version_minor(&self) -> u16 {
        self.semantic_version_minor
    }

    pub fn execution_point(&self) -> InvariantExecutionPoint {
        self.execution_point
    }

    pub fn verdict(&self) -> &str {
        &self.verdict
    }

    pub fn provenance(&self) -> &CustomInvariantProvenance {
        &self.provenance
    }
}

pub fn proof_boundary_artifact(
    result: &InvariantExecutionResult,
) -> Option<InvariantProofBoundaryArtifact> {
    result
        .metadata()
        .proof_boundary()
        .map(InvariantProofBoundaryArtifact::from_summary)
}

pub fn failure_artifact(
    result: &InvariantExecutionResult,
    failure: &InvariantFailure,
) -> InvariantFailureArtifact {
    let matching_result = matching_failure_check_result(result, failure);
    InvariantFailureArtifact::new(
        failure.execution_point(),
        failure.effect(),
        matching_result
            .map(|result| result.witness.clone())
            .unwrap_or_else(|| failure.violation().witness_key()),
        proof_boundary_artifact(result),
        failure.fields().clone(),
        matching_result.and_then(|result| result.custom_provenance().cloned()),
    )
}

pub fn custom_trace_artifact(
    result: &InvariantCheckResult,
) -> Option<CustomInvariantTraceArtifact> {
    let provenance = result.custom_provenance()?;
    let InvariantReportedRule::Custom(identity) = &result.rule else {
        return None;
    };
    Some(CustomInvariantTraceArtifact::from_result(
        result, provenance, identity,
    ))
}

fn matching_failure_check_result<'a>(
    result: &'a InvariantExecutionResult,
    failure: &InvariantFailure,
) -> Option<&'a InvariantCheckResult> {
    result.results().iter().find(|result| {
        matches!(
            &result.verdict,
            InvariantVerdict::Violation(violation) if *violation == *failure.violation()
        )
    })
}
