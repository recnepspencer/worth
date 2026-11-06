use super::{
    IdentityEvolutionExecutionArtifact, IdentityEvolutionExecutionFamily,
    IdentityEvolutionOutcomeFamily, PromotionOrMergeAuthorityState,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::{WorthQueryContinuityMutationEvidence, WorthQueryContinuityOutcomeClass};
#[path = "installed/authority_subjects.rs"]
mod authority_subjects;
#[path = "installed/authority_validation.rs"]
mod authority_validation;
#[path = "installed/foundational_attachment.rs"]
mod foundational_attachment;
#[path = "installed/replay_semantics.rs"]
mod replay_semantics;
use authority_validation::{engine_matches_authority, semantic_evidence_identity};
use foundational_attachment::FoundationalInstalledLineageIdentities;
pub use foundational_attachment::WorthQueryFoundationalLineageAttachment;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstalledIdentityEvolutionKind {
    PreservedIdentity,
    SingularSuccessor,
    SplitSuccessors,
    MergedPredecessors,
    GeneratedIdentity,
    RetiredIdentity,
    AdvisoryCorrespondence,
    AmbiguousCorrespondence,
    ContinuityBreak,
    Denied,
}

/// Identity-evolution evidence executed under one exact installed workflow stage.
///
/// There is deliberately no public constructor. The only minting path is
/// `WorthQueryWorkflowStageExecutionContext::execute_identity_evolution`, which
/// runs Query's admitted identity-evolution engine and binds its artifact to the
/// installed operation, run, and stage that requested it.
#[derive(Clone, Debug)]
pub struct InstalledIdentityEvolutionOutcome {
    artifact: IdentityEvolutionExecutionArtifact,
    continuity: Option<WorthQueryContinuityMutationEvidence>,
    lifecycle_target: Option<crate::memory_workspace::WorthQueryEntityIdentity>,
    establishing_entity_targets: Vec<crate::memory_workspace::WorthQueryEntityIdentity>,
    operation_identity: String,
    run_identity: String,
    stage_identity: String,
    establishing_effect_receipt_identity: String,
    semantic_identity: String,
    foundational_identities: FoundationalInstalledLineageIdentities,
}

pub(crate) struct InstalledIdentityEvolutionBinding<'a> {
    pub(crate) operation_identity: &'a str,
    pub(crate) run_identity: &'a str,
    pub(crate) stage_identity: &'a str,
    pub(crate) effect_receipt_identity: String,
    pub(crate) establishing_entity_targets: Vec<crate::memory_workspace::WorthQueryEntityIdentity>,
}

impl InstalledIdentityEvolutionOutcome {
    pub(crate) fn from_execution(
        artifact: IdentityEvolutionExecutionArtifact,
        continuity: Option<WorthQueryContinuityMutationEvidence>,
        lifecycle_target: Option<crate::memory_workspace::WorthQueryEntityIdentity>,
        binding: InstalledIdentityEvolutionBinding<'_>,
    ) -> Result<Option<Self>, IdentityDerivationDenial> {
        if !engine_matches_authority(&artifact, continuity.as_ref(), lifecycle_target.as_ref())? {
            return Ok(None);
        }
        let semantic_evidence = semantic_evidence_identity(
            &artifact,
            continuity.as_ref(),
            lifecycle_target.as_ref(),
            &binding.establishing_entity_targets,
        )?;
        let semantic_identity = semantic_evidence
            .terminal_projection_for_reporting()
            .to_owned();
        let foundational_identities = FoundationalInstalledLineageIdentities::from_execution(
            &semantic_evidence,
            artifact.basis_digest(),
            &binding.effect_receipt_identity,
        )?;
        Ok(Some(Self {
            artifact,
            continuity,
            lifecycle_target,
            establishing_entity_targets: binding.establishing_entity_targets,
            operation_identity: binding.operation_identity.to_owned(),
            run_identity: binding.run_identity.to_owned(),
            stage_identity: binding.stage_identity.to_owned(),
            establishing_effect_receipt_identity: binding.effect_receipt_identity,
            semantic_identity,
            foundational_identities,
        }))
    }

    /// Descriptive execution metadata from the existing identity-evolution
    /// engine. Operational continuity comes from `continuity_evidence()`.
    pub fn engine_artifact(&self) -> &IdentityEvolutionExecutionArtifact {
        &self.artifact
    }

    pub fn continuity_evidence(&self) -> Option<&WorthQueryContinuityMutationEvidence> {
        self.continuity.as_ref()
    }

    pub fn kind(&self) -> InstalledIdentityEvolutionKind {
        if self.artifact.family() == &IdentityEvolutionExecutionFamily::GeneratedIdentity {
            return InstalledIdentityEvolutionKind::GeneratedIdentity;
        }
        if self.artifact.family() == &IdentityEvolutionExecutionFamily::RetiredIdentity {
            return InstalledIdentityEvolutionKind::RetiredIdentity;
        }
        match self.artifact.result_bundle().outcome_family() {
            IdentityEvolutionOutcomeFamily::AdvisoryIdentityCandidateSet => {
                return InstalledIdentityEvolutionKind::AdvisoryCorrespondence;
            }
            IdentityEvolutionOutcomeFamily::Ambiguity => {
                return InstalledIdentityEvolutionKind::AmbiguousCorrespondence;
            }
            IdentityEvolutionOutcomeFamily::IdentityBreak => {
                return InstalledIdentityEvolutionKind::ContinuityBreak;
            }
            _ => {}
        }
        let Some(continuity) = self.continuity.as_ref() else {
            return InstalledIdentityEvolutionKind::Denied;
        };
        match continuity.outcome_class() {
            WorthQueryContinuityOutcomeClass::ContinuesAsSingleSuccessor => {
                if continuity
                    .successor_authoritative_identity()
                    .is_some_and(|successor| {
                        successor.is_same_authority_as(continuity.prior_authoritative_identity())
                    })
                {
                    InstalledIdentityEvolutionKind::PreservedIdentity
                } else {
                    InstalledIdentityEvolutionKind::SingularSuccessor
                }
            }
            WorthQueryContinuityOutcomeClass::ContinuesAsSplitSuccessors => {
                InstalledIdentityEvolutionKind::SplitSuccessors
            }
            WorthQueryContinuityOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor => {
                InstalledIdentityEvolutionKind::MergedPredecessors
            }
            WorthQueryContinuityOutcomeClass::RejectedAmbiguousSuccessor => {
                InstalledIdentityEvolutionKind::AmbiguousCorrespondence
            }
            WorthQueryContinuityOutcomeClass::RejectedNoAuthoritativeSuccessor
            | WorthQueryContinuityOutcomeClass::RejectedHistoricalResolutionFailure => {
                InstalledIdentityEvolutionKind::ContinuityBreak
            }
            WorthQueryContinuityOutcomeClass::RejectedUnsupportedContinuityClass => {
                InstalledIdentityEvolutionKind::Denied
            }
        }
    }

    pub fn width(&self) -> usize {
        match self.kind() {
            InstalledIdentityEvolutionKind::GeneratedIdentity => 1,
            InstalledIdentityEvolutionKind::RetiredIdentity
            | InstalledIdentityEvolutionKind::AdvisoryCorrespondence
            | InstalledIdentityEvolutionKind::AmbiguousCorrespondence
            | InstalledIdentityEvolutionKind::ContinuityBreak
            | InstalledIdentityEvolutionKind::Denied => 0,
            _ => self.continuity.as_ref().map_or(0, |evidence| {
                evidence.successor_authoritative_identities().len()
            }),
        }
    }

    pub fn is_authoritative_continuity(&self) -> bool {
        matches!(
            self.kind(),
            InstalledIdentityEvolutionKind::PreservedIdentity
                | InstalledIdentityEvolutionKind::SingularSuccessor
                | InstalledIdentityEvolutionKind::SplitSuccessors
                | InstalledIdentityEvolutionKind::MergedPredecessors
                | InstalledIdentityEvolutionKind::GeneratedIdentity
        ) && self
            .artifact
            .result_bundle()
            .metadata()
            .promotion_or_merge_authority_state()
            != PromotionOrMergeAuthorityState::RequiredButUnavailable
    }

    pub fn foundational_outcome_kind(
        &self,
    ) -> worth_foundational::facade::FoundationalBoundaryEvidenceLineageOutcomeKind {
        use worth_foundational::facade::FoundationalBoundaryEvidenceLineageOutcomeKind as Kind;
        match self.kind() {
            InstalledIdentityEvolutionKind::PreservedIdentity => Kind::SingularContinuity,
            InstalledIdentityEvolutionKind::SingularSuccessor => Kind::SingularContinuity,
            InstalledIdentityEvolutionKind::SplitSuccessors => Kind::PluralSuccessorPredecessor,
            InstalledIdentityEvolutionKind::MergedPredecessors => Kind::MergeSuccessor,
            InstalledIdentityEvolutionKind::GeneratedIdentity => Kind::SingularContinuity,
            InstalledIdentityEvolutionKind::RetiredIdentity => Kind::IdentityBreak,
            InstalledIdentityEvolutionKind::AdvisoryCorrespondence => {
                Kind::AdvisoryCorrespondenceCandidate
            }
            InstalledIdentityEvolutionKind::AmbiguousCorrespondence => Kind::Ambiguity,
            InstalledIdentityEvolutionKind::ContinuityBreak
            | InstalledIdentityEvolutionKind::Denied => Kind::IdentityBreak,
        }
    }

    pub(crate) fn binds(
        &self,
        operation: &str,
        run: &str,
        stage: &str,
        stage_effect_receipt_identities: &std::collections::BTreeSet<&str>,
    ) -> bool {
        self.operation_identity == operation
            && self.run_identity == run
            && self.stage_identity == stage
            && stage_effect_receipt_identities
                .contains(self.establishing_effect_receipt_identity.as_str())
    }

    pub(crate) fn establishing_effect_receipt_identity(&self) -> Option<&str> {
        Some(&self.establishing_effect_receipt_identity)
    }

    pub(crate) fn semantic_identity(&self) -> &str {
        &self.semantic_identity
    }

    pub(crate) fn foundational_attested_lineage(
        &self,
    ) -> Option<WorthQueryFoundationalLineageAttachment> {
        self.foundational_identities
            .attested_lineage(self.kind(), self.is_authoritative_continuity())
    }
}
