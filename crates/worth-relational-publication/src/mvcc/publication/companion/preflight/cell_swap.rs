//! Pair the native reserved image replacement with the published cell admission.
use super::PublicationCellAdmission;
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::projection_view::companion_slot::transfer::{
    AdmittedCompanionSlotSwap, PreparedCompanionSlotSwap,
};
use worth_relational_contracts::internal_port::mvcc::publication::companion::preflight_stop::CompanionPreflightStop;
use worth_relational_contracts::internal_port::publication::patch::data::patch_position::PatchStreamPosition;

pub(super) struct PreparedCellSwap {
    pub(super) native: PreparedCompanionSlotSwap,
    pub(super) publication_admission: Option<Arc<PublicationCellAdmission>>,
}

impl PreparedCellSwap {
    pub(super) fn check(&self, registration_generation: u64) -> Result<(), CompanionPreflightStop> {
        self.native.check(registration_generation)
    }

    pub(super) fn enter(&mut self) -> Result<PreparedCellCutover<'_>, CompanionPreflightStop> {
        Ok(PreparedCellCutover {
            native: self.native.enter()?,
            publication_admission: self.publication_admission.as_deref(),
        })
    }
}

pub(super) struct PreparedCellCutover<'a> {
    native: AdmittedCompanionSlotSwap<'a>,
    publication_admission: Option<&'a PublicationCellAdmission>,
}

impl PreparedCellCutover<'_> {
    pub(super) fn install_at(&mut self, position: PatchStreamPosition) {
        self.native.install_at(position);
        if let Some(admission) = self.publication_admission {
            admission.installed(self.native.cell());
        }
    }
}
