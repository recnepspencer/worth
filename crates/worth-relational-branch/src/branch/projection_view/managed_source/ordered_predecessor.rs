//! Birth consumes the actual predecessor material at the selected Native image.
use super::ManagedSourceRootImage;
use crate::history::data::coordinates::BranchId;
use crate::identity::data::EntityId;
use crate::validation::candidate_source::availability::ManagedIndexSourceAvailability;
use crate::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial as Denial;
use crate::validation::candidate_source::program::ManagedIndexSourceProgram;
use std::{any::Any, sync::Arc};

impl ManagedSourceRootImage {
    pub fn admit_predecessor_material(
        &self,
        runtime_identity: u64,
        branch: &BranchId,
        root_identity: u64,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
        payload: &Arc<dyn Any + Send + Sync>,
        generation: u64,
    ) -> Result<(), Denial> {
        let core = self.core.upgrade().ok_or(Denial::ForeignSource)?;
        if core.runtime_instance_id() != runtime_identity
            || core.branch_id() != branch
            || self.image.root_id() != root_identity
            || core.registration_generation() != generation
        {
            return Err(Denial::ForeignSource);
        }
        let image = self.selected_source_image()?;
        let entry = image
            .retained(program, scope)?
            .ok_or(Denial::PredecessorUnavailable { predecessor: scope })?;
        if entry.physical_availability() != ManagedIndexSourceAvailability::Available {
            return Err(Denial::PredecessorUnavailable { predecessor: scope });
        }
        let actual = entry
            .payload()
            .ok_or(Denial::PredecessorUnavailable { predecessor: scope })?;
        if !Arc::ptr_eq(actual, payload) {
            return Err(Denial::ForeignSource);
        }
        Ok(())
    }
}
