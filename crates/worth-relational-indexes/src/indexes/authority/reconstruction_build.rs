//! Prepare and publish reconstructed index generations on the original payer.
use super::*;
use crate::contracts::authority::commit::preparation::planning::strategy::strategy_for_parallel_packets;
use crate::contracts::execution::request_work::RequestWorkBudget;
use crate::contracts::indexes::data::definition::DerivedIndexDefinition;
use crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane;
use crate::contracts::transactions::data::preparation_strategy::PreparationStrategy;
use crate::runtime::state::subsystems::indexing::IndexingSubsystem;
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest, ExecutionResourceLease};

enum Packets {
    Checked(
        super::packet_planning::PlannedIndexPackets,
        RequestWorkBudget,
    ),
    Unleased(Vec<DerivedIndexDefinition>),
}

pub struct IndexBuildPlan {
    packets: Packets,
    present: Vec<DerivedIndexId>,
    missing: Vec<DerivedIndexId>,
}

impl IndexBuildPlan {
    pub fn prepare(
        indexes: &IndexingSubsystem,
        request: &DerivedIndexBuildRequest,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, DerivedIndexBuildOutcome> {
        let (packets, present, missing) = if let Some(lease) = lease(execution) {
            let work = RequestWorkBudget::new();
            let mut planned = plan_index_packets_checked(indexes, &request.index_ids, lease, &work)
                .map_err(|stop| DerivedIndexBuildOutcome {
                    source_commit_id: request.source_commit_id,
                    generations: Vec::new(),
                    failed_indexes: request.index_ids.clone(),
                    basis_denial: None,
                    execution_denial: Some(index_execution_denial(stop)),
                })?;
            let present = std::mem::take(&mut planned.present);
            let missing = std::mem::take(&mut planned.missing);
            (Packets::Checked(planned, work), present, missing)
        } else {
            let (definitions, missing) = planned_index_definitions(indexes, &request.index_ids);
            let present = definitions
                .iter()
                .map(|definition| definition.index_id)
                .collect();
            (Packets::Unleased(definitions), present, missing)
        };
        Ok(Self {
            packets,
            present,
            missing,
        })
    }

    pub fn strategy(&self, execution: &ExecutionRequest<'_, '_>) -> (usize, PreparationStrategy) {
        let count = self.present.len();
        (
            count,
            strategy_for_parallel_packets(lease(execution), count),
        )
    }

    pub fn unavailable(
        mut self,
        request: &DerivedIndexBuildRequest,
        denial: Option<crate::contracts::branch::basis_denial::RelationalBranchBasisDenial>,
    ) -> DerivedIndexBuildOutcome {
        self.missing.extend(self.present);
        failed_build_outcome(request.source_commit_id, Vec::new(), self.missing, denial)
    }

    pub fn reconstruct(
        mut self,
        indexes: &IndexingSubsystem,
        execution: &ExecutionRequest<'_, '_>,
        lane: &ReconstructionLane<'_>,
        request: &DerivedIndexBuildRequest,
        selected: (
            crate::contracts::history::data::coordinates::BranchId,
            crate::contracts::identity::data::primitives::VersionId,
        ),
        projection: &IndexProjectionSource<'_>,
    ) -> (DerivedIndexBuildOutcome, bool) {
        let Some(schema_version) = projection.schema_version() else {
            return (self.unavailable(request, None), false);
        };
        let results = match self.packets {
            Packets::Checked(planned, work) => execute_index_inputs(
                lane,
                execution,
                projection,
                planned.inputs,
                lease(execution),
                Some(&work),
            ),
            Packets::Unleased(definitions) => execute_index_packets(
                lane,
                execution,
                projection,
                plan_index_packets(definitions),
                None,
                None,
            ),
        };
        let results = match results {
            Ok(results) => results,
            Err(execution_denial) => {
                self.missing.extend(self.present);
                return (
                    DerivedIndexBuildOutcome {
                        source_commit_id: request.source_commit_id,
                        generations: Vec::new(),
                        failed_indexes: self.missing,
                        basis_denial: None,
                        execution_denial: Some(execution_denial),
                    },
                    false,
                );
            }
        };
        if let (Some(digest), Some(plans)) = (
            projection.schema_authority_digest(),
            projection.aspect_plans(),
        ) {
            indexes.install_routing(digest, plans);
        }
        let basis =
            IndexGenerationPublicationBasis::new(request, selected.0, selected.1, schema_version);
        let generations = publish_index_generations(indexes, &basis, results, &mut self.missing);
        (
            DerivedIndexBuildOutcome {
                source_commit_id: request.source_commit_id,
                generations,
                failed_indexes: self.missing,
                basis_denial: None,
                execution_denial: None,
            },
            true,
        )
    }
}

fn lease<'request, 'authority>(
    execution: &ExecutionRequest<'request, 'authority>,
) -> Option<&'request ExecutionResourceLease<'authority>> {
    match execution.allocation_policy() {
        ExecutionAllocationPolicy::Execution(lease) => Some(lease),
        ExecutionAllocationPolicy::SystemAllocation => None,
    }
}
