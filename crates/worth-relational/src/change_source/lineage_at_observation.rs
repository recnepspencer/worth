use super::RelationalObservationReadDenial;
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::data::HistoricalLineageResolution;
use crate::mvcc::RelationalBranchObservation;
use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    /// Resolve the lineage history of one entity as `observation`'s retained
    /// root and commit ancestry see it.
    ///
    /// Returns `None` when the entity has no lineage at that root. Later
    /// commits never change the answer.
    ///
    /// # Errors
    ///
    /// [`RelationalObservationReadDenial::ForeignObservation`] when another
    /// runtime issued `observation`.
    pub fn record_history_at_observation(
        &self,
        observation: &RelationalBranchObservation,
        entity_id: EntityId,
    ) -> Result<Option<HistoricalLineageResolution>, RelationalObservationReadDenial> {
        self.admit_observation_read(observation)?;
        Ok(self
            .lineage_access()
            .resolve_record_history_for_observation(entity_id, observation))
    }

    /// The entities `observation`'s retained root holds for `lineage_ids`.
    ///
    /// # Errors
    ///
    /// [`RelationalObservationReadDenial::ForeignObservation`] when another
    /// runtime issued `observation`.
    pub fn visible_entities_for_lineages_at_observation(
        &self,
        observation: &RelationalBranchObservation,
        lineage_ids: &[LineageId],
    ) -> Result<Vec<EntityId>, RelationalObservationReadDenial> {
        self.admit_observation_read(observation)?;
        Ok(self
            .lineage_access()
            .visible_entity_ids_for_lineages_for_observation(lineage_ids, observation))
    }
}
