use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::operation_authority_chain::{
    mint_operation_phase_proof, operation_phase_basis, WorthQueryOperationPhaseProof,
    WorthQueryPersistentNamingPhase,
};
use crate::domain_installation::operation_identity_basis::canonical_operation_identity;
use crate::domain_installation::WorthQueryCompletedWorkflowTrace;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::{WorthQueryMutationAuthorityIdentity, WorthQueryNamingMutationOutcome};
use worth_proof::TransitionOutcome;

/// A request to bind an already-executed domain naming mutation to one exact
/// authoritative lineage target. It does not author a name or naming policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPersistentNameIntent {
    attachment_identity: WorthQueryMutationAuthorityIdentity,
    target: WorthQueryPersistentNameTarget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryPersistentNameTarget {
    ExistingAuthority(WorthQueryMutationAuthorityIdentity),
    GeneratedEntity(WorthQueryEntityIdentity),
}

impl WorthQueryPersistentNameIntent {
    pub fn from_executed_naming_attachment(
        attachment_identity: WorthQueryMutationAuthorityIdentity,
        target_authority_identity: WorthQueryMutationAuthorityIdentity,
    ) -> Self {
        Self {
            attachment_identity,
            target: WorthQueryPersistentNameTarget::ExistingAuthority(target_authority_identity),
        }
    }

    pub fn from_executed_generated_naming_attachment(
        attachment_identity: WorthQueryMutationAuthorityIdentity,
        target_entity_identity: WorthQueryEntityIdentity,
    ) -> Self {
        Self {
            attachment_identity,
            target: WorthQueryPersistentNameTarget::GeneratedEntity(target_entity_identity),
        }
    }

    pub fn attachment_identity(&self) -> &WorthQueryMutationAuthorityIdentity {
        &self.attachment_identity
    }

    pub fn target(&self) -> &WorthQueryPersistentNameTarget {
        &self.target
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPersistentNameDenial {
    StaleInstallationGeneration,
    LineageMissing,
    LineageEvidenceMissing,
    AdvisoryAmbiguousRetiredOrBroken,
    TargetNotEstablishedByEvidence,
    EstablishingEffectMissing,
    NamingMutationMissing,
    NamingAttachmentMismatch,
    NamingTargetMismatch,
}

pub struct WorthQueryPersistentNameAdmission {
    identity: String,
    lineage_report_identity: String,
    lineage_evidence_index: usize,
    intent: WorthQueryPersistentNameIntent,
    proof: WorthQueryOperationPhaseProof<WorthQueryPersistentNamingPhase>,
}

impl WorthQueryPersistentNameAdmission {
    pub fn identity(&self) -> &str {
        debug_assert_eq!(self.proof.payload().identity(), self.identity);
        &self.identity
    }
    pub fn lineage_report_identity(&self) -> &str {
        &self.lineage_report_identity
    }
    pub fn lineage_evidence_index(&self) -> usize {
        self.lineage_evidence_index
    }
    pub fn intent(&self) -> &WorthQueryPersistentNameIntent {
        &self.intent
    }
}

pub type WorthQueryPersistentNameOutcome = TransitionOutcome<
    WorthQueryPersistentNameAdmission,
    WorthQueryPersistentNameDenial,
    std::convert::Infallible,
    WorthQueryPersistentNameDenial,
    WorthQueryPersistentNameDenial,
    WorthQueryPersistentNameDenial,
>;

impl<D, O, F, L: BasisOperationLane> WorthQueryCompletedWorkflowTrace<D, O, F, L> {
    pub fn admit_persistent_name(
        &self,
        lineage_evidence_index: usize,
        intent: WorthQueryPersistentNameIntent,
    ) -> Result<WorthQueryPersistentNameOutcome, IdentityDerivationDenial> {
        if !self.bound().installation_is_current() {
            return Ok(TransitionOutcome::Stale(
                WorthQueryPersistentNameDenial::StaleInstallationGeneration,
            ));
        }
        let Some(report) = self.lineage_report() else {
            return Ok(denied(WorthQueryPersistentNameDenial::LineageMissing));
        };
        let Some(evidence) = report.evidence().get(lineage_evidence_index) else {
            return Ok(denied(
                WorthQueryPersistentNameDenial::LineageEvidenceMissing,
            ));
        };
        if !evidence.outcome().is_authoritative_continuity() {
            return Ok(denied(
                WorthQueryPersistentNameDenial::AdvisoryAmbiguousRetiredOrBroken,
            ));
        }
        let target_is_established = match intent.target() {
            WorthQueryPersistentNameTarget::ExistingAuthority(target) => {
                evidence.outcome().establishes_existing_target(target)
            }
            WorthQueryPersistentNameTarget::GeneratedEntity(target) => {
                evidence.outcome().establishes_generated_target(target)
            }
        };
        if !target_is_established {
            return Ok(denied(
                WorthQueryPersistentNameDenial::TargetNotEstablishedByEvidence,
            ));
        }
        let Some(effect) = self
            .stage_receipts()
            .iter()
            .find(|stage| stage.stage_identity() == evidence.stage_identity())
            .and_then(|stage| {
                stage.effect_evidence().iter().find(|effect| {
                    evidence
                        .effect_receipt_identities()
                        .iter()
                        .any(|identity| identity == effect.receipt_identity())
                })
            })
        else {
            return Ok(denied(
                WorthQueryPersistentNameDenial::EstablishingEffectMissing,
            ));
        };
        let Some(naming) = effect
            .mutation_receipt()
            .and_then(|receipt| receipt.naming_mutation_evidence())
        else {
            return Ok(denied(
                WorthQueryPersistentNameDenial::NamingMutationMissing,
            ));
        };
        if naming.outcome() == WorthQueryNamingMutationOutcome::Removed
            || naming.attachment_identity() != intent.attachment_identity()
        {
            return Ok(denied(
                WorthQueryPersistentNameDenial::NamingAttachmentMismatch,
            ));
        }
        let naming_target_matches = match intent.target() {
            WorthQueryPersistentNameTarget::ExistingAuthority(target) => naming
                .target_authoritative_identity()
                .is_some_and(|resolved| resolved.is_same_authority_as(target)),
            WorthQueryPersistentNameTarget::GeneratedEntity(target) => {
                naming.outcome() == WorthQueryNamingMutationOutcome::AttachedToNewTarget
                    && naming
                        .resolved_target_entity_identity()
                        .is_some_and(|resolved| resolved.is_same_current_identity_as(target))
            }
        };
        if !naming_target_matches {
            return Ok(denied(WorthQueryPersistentNameDenial::NamingTargetMismatch));
        }
        let identity = canonical_operation_identity(
            "persistent-name-admission-v2",
            vec![
                ("naming.trace", self.identity().to_owned()),
                ("naming.lineage_report", report.identity().to_owned()),
                (
                    "naming.attachment",
                    intent
                        .attachment_identity()
                        .evidence_identity()
                        .terminal_projection_for_reporting()
                        .to_owned(),
                ),
                (
                    "naming.target",
                    persistent_name_target_material(intent.target()),
                ),
            ],
        )?;
        let proof = mint_operation_phase_proof(
            identity.clone(),
            Some(self.phase_proof().payload().identity()),
            operation_phase_basis(self.phase_proof()).clone(),
        );
        Ok(TransitionOutcome::Success(
            WorthQueryPersistentNameAdmission {
                identity,
                lineage_report_identity: report.identity().to_owned(),
                lineage_evidence_index,
                intent,
                proof,
            },
        ))
    }
}

fn persistent_name_target_material(target: &WorthQueryPersistentNameTarget) -> String {
    match target {
        WorthQueryPersistentNameTarget::ExistingAuthority(identity) => format!(
            "existing:{}",
            identity
                .evidence_identity()
                .terminal_projection_for_reporting()
        ),
        WorthQueryPersistentNameTarget::GeneratedEntity(identity) => format!(
            "generated:{}",
            identity
                .evidence_identity()
                .terminal_projection_for_reporting()
        ),
    }
}

fn denied(denial: WorthQueryPersistentNameDenial) -> WorthQueryPersistentNameOutcome {
    TransitionOutcome::Denied(denial)
}
