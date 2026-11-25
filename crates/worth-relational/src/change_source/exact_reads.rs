use super::RelationalObservationReadDenial;
use crate::identity::data::{EntityId, RelationId};
use crate::mvcc::RelationalBranchObservation;
use crate::runtime::RelationalRuntime;
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use worth_relational_branch::internal_port::mvcc::observation::RelationalBranchObservationOwnerPort as _;

impl RelationalRuntime {
    /// Read one entity exactly as `observation`'s retained root holds it.
    ///
    /// Returns `None` when the root has no such entity, or holds a different
    /// generation of its slot. Later commits never change the answer.
    ///
    /// # Errors
    ///
    /// [`RelationalObservationReadDenial::ForeignObservation`] when another
    /// runtime issued `observation`; [`RelationalObservationReadDenial::Execution`]
    /// when the request or required resident storage refuses the read.
    pub fn entity_record_at_observation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &RelationalBranchObservation,
        entity_id: EntityId,
    ) -> Result<Option<EntityReadRecord>, RelationalObservationReadDenial> {
        self.admit_observation_read(observation)?;
        crate::execution::check_request(execution)
            .map_err(RelationalObservationReadDenial::Execution)?;
        let root = observation.selected_root().read();
        root.demand_regions_full(execution, &self.services.symbols, [entity_id.partition_id])
            .map_err(|failure| RelationalObservationReadDenial::Execution(failure.into()))?;
        Ok(self
            .read_truth()
            .authoritative_entity_record_for_id_from_exact_state(
                &root,
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
    /// runtime issued `observation`; [`RelationalObservationReadDenial::Execution`]
    /// when the request or required resident storage refuses the read.
    pub fn relation_record_at_observation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &RelationalBranchObservation,
        relation_id: RelationId,
    ) -> Result<Option<RelationReadRecord>, RelationalObservationReadDenial> {
        self.admit_observation_read(observation)?;
        crate::execution::check_request(execution)
            .map_err(RelationalObservationReadDenial::Execution)?;
        let root = observation.selected_root().read();
        root.demand_regions_full(
            execution,
            &self.services.symbols,
            [relation_id.partition_id],
        )
        .map_err(|failure| RelationalObservationReadDenial::Execution(failure.into()))?;
        Ok(self
            .read_truth()
            .authoritative_relation_record_for_id_from_exact_state(
                &root,
                root.schema_authority().registry(),
                relation_id,
            ))
    }
}
