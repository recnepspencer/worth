//! Logical correspondence cannot select a payload outside its physical basis.
use super::*;
#[cfg(test)]
mod tests;

impl ManagedSourceRootImage {
    pub(crate) fn corresponding_payload(
        &self,
        selected: &PositionedRelationalSnapshot,
        generation: u64,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Denial> {
        self.admit_selected(selected, generation)?;
        let image = self
            .image
            .payload
            .downcast_ref::<ManagedIndexBranchImage>()
            .ok_or(Denial::ForeignSource)?;
        let Some(record) = image.retained(program, scope)? else {
            return Ok(None);
        };
        if record.physical_availability() != ManagedIndexSourceAvailability::Unavailable {
            return Err(Denial::ForeignSource);
        }
        Ok(record.payload().cloned())
    }
}
