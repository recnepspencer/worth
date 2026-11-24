//! Logical correspondence cannot select a payload outside its physical basis.
use super::ManagedSourceRootImage;
use crate::identity::data::EntityId;
use crate::validation::candidate_source::availability::ManagedIndexSourceAvailability;
use crate::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial as Denial;
use crate::validation::candidate_source::program::ManagedIndexSourceProgram;
use crate::visibility::materialization::read_records::reader::positioned_snapshot::PositionedRelationalSnapshot;
use std::{any::Any, sync::Arc};

impl ManagedSourceRootImage {
    pub(crate) fn corresponding_payload(
        &self,
        selected: &PositionedRelationalSnapshot,
        generation: u64,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Denial> {
        self.admit_selected(selected, generation)?;
        let image = self.selected_source_image()?;
        let Some(record) = image.retained(program, scope)? else {
            return Ok(None);
        };
        if record.physical_availability() != ManagedIndexSourceAvailability::Unavailable {
            return Err(Denial::ForeignSource);
        }
        Ok(record.payload().cloned())
    }
}
