use worth_foundational::facade::AspectKey;

use super::RelationalObservationReadDenial;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::mvcc::RelationalBranchObservation;
use crate::runtime::RelationalRuntime;
use crate::storage::data::{EntityReadRecord, RelationReadRecord};

impl RelationalRuntime {
    /// Read one entity exactly as `observation`'s retained root holds it.
    ///
    /// Returns `None` when the root has no such entity, or holds a different
    /// generation of its slot. Later commits never change the answer.
    ///
    /// # Errors
    ///
    /// [`RelationalObservationReadDenial::ForeignObservation`] when another
    /// runtime issued `observation`.
    pub fn entity_record_at_observation(
        &self,
        observation: &RelationalBranchObservation,
        entity_id: EntityId,
    ) -> Result<Option<EntityReadRecord>, RelationalObservationReadDenial> {
        self.admit_observation_read(observation)?;
        let root = observation.selected_root();
        Ok(self
            .read_truth()
            .authoritative_entity_record_for_id_from_exact_state(
                root.as_ref(),
                root.schema_authority().registry(),
                entity_id,
            ))
    }

    /// Read one relation exactly as `observation`'s retained root holds it.
    ///
    /// Returns `None` when the root has no such relation. Later commits never
    /// change the answer.
    ///
    /// # Errors
    ///
    /// [`RelationalObservationReadDenial::ForeignObservation`] when another
    /// runtime issued `observation`.
    pub fn relation_record_at_observation(
        &self,
        observation: &RelationalBranchObservation,
        relation_id: RelationId,
    ) -> Result<Option<RelationReadRecord>, RelationalObservationReadDenial> {
        self.admit_observation_read(observation)?;
        let root = observation.selected_root();
        Ok(self
            .read_truth()
            .authoritative_relation_record_for_id_from_exact_state(
                root.as_ref(),
                root.schema_authority().registry(),
                relation_id,
            ))
    }
}

impl RelationalBranchObservation {
    /// Whether the schema retained with this observation declares `aspect`
    /// for entity kind `kind_id`. A later schema change never alters the
    /// answer.
    pub fn entity_kind_declares_aspect(&self, kind_id: KindId, aspect: &AspectKey) -> bool {
        self.selected_root()
            .schema_authority()
            .entity_aspect_plan(kind_id)
            .and_then(|plan| plan.contract_for(aspect))
            .is_some()
    }

    /// Whether the schema retained with this observation declares `aspect`
    /// for relation kind `kind_id`. A later schema change never alters the
    /// answer.
    pub fn relation_kind_declares_aspect(&self, kind_id: KindId, aspect: &AspectKey) -> bool {
        self.selected_root()
            .schema_authority()
            .relation_aspect_plan(kind_id)
            .and_then(|plan| plan.contract_for(aspect))
            .is_some()
    }
}
