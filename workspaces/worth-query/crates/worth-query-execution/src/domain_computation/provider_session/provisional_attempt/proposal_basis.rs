use std::sync::Arc;

use crate::domain_computation::provider_session::{
    WorthQueryFreshDecisionReadSet, WorthQuerySessionEffectAuthority,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProvisionalProposalBasis {
    identity: Arc<str>,
    source_occurrence: Arc<str>,
    search_occurrence: Arc<str>,
    candidate_identity: Arc<str>,
    transformation_evidence: Arc<str>,
    terminal_binding: super::super::WorthQueryProviderSessionTerminalBinding,
    target_generation: u64,
    installed_policy_identity: Arc<str>,
    correspondence_identity: Arc<str>,
    identity_consequence_identity: Arc<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProvisionalProposalBasisParts {
    pub source_occurrence: String,
    pub search_occurrence: String,
    pub candidate_identity: String,
    pub transformation_evidence: String,
    pub target_generation: u64,
    pub installed_policy_identity: String,
    pub correspondence_identity: String,
    pub identity_consequence_identity: String,
}

impl WorthQueryProvisionalProposalBasis {
    pub(super) fn new(
        identity: impl Into<Arc<str>>,
        terminal_binding: super::super::WorthQueryProviderSessionTerminalBinding,
        parts: WorthQueryProvisionalProposalBasisParts,
    ) -> Result<Self, super::WorthQueryProvisionalFailure> {
        let text = [
            &parts.source_occurrence,
            &parts.search_occurrence,
            &parts.candidate_identity,
            &parts.transformation_evidence,
            &parts.installed_policy_identity,
            &parts.correspondence_identity,
            &parts.identity_consequence_identity,
        ];
        if text
            .into_iter()
            .any(|value| value.trim().is_empty() || value.trim() != value)
        {
            return Err(super::WorthQueryProvisionalFailure::invalid_program(
                "proposal basis fields must be non-empty canonical text",
            ));
        }
        Ok(Self {
            identity: identity.into(),
            source_occurrence: parts.source_occurrence.into(),
            search_occurrence: parts.search_occurrence.into(),
            candidate_identity: parts.candidate_identity.into(),
            transformation_evidence: parts.transformation_evidence.into(),
            terminal_binding,
            target_generation: parts.target_generation,
            installed_policy_identity: parts.installed_policy_identity.into(),
            correspondence_identity: parts.correspondence_identity.into(),
            identity_consequence_identity: parts.identity_consequence_identity.into(),
        })
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub(super) fn belongs_to(
        &self,
        terminal: &super::super::WorthQueryProviderSessionTerminalBinding,
    ) -> bool {
        self.terminal_binding.same_session(terminal)
    }

    pub fn target_generation(&self) -> u64 {
        self.target_generation
    }

    pub fn dimensions(&self) -> [&str; 7] {
        [
            &self.source_occurrence,
            &self.search_occurrence,
            &self.candidate_identity,
            &self.transformation_evidence,
            &self.installed_policy_identity,
            &self.correspondence_identity,
            &self.identity_consequence_identity,
        ]
    }
}

impl WorthQuerySessionEffectAuthority<'_> {
    pub fn admit_proposal_basis(
        &self,
        read_set: &WorthQueryFreshDecisionReadSet,
        parts: WorthQueryProvisionalProposalBasisParts,
    ) -> Result<WorthQueryProvisionalProposalBasis, super::WorthQueryProvisionalFailure> {
        if !read_set.belongs_to(self.binding().canonical_identity()) {
            return Err(super::WorthQueryProvisionalFailure::new(
                super::WorthQueryProvisionalDenialKind::ProposalBasisMismatch,
                "proposal does not belong to the exact session and semantic basis",
            ));
        }
        let proposal = WorthQueryProvisionalProposalBasis::new(
            read_set.read_set_identity(),
            self.terminal_binding().clone(),
            parts,
        )?;
        if proposal
            .dimensions()
            .into_iter()
            .any(|identity| !read_set.contains_locator(identity))
        {
            return Err(super::WorthQueryProvisionalFailure::new(
                super::WorthQueryProvisionalDenialKind::ProposalBasisMismatch,
                "proposal provenance was not observed in the admitted decision read-set",
            ));
        }
        Ok(proposal)
    }
}
