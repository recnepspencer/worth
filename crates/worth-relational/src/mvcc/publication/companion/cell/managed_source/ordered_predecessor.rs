//! Birth consumes the actual predecessor material at the selected Native image.
use super::*;

impl ManagedSourceRootImage {
    pub(crate) fn admit_predecessor_material(
        &self,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
        payload: &Arc<dyn Any + Send + Sync>,
        generation: u64,
    ) -> Result<(), Denial> {
        let core = self.core.upgrade().ok_or(Denial::ForeignSource)?;
        if core.runtime_instance_id != basis.identity().runtime_instance_id()
            || core.branch_id != *basis.identity().branch_id()
            || self.image.root_id != basis.descriptor().root_identity()
            || core.registration_generation != generation
        {
            return Err(Denial::ForeignSource);
        }
        let image = self
            .image
            .payload
            .downcast_ref::<ManagedIndexBranchImage>()
            .ok_or(Denial::ForeignSource)?;
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
