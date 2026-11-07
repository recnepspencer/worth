use super::{InstalledIdentityEvolutionKind, InstalledIdentityEvolutionOutcome};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

impl InstalledIdentityEvolutionOutcome {
    pub fn authoritative_subject_count(&self) -> usize {
        if !self.is_authoritative_continuity() {
            return 0;
        }
        self.lifecycle_target.as_ref().map_or_else(
            || {
                self.continuity.as_ref().map_or(0, |evidence| {
                    evidence.successor_authoritative_identities().len()
                })
            },
            |_| 1,
        )
    }

    pub(crate) fn authoritative_subject_evidence_identity(
        &self,
        ordinal: usize,
    ) -> Result<
        Option<crate::evidence_identity::WorthQueryEvidenceIdentity>,
        IdentityDerivationDenial,
    > {
        if !self.is_authoritative_continuity() {
            return Ok(None);
        }
        if let Some(target) = &self.lifecycle_target {
            return Ok((ordinal == 0).then(|| target.evidence_identity())?);
        }
        Ok(self
            .continuity
            .as_ref()
            .successor_authoritative_identities()
            .get(ordinal)
            .map(crate::runtime::WorthQueryMutationAuthorityIdentity::evidence_identity)
            .cloned())
    }

    pub(crate) fn authoritative_subject_entity_identity(
        &self,
        ordinal: usize,
    ) -> Option<&crate::memory_workspace::WorthQueryEntityIdentity> {
        if !self.is_authoritative_continuity() {
            return None;
        }
        // Mutation receipts currently prove only a singular authority-to-entity
        // correspondence. A split or merge may carry several entity deltas, but
        // their vector positions do not prove which authoritative successor
        // names which entity.
        if self.authoritative_subject_count() != 1
            || self.establishing_entity_targets.len() != 1
            || ordinal != 0
        {
            return None;
        }
        self.establishing_entity_targets.first()
    }

    pub(crate) fn establishes_existing_target(
        &self,
        target: &crate::runtime::WorthQueryMutationAuthorityIdentity,
    ) -> bool {
        self.is_authoritative_continuity()
            && self.continuity.as_ref().is_some_and(|continuity| {
                continuity
                    .successor_authoritative_identities()
                    .contains(target)
            })
    }

    pub(crate) fn establishes_generated_target(
        &self,
        target: &crate::memory_workspace::WorthQueryEntityIdentity,
    ) -> bool {
        self.kind() == InstalledIdentityEvolutionKind::GeneratedIdentity
            && self.is_authoritative_continuity()
            && self.lifecycle_target.as_ref() == Some(target)
    }
}
