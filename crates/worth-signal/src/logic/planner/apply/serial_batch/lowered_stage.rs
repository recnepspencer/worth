use crate::data::aspect::AspectMask;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::node::AuthorityPolicy;
use crate::data::output::CanonicalChangedRegions;
use crate::data::performance::ResolvedMaintenanceStrategy;
use crate::data::proof::{DedupedNodeBatch, DirtyDelta, PartitionScopeSet, SortedSourceBatch};
use crate::logic::planner::semantic::StageSemanticIdentity;
use crate::logic::planner::types::EligibleTask;

use super::preparation::SerialFinalizeSeed;
use super::task_lowering::{lower_serial_task_patch, LoweredSerialTask};
use super::witness::{ExactStageWidth, StageTaskOrderProof};
use crate::logic::planner::precompute::PreparedTaskPatch;
use crate::logic::planner::types::ApplyPlanSerialFallbackReason;

#[derive(Debug, Clone)]
pub(in crate::logic::planner) struct LoweredSerialStage {
    pub(super) stage_index: u32,
    pub(super) stage_tasks: Vec<EligibleTask>,
    pub(super) authority_policy: AuthorityPolicy,
    pub(super) dirty_delta: crate::data::proof::StructuralDelta,
    pub(super) maintenance_strategy: ResolvedMaintenanceStrategy,
    pub(super) serial_rejection_reason: Option<ApplyPlanSerialFallbackReason>,
    pub(super) lowered_tasks: Vec<LoweredSerialTask>,
    pub(super) finalize_seeds: Vec<SerialFinalizeSeed>,
    pub(super) stage_order: StageTaskOrderProof,
    pub(super) exact_width: ExactStageWidth,
}

impl LoweredSerialStage {
    pub(in crate::logic::planner) fn from_prepared_patches(
        graph: &mut SignalGraph,
        stage_index: u32,
        stage_tasks: &[EligibleTask],
        patches: Vec<PreparedTaskPatch>,
        maintenance_strategy: ResolvedMaintenanceStrategy,
        default_authority_policy: AuthorityPolicy,
        stage_identities: &[StageSemanticIdentity],
    ) -> Result<Self, SignalError> {
        let mut lowered_tasks = Vec::with_capacity(patches.len());
        let mut finalize_seeds = Vec::with_capacity(patches.len());
        let mut changed_aspects = AspectMask::EMPTY;
        let mut changed_regions = Vec::new();
        let mut touched_nodes = Vec::with_capacity(patches.len());
        let mut touched_sources = Vec::new();
        let mut touched_scopes = Vec::new();
        let mut authority_policy = default_authority_policy;

        for patch in patches {
            let material = lower_serial_task_patch(graph, patch, stage_identities)?;
            changed_aspects = changed_aspects | material.produced_aspects;
            changed_regions.extend(material.changed_regions);
            touched_nodes.push(material.task.node);
            touched_sources.extend(material.touched_sources);
            touched_scopes.extend(material.touched_scopes);
            if matches!(
                material.authority_policy,
                AuthorityPolicy::AuthoritativeOnly
            ) {
                authority_policy = AuthorityPolicy::AuthoritativeOnly;
            }
            lowered_tasks.push(material.task);
            finalize_seeds.push(material.finalize_seed);
        }

        let dirty_delta = DirtyDelta::new(
            changed_aspects,
            CanonicalChangedRegions::new(changed_regions),
            DedupedNodeBatch::new(touched_nodes.clone()),
        );
        let touched_scope = crate::data::proof::TouchedScopeSummary::new(
            PartitionScopeSet::new(touched_scopes),
            touched_nodes,
            SortedSourceBatch::new(touched_sources),
        );

        Ok(Self {
            stage_index,
            stage_tasks: stage_tasks.to_vec(),
            authority_policy,
            dirty_delta: crate::data::proof::StructuralDelta::new(
                Some(dirty_delta),
                Some(touched_scope),
            ),
            maintenance_strategy,
            serial_rejection_reason: None,
            exact_width: ExactStageWidth::new(lowered_tasks.len()),
            lowered_tasks,
            finalize_seeds,
            stage_order: StageTaskOrderProof::established(),
        })
    }

    pub(in crate::logic::planner) fn authority_policy(&self) -> AuthorityPolicy {
        self.authority_policy
    }

    pub(in crate::logic::planner) fn dirty_delta(&self) -> &crate::data::proof::StructuralDelta {
        &self.dirty_delta
    }

    pub(in crate::logic::planner) fn stage_width(&self) -> usize {
        self.exact_width.get()
    }

    pub(in crate::logic::planner) fn maintenance_strategy(&self) -> ResolvedMaintenanceStrategy {
        self.maintenance_strategy
    }

    pub(in crate::logic::planner) fn serial_rejection_reason(
        &self,
    ) -> Option<ApplyPlanSerialFallbackReason> {
        self.serial_rejection_reason
    }
}
