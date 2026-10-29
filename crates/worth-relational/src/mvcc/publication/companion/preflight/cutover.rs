//! Candidate-bound, callback-free companion installation after Native verifies the head.
use super::cell_swap::{PreparedCellCutover, PreparedCellSwap};
use super::{
    CandidateCompanionBinding, CompanionBranchCellCore, CompanionPreflightStop,
    CompanionPublicationCompletionObserver, CompanionRootImage, PublicationCellAdmission,
    PublicationCompanionPreflight,
};
use crate::publication::patch::data::PatchStreamPosition;

/// Non-cloneable, candidate-bound successor image. Only Relational installs it.
pub struct PreparedPublicationCompanionEffect {
    pub(super) binding: CandidateCompanionBinding,
    pub(super) primary: Option<PreparedCellSwap>,
    pub(super) managed: Option<PreparedCellSwap>,
    pub(super) completion: Option<CompanionPublicationCompletionObserver>,
}

impl PreparedPublicationCompanionEffect {
    pub(in crate::mvcc) fn matches_read_condition(
        &self,
        condition: &super::super::cell::CompanionReadCondition,
    ) -> Result<(), super::CompanionPreflightStop> {
        let primary = self.primary.as_ref().expect("sealed primary swap");
        match condition.matches_reserved(&primary.cell, &primary.pinned_image) {
            Err(CompanionPreflightStop::ForeignCell) => match &self.managed {
                Some(managed) => condition.matches_reserved(&managed.cell, &managed.pinned_image),
                None => Err(CompanionPreflightStop::ForeignCell),
            },
            result => result,
        }
    }

    /// The caller funds this scheduling observer before native publication.
    /// It never changes the authority or selected-image checks of the effect.
    pub fn attach_completion_observer(
        &mut self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<CompanionPublicationCompletionObserver, CompanionPreflightStop> {
        context.checkpoint()?;
        Ok(self
            .completion
            .get_or_insert_with(CompanionPublicationCompletionObserver::new)
            .clone())
    }

    pub(crate) fn matches(
        &self,
        binding: &CandidateCompanionBinding,
    ) -> Result<(), CompanionPreflightStop> {
        if self.binding != *binding {
            return Err(CompanionPreflightStop::RegistrationChanged);
        }
        self.primary
            .as_ref()
            .expect("sealed primary swap")
            .check(binding.registration_generation)?;
        if let Some(managed) = &self.managed {
            managed.check(binding.registration_generation)?;
        }
        Ok(())
    }

    /// Acquire the fixed swap cell before any branch movement.
    pub(crate) fn enter_cutover(
        &mut self,
    ) -> Result<PreparedCompanionCutover<'_>, CompanionPreflightStop> {
        // Both guards must be held before Native moves the graph. Failure to
        // acquire the second drops the first without installing either image.
        let primary = self
            .primary
            .as_mut()
            .expect("sealed primary swap")
            .enter()?;
        let managed = self
            .managed
            .as_mut()
            .map(PreparedCellSwap::enter)
            .transpose()?;
        Ok(PreparedCompanionCutover {
            primary,
            managed,
            completion: self.completion.as_ref(),
        })
    }
}

pub(crate) struct PreparedCompanionCutover<'a> {
    primary: PreparedCellCutover<'a>,
    managed: Option<PreparedCellCutover<'a>>,
    completion: Option<&'a CompanionPublicationCompletionObserver>,
}

impl PreparedCompanionCutover<'_> {
    /// No callbacks, allocation, failure, or retired-payload Drop occur here.
    pub(crate) fn install_at(&mut self, position: PatchStreamPosition) {
        self.primary.install_at(position);
        if let Some(managed) = &mut self.managed {
            managed.install_at(position);
        }
        if let Some(completion) = self.completion {
            completion.installed();
        }
    }
}

impl Drop for PreparedPublicationCompanionEffect {
    fn drop(&mut self) {
        if let Some(completion) = &self.completion {
            completion.abort_uninstalled();
        }
    }
}
