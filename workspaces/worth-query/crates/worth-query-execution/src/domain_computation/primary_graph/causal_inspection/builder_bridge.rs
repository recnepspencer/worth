use worth_runtime_bridge::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEnvelopeDenial, BridgeCausalEvidenceFamily,
    BridgeCausalEvidenceOwner, BridgeCausalEvidenceReference,
    BridgeCausalEvidenceReferenceIdentity, BridgeCausalInspectionAdmissionSummary,
    BridgeIdentityEvidence, RuntimeBridge,
};

use crate::domain_computation::evidence_identity::WorthQueryEvidenceIdentity;

use super::admission::{
    AdmittedCausalInspection, AdvisoryCausalInspection, CausalInspectionProofFlow,
};
use super::builder::CausalInspectionPlan;
use super::inventory::{CausalEvidenceFamily, CausalEvidenceOwner};
use super::materialization::{
    materialize_admitted_causal_inspection, materialize_advisory_causal_inspection,
    materialize_bridge_denied_admitted_causal_inspection,
    materialize_bridge_denied_advisory_causal_inspection, materialize_denied_causal_inspection,
    CausalInspectionMaterializationError, QueryCausalInspectionArtifact,
};
use super::observation_identity::CausalEvidenceReferenceDigest;
use super::reference::CausalEvidenceReference;

impl CausalInspectionPlan {
    pub fn materialize_with_bridge(
        &self,
        bridge: &RuntimeBridge,
    ) -> Result<QueryCausalInspectionArtifact, CausalInspectionMaterializationError> {
        match &self.admission {
            CausalInspectionProofFlow::Denied(inspection) => {
                Ok(materialize_denied_causal_inspection(
                    inspection,
                    None,
                    self.redaction_policy,
                    self.materialization_policy,
                ))
            }
            CausalInspectionProofFlow::Admitted(inspection) => {
                self.materialize_admitted_with_bridge(inspection, bridge)
            }
            CausalInspectionProofFlow::Advisory(inspection) => {
                self.materialize_advisory_with_bridge(inspection, bridge)
            }
        }
    }

    fn materialize_admitted_with_bridge(
        &self,
        inspection: &AdmittedCausalInspection,
        bridge: &RuntimeBridge,
    ) -> Result<QueryCausalInspectionArtifact, CausalInspectionMaterializationError> {
        match bridge
            .diagnostics()
            .assemble_causal_explanation_envelope(self.bridge_request_for_admitted(inspection)?)
        {
            Ok(envelope) => materialize_admitted_causal_inspection(
                inspection,
                &envelope,
                self.redaction_policy,
                self.materialization_policy,
            ),
            Err(denial) => Ok(materialize_bridge_denied_admitted_causal_inspection(
                inspection,
                &denial,
                self.redaction_policy,
                self.materialization_policy,
            )),
        }
    }

    fn materialize_advisory_with_bridge(
        &self,
        inspection: &AdvisoryCausalInspection,
        bridge: &RuntimeBridge,
    ) -> Result<QueryCausalInspectionArtifact, CausalInspectionMaterializationError> {
        match bridge
            .diagnostics()
            .assemble_causal_explanation_envelope(self.bridge_request_for_advisory(inspection)?)
        {
            Ok(envelope) => materialize_advisory_causal_inspection(
                inspection,
                &envelope,
                self.redaction_policy,
                self.materialization_policy,
            ),
            Err(denial) => Ok(materialize_bridge_denied_advisory_causal_inspection(
                inspection,
                &denial,
                self.redaction_policy,
                self.materialization_policy,
            )),
        }
    }

    fn bridge_request_for_admitted(
        &self,
        inspection: &AdmittedCausalInspection,
    ) -> Result<BridgeCausalEnvelopeAssemblyRequest, CausalInspectionMaterializationError> {
        let summary = BridgeCausalInspectionAdmissionSummary::admitted(
            inspection
                .admitted_inspection_identity()
                .evidence_identity()
                .bridge_evidence_identity(),
            inspection
                .subject()
                .anchor_identity()
                .evidence_identity()
                .bridge_evidence_identity(),
        )
        .map_err(materialization_error_from_bridge_denial)?;
        bridge_request_from_summary(
            summary,
            inspection.subject().query_observation_evidence_identity(),
            self.reference_set.references(),
        )
    }

    fn bridge_request_for_advisory(
        &self,
        inspection: &AdvisoryCausalInspection,
    ) -> Result<BridgeCausalEnvelopeAssemblyRequest, CausalInspectionMaterializationError> {
        let summary = BridgeCausalInspectionAdmissionSummary::advisory(
            inspection
                .advisory_inspection_identity()
                .evidence_identity()
                .bridge_evidence_identity(),
            inspection
                .subject()
                .anchor_identity()
                .evidence_identity()
                .bridge_evidence_identity(),
        )
        .map_err(materialization_error_from_bridge_denial)?;
        bridge_request_from_summary(
            summary,
            inspection.subject().query_observation_evidence_identity(),
            self.reference_set.references(),
        )
    }
}

fn bridge_request_from_summary(
    summary: BridgeCausalInspectionAdmissionSummary,
    query_observation_identity: &WorthQueryEvidenceIdentity,
    references: &[CausalEvidenceReference],
) -> Result<BridgeCausalEnvelopeAssemblyRequest, CausalInspectionMaterializationError> {
    let mut bridge_references = vec![BridgeCausalEvidenceReference::new(
        BridgeCausalEvidenceOwner::Query,
        BridgeCausalEvidenceFamily::QueryObservation,
        BridgeCausalEvidenceReferenceIdentity::query_observation(bridge_query_evidence_identity(
            query_observation_identity,
        ))
        .map_err(materialization_error_from_bridge_denial)?,
    )
    .map_err(materialization_error_from_bridge_denial)?];
    for reference in references {
        if let Some((owner, family)) =
            bridge_reference_family(reference.owner(), reference.family())
        {
            bridge_references.push(
                BridgeCausalEvidenceReference::new(
                    owner,
                    family,
                    bridge_reference_identity(owner, family, reference.reference_digest())?,
                )
                .map_err(materialization_error_from_bridge_denial)?,
            );
        }
    }
    BridgeCausalEnvelopeAssemblyRequest::from_query_admission(summary, bridge_references)
        .map_err(materialization_error_from_bridge_denial)
}

fn bridge_reference_identity(
    owner: BridgeCausalEvidenceOwner,
    family: BridgeCausalEvidenceFamily,
    reference_digest: &CausalEvidenceReferenceDigest,
) -> Result<BridgeCausalEvidenceReferenceIdentity, CausalInspectionMaterializationError> {
    let identity = match owner {
        BridgeCausalEvidenceOwner::Query => {
            BridgeCausalEvidenceReferenceIdentity::query_observation(
                bridge_query_evidence_identity(reference_digest.evidence_identity()),
            )
        }
        BridgeCausalEvidenceOwner::RuntimeBridge => {
            BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
                family,
                reference_digest.bridge_authority_evidence(),
            )
        }
        BridgeCausalEvidenceOwner::Relational => {
            BridgeCausalEvidenceReferenceIdentity::relational_authority(
                reference_digest.bridge_authority_evidence(),
            )
        }
        BridgeCausalEvidenceOwner::Signal => BridgeCausalEvidenceReferenceIdentity::signal(
            family,
            reference_digest.bridge_authority_evidence(),
        ),
    };
    identity.map_err(materialization_error_from_bridge_denial)
}

fn bridge_query_evidence_identity(identity: &WorthQueryEvidenceIdentity) -> BridgeIdentityEvidence {
    identity.bridge_evidence_identity()
}

fn bridge_reference_family(
    owner: CausalEvidenceOwner,
    family: CausalEvidenceFamily,
) -> Option<(BridgeCausalEvidenceOwner, BridgeCausalEvidenceFamily)> {
    let bridge_owner = match owner {
        CausalEvidenceOwner::Query => return None,
        CausalEvidenceOwner::RuntimeBridge => BridgeCausalEvidenceOwner::RuntimeBridge,
        CausalEvidenceOwner::Relational => BridgeCausalEvidenceOwner::Relational,
        CausalEvidenceOwner::Signal => BridgeCausalEvidenceOwner::Signal,
    };
    let bridge_family = bridge_family_for(family)?;
    Some((bridge_owner, bridge_family))
}

fn bridge_family_for(family: CausalEvidenceFamily) -> Option<BridgeCausalEvidenceFamily> {
    Some(match family {
        CausalEvidenceFamily::RelationalAuthority | CausalEvidenceFamily::RelationalDecision => {
            BridgeCausalEvidenceFamily::RelationalAuthority
        }
        CausalEvidenceFamily::BridgeRoute => BridgeCausalEvidenceFamily::BridgeRoute,
        CausalEvidenceFamily::BridgeEvaluation => {
            BridgeCausalEvidenceFamily::BridgeHistoricalEvaluation
        }
        CausalEvidenceFamily::BridgeSourceMaterialization => {
            BridgeCausalEvidenceFamily::BridgeSourceMaterialization
        }
        CausalEvidenceFamily::BridgeSourceFailure => {
            BridgeCausalEvidenceFamily::BridgeSourceFailure
        }
        CausalEvidenceFamily::BridgeContinuity => BridgeCausalEvidenceFamily::BridgeContinuity,
        CausalEvidenceFamily::BridgeMerge => BridgeCausalEvidenceFamily::BridgeMerge,
        CausalEvidenceFamily::BridgeStructural => BridgeCausalEvidenceFamily::BridgeStructuralRemap,
        CausalEvidenceFamily::BridgeStream | CausalEvidenceFamily::BridgeReplay => {
            BridgeCausalEvidenceFamily::BridgeStreamReplay
        }
        CausalEvidenceFamily::BridgePreview => BridgeCausalEvidenceFamily::BridgePreviewExecution,
        CausalEvidenceFamily::BridgeWriteback => {
            BridgeCausalEvidenceFamily::BridgeWritebackExecution
        }
        CausalEvidenceFamily::BridgeMapper => BridgeCausalEvidenceFamily::BridgeWritebackMapper,
        CausalEvidenceFamily::SignalInvalidation => BridgeCausalEvidenceFamily::SignalInvalidation,
        CausalEvidenceFamily::SignalEvaluation => BridgeCausalEvidenceFamily::SignalEvaluation,
        CausalEvidenceFamily::SignalForensicAvailability => {
            BridgeCausalEvidenceFamily::SignalForensicAvailability
        }
        CausalEvidenceFamily::SignalReplayCursor => BridgeCausalEvidenceFamily::SignalReplayCursor,
        CausalEvidenceFamily::SignalLineage | CausalEvidenceFamily::Lineage => {
            BridgeCausalEvidenceFamily::SignalLineage
        }
        CausalEvidenceFamily::SignalProvenance => BridgeCausalEvidenceFamily::SignalProvenance,
        CausalEvidenceFamily::QueryInspection
        | CausalEvidenceFamily::QueryMutationCausality
        | CausalEvidenceFamily::QueryMutationProvenance
        | CausalEvidenceFamily::Provenance
        | CausalEvidenceFamily::Policy
        | CausalEvidenceFamily::Redaction => return None,
    })
}

fn materialization_error_from_bridge_denial(
    denial: BridgeCausalEnvelopeDenial,
) -> CausalInspectionMaterializationError {
    CausalInspectionMaterializationError::from_bridge_assembly_denial(&denial)
}
