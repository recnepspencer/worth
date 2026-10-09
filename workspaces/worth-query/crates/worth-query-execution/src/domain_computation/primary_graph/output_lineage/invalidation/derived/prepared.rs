//! Prepared source-index publication and deferred native cleanup custody.

use std::sync::Arc;

use worth_relational::facade::mvcc::{
    CompanionBranchCell, CompanionBranchImage, CompanionCellEditStop,
    CompanionDerivedImageRetention, CompanionDerivedRootAdmission, CompanionDerivedRootCleanup,
    CompanionDerivedRootCost, CompanionDerivedRootPreparationStop, CompanionDerivedRootStopped,
    CompanionPreflightStop, PreparedCompanionDerivedRoot,
};

use super::super::super::RecordedSettlementIdentity;
use super::super::{
    admission::IndexAdmission, retention, source_alignment::BranchMarkRoot,
    InvalidationEditAdmission, SourceInvalidationOwner,
};
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
}

/// Both old and new image references stay alive until the caller has left its
/// outer publication guards. Their last-owner Drop cannot run inside cutover.
pub(in crate::domain_computation::primary_graph) struct SettlementRegistrationCleanup {
    _image: CompanionBranchImage<BranchMarkRoot>,
    _native: CompanionDerivedRootCleanup<BranchMarkRoot>,
    _prior_work_hint: Option<crate::domain_computation::primary_graph::application_output_demand::ReplacedRequiredWorkHint>,
}

pub(in crate::domain_computation::primary_graph) struct StoppedSettlementRegistration {
    native: CompanionDerivedRootStopped<BranchMarkRoot>,
    work_cue: Option<PreparedRegistrationWorkCue>,
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
            },
        )
    }
}

impl PreparedSettlementRegistration {
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
        let PreparedSettlementRegistration { native, work_cue } = self;
        let installed = match native.install() {
            Ok(installed) => installed,
            Err(native) => return Err(StoppedSettlementRegistration { native, work_cue }),
        };
        let (image, native) = installed.into_parts();
        let prior_work_hint = work_cue.map(|cue| cue.membership.mark_local_required(cue.identity));
        Ok(SettlementRegistrationCleanup {
            _image: image,
            _native: native,
            _prior_work_hint: prior_work_hint,
        })
    }
}

impl CompanionDerivedRootAdmission for InvalidationEditAdmission {
    type Stop = CompanionPreflightStop;

    fn admit_derived_root(&mut self, cost: CompanionDerivedRootCost) -> Result<(), Self::Stop> {
        self.work(cost.work)?;
        self.bytes(cost.allocation_bytes)
    }
}

impl SourceInvalidationOwner {
    /// Retention refusal ends this edit, but must not strand the optional live
    /// index. Attempt its paid empty image at the same exact Native position.
    /// The caller still returns the original typed refusal; no output is certified.
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn evict_after_refused_edit(
        &self,
        selected: &worth_relational::facade::runtime::PositionedRelationalSnapshot,
        stop: &SettlementRegistrationStop,
        admission: &mut InvalidationEditAdmission,
    ) {
        if !matches!(
            stop,
            SettlementRegistrationStop::Admission(
                CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. }
            )
        ) {
            return;
        }
        let Ok(Some(cell)) = self.cell_for_read(selected, admission) else {
            return;
        };
        let image = cell.read_image();
        if image.root_id() != selected.root_id()
            || image.commit_id() != selected.commit_id()
            || image.position() != selected.position()
        {
            return;
        }
        let vacant = self
            .branches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .vacant
            .as_ref()
            .cloned();
        let Some(vacant) = vacant else {
            return;
        };
        // The original retention refusal remains the answer if eviction stops.
        if let Ok(prepared) = self.prepare_root_replacement(cell, image, vacant, admission) {
            match prepared.install() {
                Ok(cleanup) => drop(cleanup),
                Err(stopped) => drop(stopped),
            }
        }
    }

    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn prepare_root_replacement(
        &self,
        cell: CompanionBranchCell<BranchMarkRoot>,
        image: CompanionBranchImage<BranchMarkRoot>,
        root: Arc<BranchMarkRoot>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedSettlementRegistration, SettlementRegistrationStop> {
        // Native image lifetime differs from the payload's. Even a shared
        // payload needs one new ticket for this newly allocated native image.
        let bytes = CompanionBranchCell::<BranchMarkRoot>::derived_root_cost().retained_image_bytes;
        let ticket = retention::reserve_live_image(bytes, &self.resources, admission)?;
        let retention = CompanionDerivedImageRetention::from_prepared_owner(ticket);
        let native = cell
            .prepare_derived_root_at_same_position(image, root, retention, admission)
            .map_err(|stop| match stop {
                CompanionDerivedRootPreparationStop::Admission { reason, .. } => {
                    SettlementRegistrationStop::Admission(reason)
                }
                CompanionDerivedRootPreparationStop::Cell { reason, .. } => {
                    SettlementRegistrationStop::Edit(reason)
                }
            })?;
        Ok(PreparedSettlementRegistration {
            native,
            work_cue: None,
        })
    }
}
