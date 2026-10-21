//! The actual advance mode of a required successor travels with its demand.
//!
//! This is custody from the executed predecessor Ready and its authenticated
//! installed provider. It never authorizes a fresh operation by itself.

mod validation;

use crate::domain_computation::primary_graph::application_contribution::producer::registry::InstalledProducerEdition;
use crate::domain_computation::primary_graph::application_contribution::producer::registry::InstalledProducerProvider;
use crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerCommitAuthority;
use crate::domain_computation::primary_graph::application_output_demand::SelectedRequiredRefreshClaim;

use super::WorthQueryOutputDemandDenial;
use super::WorthQueryOutputDemandDenialKind;

#[derive(Default)]
pub(super) enum DemandProgressionProvenance {
    #[default]
    Ordinary,
    RequiredSuccessor(RequiredSuccessorProvenance),
}

pub(super) struct RequiredSuccessorProvenance {
    commit_authority: WorthQueryProducerCommitAuthority,
    installed_edition: InstalledProducerEdition,
}

impl RequiredSuccessorProvenance {
    /// Mint only from one registry-issued Ready claim and its exact installed
    /// provider. The installed program is checked again before any effect.
    pub(super) fn prepare_from_ready<Schema>(
        claim: &SelectedRequiredRefreshClaim,
        installed: &InstalledProducerProvider<Schema>,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        const SUBJECT: &str = "required successor producer differs from accepted Ready";

        // Claim, selected producer, installed declaration, and its width are
        // all inspected before the variable text comparison is prepared.

        let expected = claim.selected().producer_identity();
        let actual = &installed.declaration.identity;

        if expected != actual
            || claim.selected().key().family_identity() != installed.declaration.output_family
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                SUBJECT,
            ));
        }
        Ok(Self {
            commit_authority: claim.commit_authority().clone(),
            installed_edition: installed.edition,
        })
    }

    pub(super) fn commit_authority(&self) -> &WorthQueryProducerCommitAuthority {
        &self.commit_authority
    }

    /// After typed family selection, execution follows the actual immutable
    /// successor entry. The exact predecessor claim still owns its mode.
    pub(super) fn bind_successor(&mut self, edition: InstalledProducerEdition) {
        self.installed_edition = edition;
    }
}

impl DemandProgressionProvenance {
    pub(super) fn required(&self) -> Option<&RequiredSuccessorProvenance> {
        match self {
            Self::Ordinary => None,
            Self::RequiredSuccessor(required) => Some(required),
        }
    }
}
