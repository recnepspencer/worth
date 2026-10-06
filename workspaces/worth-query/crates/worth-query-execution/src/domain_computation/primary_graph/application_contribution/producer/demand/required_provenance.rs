//! The actual advance mode of a required successor travels with its demand.
//!
//! This is custody from the executed predecessor Ready and its authenticated
//! installed provider. It never authorizes a fresh operation by itself.

mod validation;

use crate::domain_computation::primary_graph::application_contribution::producer::{
    registry::{InstalledProducerEdition, InstalledProducerProvider},
    WorthQueryProducerCommitAuthority,
};
use crate::domain_computation::primary_graph::{
    application_output_demand::SelectedRequiredRefreshClaim,
    output_lineage::invalidation::InvalidationEditAdmission,
};

use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};

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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        const SUBJECT: &str = "required successor producer differs from accepted Ready";
        let work_denial = || {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                "",
            )
        };
        let capacity_denial = || {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                "",
            )
        };
        // Claim, selected producer, installed declaration, and its width are
        // all inspected before the variable text comparison is prepared.
        admission
            .charge_external_work(4)
            .map_err(|_| work_denial())?;
        let expected = claim.selected().producer_identity();
        let actual = &installed.declaration.identity;
        let copied_mode = std::mem::size_of::<WorthQueryProducerCommitAuthority>()
            .checked_add(std::mem::size_of::<InstalledProducerEdition>())
            .ok_or_else(work_denial)?;
        let work = expected
            .len()
            .checked_add(actual.len())
            .and_then(|bytes| bytes.checked_add(SUBJECT.len()))
            .and_then(|bytes| bytes.checked_add(copied_mode))
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(u64::try_from(work).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        let denial_backing = SUBJECT
            .len()
            .checked_add(std::mem::size_of::<String>())
            .ok_or_else(capacity_denial)?;
        admission
            .admit_read_scratch(u64::try_from(denial_backing).map_err(|_| capacity_denial())?)
            .map_err(|stop| match stop {
                worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted {
                    ..
                }
                | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => {
                    work_denial()
                }
                _ => capacity_denial(),
            })?;
        if expected != actual
            || claim.selected().key().family_type() != installed.declaration.output_family_type
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
