//! Native-owned pre-intent target cold-reconstruction budget admission for
//! Home program transitions.

use super::*;

impl RelationalRuntime {
    /// Admit `budget` against the exact successor cold-reconstruction
    /// footprint (entity/relation/region slots, live-bitset record counts
    /// including `RetainedDanglingForAudit` relations, prepared target index
    /// definitions, and per-index publication work) before publishing any
    /// durable transition intent.
    #[allow(clippy::too_many_arguments)]
    pub fn preflight_home_program_target_index_budget(
        &self,
        request: &ExecutionRequest<'_, '_>,
        predecessor: &AdmittedRelationalBranchBasis,
        target: &PreparedRelationalHomeProgramContracts,
        created_entities: usize,
        created_relations: usize,
        touched_partitions: &[crate::identity::data::PartitionId],
        budget: crate::indexes::data::DerivedIndexMaintenanceBudget,
    ) -> Result<
        crate::indexes::data::DerivedIndexMaintenanceWork,
        crate::indexes::data::DerivedIndexMaintenanceDenial,
    > {
        use crate::indexes::data::{
            DerivedIndexMaintenanceDenial, DerivedIndexMaintenanceDenialKind,
            DerivedIndexMaintenanceWork,
        };
        if target.meaning.runtime != self.runtime_instance_id()
            || predecessor.identity().runtime_instance_id() != self.runtime_instance_id()
        {
            return Err(DerivedIndexMaintenanceDenial {
                kind: DerivedIndexMaintenanceDenialKind::ForeignCandidate,
                work: DerivedIndexMaintenanceWork::default(),
            });
        }
        worth_relational_indexes::internal_port::indexes::authority::maintenance::cold_preflight::admit_home_target_cold_reconstruction_budget(
            request,
            &predecessor.owner_inner().root,
            target.index_definitions.as_deref(),
            created_entities,
            created_relations,
            touched_partitions,
            budget,
        )
    }
}
