//! Prepared source-index publication and deferred native cleanup custody.

use std::sync::Arc;

use worth_relational::facade::mvcc::CompanionBranchCell;
use worth_relational::facade::mvcc::CompanionBranchImage;
use worth_relational::facade::mvcc::CompanionCellEditStop;
use worth_relational::facade::mvcc::CompanionDerivedRootCleanup;
use worth_relational::facade::mvcc::CompanionDerivedRootPreparationStop;
use worth_relational::facade::mvcc::CompanionDerivedRootStopped;
use worth_relational::facade::mvcc::PreparedCompanionDerivedRoot;

use super::super::super::RecordedSettlementIdentity;
use super::super::source_alignment::BranchMarkRoot;
use super::super::SourceInvalidationOwner;
use super::SettlementRegistrationStop;
use crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership;

pub(super) struct PreparedRegistrationWorkCue {
    membership: Arc<RequiredWorkMembership>,
    identity: Arc<RecordedSettlementIdentity>,
}

/// Only the source owner can prepare a registration. Installation consumes
/// this exact native image proof; no caller can substitute a cell or payload.
#[must_use = "prepared source registration must be installed or released"]
pub(in crate::domain_computation::primary_graph) struct PreparedSettlementRegistration {
    native: PreparedCompanionDerivedRoot<BranchMarkRoot>,
    work_cue: Option<PreparedRegistrationWorkCue>,
    completion_cues: Option<super::completion_cues::PreparedCompletionCues>,
}

/// Both old and new image references stay alive until the caller has left its
/// outer publication guards. Their last-owner Drop cannot run inside cutover.
pub(in crate::domain_computation::primary_graph) struct SettlementRegistrationCleanup {
    _completion_cues: Option<super::completion_cues::PublishedCompletionCues>,
    _image: CompanionBranchImage<BranchMarkRoot>,
    _native: CompanionDerivedRootCleanup<BranchMarkRoot>,
    _prior_work_hint: Option<crate::domain_computation::primary_graph::application_output_demand::ReplacedRequiredWorkHint>,
}

pub(in crate::domain_computation::primary_graph) struct StoppedSettlementRegistration {
    native: CompanionDerivedRootStopped<BranchMarkRoot>,
    work_cue: Option<PreparedRegistrationWorkCue>,
    completion_cues: Option<super::completion_cues::PreparedCompletionCues>,
}

impl StoppedSettlementRegistration {
    pub(in crate::domain_computation::primary_graph) fn reason(&self) -> CompanionCellEditStop {
        self.native.reason()
    }

    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (CompanionCellEditStop, PreparedSettlementRegistration) {
        let (reason, native) = self.native.into_parts();
        (
            reason,
            PreparedSettlementRegistration {
                native,
                work_cue: self.work_cue,
                completion_cues: self.completion_cues,
            },
        )
    }
}

impl PreparedSettlementRegistration {
    pub(super) fn retain_completion_cues(
        &mut self,
        cues: Option<super::completion_cues::PreparedCompletionCues>,
    ) {
        self.completion_cues = cues;
    }

    pub(super) fn retain_work_cue(
        &mut self,
        membership: Arc<RequiredWorkMembership>,
        identity: Arc<RecordedSettlementIdentity>,
    ) {
        assert!(self
            .work_cue
            .replace(PreparedRegistrationWorkCue {
                membership,
                identity
            })
            .is_none());
    }

    pub(in crate::domain_computation::primary_graph) fn install(
        self,
    ) -> Result<SettlementRegistrationCleanup, StoppedSettlementRegistration> {
        let PreparedSettlementRegistration {
            native,
            work_cue,
            completion_cues,
        } = self;
        let installed = match native.install() {
            Ok(installed) => installed,
            Err(native) => {
                return Err(StoppedSettlementRegistration {
                    native,
                    work_cue,
                    completion_cues,
                })
            }
        };
        let (image, native) = installed.into_parts();
        let prior_work_hint = work_cue.map(|cue| cue.membership.mark_local_required(cue.identity));
        Ok(SettlementRegistrationCleanup {
            _completion_cues: completion_cues
                .map(super::completion_cues::PreparedCompletionCues::publish),
            _image: image,
            _native: native,
            _prior_work_hint: prior_work_hint,
        })
    }
}

impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn prepare_root_replacement(
        &self,
        cell: CompanionBranchCell<BranchMarkRoot>,
        image: CompanionBranchImage<BranchMarkRoot>,
        root: Arc<BranchMarkRoot>,
    ) -> Result<PreparedSettlementRegistration, SettlementRegistrationStop> {
        let native = cell
            .prepare_derived_root_at_same_position(image, root)
            .map_err(|stop| match stop {
                CompanionDerivedRootPreparationStop::Cell { reason, .. } => {
                    SettlementRegistrationStop::Edit(reason)
                }
            })?;
        Ok(PreparedSettlementRegistration {
            native,
            work_cue: None,
            completion_cues: None,
        })
    }
}
