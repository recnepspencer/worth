//! The mirror owns its region directory; reader editions are derived views.
use std::collections::BTreeMap;

use super::{PartitionEdition, PartitionMap, ResidentMirrorTarget};
use crate::identity::data::PartitionId;
use crate::storage::residency::RelationalRegionRetention;

#[derive(Debug)]
pub(super) enum MirrorRegion {
    AwaitingResidentTarget(RelationalRegionRetention),
    Resident(ResidentMirrorTarget),
}

#[derive(Debug, Default)]
pub(super) struct CurrentMirror {
    pub(super) edition: PartitionEdition,
    pub(super) regions: BTreeMap<PartitionId, MirrorRegion>,
}

impl CurrentMirror {
    pub(super) fn from_cells(cells: PartitionMap) -> Self {
        let edition = PartitionEdition::new(cells.clone());
        Self {
            edition,
            regions: cells
                .into_iter()
                .map(|(id, cell)| (id, MirrorRegion::AwaitingResidentTarget(cell)))
                .collect(),
        }
    }

    /// Retain an accepted Full witness without loading or treating a cold
    /// region as an absent partition. The awaiting directory is replaced only
    /// after admission; no earlier payload remains in the target spine.
    pub(super) fn admit_target(
        &mut self,
        id: PartitionId,
    ) -> Result<Option<ResidentMirrorTarget>, crate::storage::data::RelationalStorageUnavailable>
    {
        let target = match self.regions.get(&id) {
            None => return Ok(None),
            Some(MirrorRegion::Resident(target)) => {
                target.promote_full(self.edition.resident_pin_lease(id)?);
                return Ok(Some(target.clone()));
            }
            Some(MirrorRegion::AwaitingResidentTarget(region)) => {
                // The explicit guard admitted this edition under its root's
                // kind authority. A copied directory retention has a separate
                // admission cache and cannot promote physically warmed bytes.
                ResidentMirrorTarget::new(
                    self.edition.resident_pin_lease(id)?,
                    region.clears_pins_on_write(),
                )
            }
        };
        self.regions
            .insert(id, MirrorRegion::Resident(target.clone()));
        Ok(Some(target))
    }

    pub(super) fn admit_current_target(
        &mut self,
        id: PartitionId,
    ) -> Result<Option<ResidentMirrorTarget>, crate::storage::data::RelationalStorageUnavailable>
    {
        let target = match self.regions.get(&id) {
            None => return Ok(None),
            Some(MirrorRegion::Resident(target)) => return Ok(Some(target.clone())),
            Some(MirrorRegion::AwaitingResidentTarget(region)) => {
                match self.edition.admitted_full_lease(id) {
                    Some(lease) => ResidentMirrorTarget::new(lease, region.clears_pins_on_write()),
                    _ => ResidentMirrorTarget::from_head(
                        self.edition.resident_head_lease(id)?,
                        region.clears_pins_on_write(),
                    )?,
                }
            }
        };
        self.regions
            .insert(id, MirrorRegion::Resident(target.clone()));
        Ok(Some(target))
    }
}
