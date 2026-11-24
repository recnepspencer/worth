//! Retained Source identity owns no live cell, newer image or execution request.
use super::ManagedSourceRootImage;
use crate::mvcc::publication::companion::preflight_stop::CompanionPreflightStop;
use crate::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial as Denial;
use crate::visibility::materialization::read_records::reader::positioned_snapshot::PositionedRelationalSnapshot;

pub(crate) use crate::branch::managed_source_affinity::ManagedSourceCellAffinity;
pub(crate) fn admit_affinity(
    affinity: &ManagedSourceCellAffinity,
    selected: &PositionedRelationalSnapshot,
    generation: u64,
) -> Result<(), Denial> {
    if affinity.generation != generation {
        return Err(Denial::Publication(
            CompanionPreflightStop::RegistrationChanged,
        ));
    }
    if affinity.runtime != selected.runtime_instance_id()
        || affinity.branch != *selected.branch_id()
    {
        return Err(Denial::ForeignSource);
    }
    Ok(())
}

impl ManagedSourceRootImage {
    pub(crate) fn admit_selected(
        &self,
        selected: &PositionedRelationalSnapshot,
        generation: u64,
    ) -> Result<(), Denial> {
        admit_affinity(&self.affinity, selected, generation)?;
        if self.image.root_id() != selected.root_id()
            || self.image.commit_id() != selected.commit_id()
            || self.image.position() != selected.position()
        {
            return Err(Denial::ForeignSource);
        }
        Ok(())
    }
}
