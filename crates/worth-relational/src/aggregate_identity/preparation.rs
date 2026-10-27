//! Request-funded preparation of deterministic, atomic aggregate consequences.
use worth_execution::{ChargedBytes, ExecutionMemoryReservation};

use crate::branch::{RelationalBranchRootSchemaAuthority, SelectedRelationalBranchState};
use crate::history::data::CommitId;
use crate::identity::data::KindId;
use crate::identity::data::VersionId;
use crate::storage::overlay::{OverlayStateView, WorkingState};
use crate::transactions::data::{AuthoritativeApplyPlan, MergedCommitPlan, TransactionCommitError};
use crate::validation::engine::state_view::InvariantStateView;

use super::changes::AggregateChanges;
pub(super) use super::funding::{AggregateWork, BuildResult, PlanDenial};
use super::projection::{AggregateProjection, DefinitionProjection, ProjectionData};
use super::{cohorts, consequence_plan};
use std::sync::Arc;

pub(crate) struct AggregateAdmission {
    pub(crate) plan: MergedCommitPlan,
    _retention: Option<Arc<ExecutionMemoryReservation>>,
    pub(super) storage_retention: Option<ExecutionMemoryReservation>,
}
pub(crate) struct PreparedConsequences {
    pub(crate) apply: AuthoritativeApplyPlan,
    pub(crate) demands: consequence_plan::RecordDemands,
    pub(crate) admission: AggregateAdmission,
}
pub(super) struct PreparedAggregateArtifact {
    pub(super) projection: Arc<AggregateProjection>,
    pub(super) consequences: Option<PreparedConsequences>,
}
struct PreparedValue {
    apply: AuthoritativeApplyPlan,
    demands: consequence_plan::RecordDemands,
    admission: MergedCommitPlan,
    projection: ProjectionData,
    source_bytes: u64,
    controls: u64,
}
impl ChargedBytes for PreparedValue {
    fn additional_charged_bytes(&self) -> u64 {
        let backing = (self.apply.merged_intents.capacity()
            + self.admission.merged_intents.capacity()) as u64
            * std::mem::size_of::<crate::transactions::data::MutationIntent>() as u64;
        self.apply
            .merged_intents
            .iter()
            .chain(&self.admission.merged_intents)
            .fold(
                self.demands
                    .owned_bytes()
                    .saturating_add(backing)
                    .saturating_add(self.projection.owned_bytes())
                    .saturating_add(self.source_bytes)
                    .saturating_add(self.controls),
                |bytes, intent| bytes.saturating_add(intent.owned_allocation_capacity_bytes()),
            )
    }
}

pub(super) fn prepare(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    budget: Option<&crate::execution::RequestWorkBudget>,
    authority: &RelationalBranchRootSchemaAuthority,
    selected: &SelectedRelationalBranchState,
    working: &WorkingState,
    business: &MergedCommitPlan,
    effect: &AggregateChanges,
    version: VersionId,
    occurrence: CommitId,
) -> Result<PreparedAggregateArtifact, TransactionCommitError> {
    let before = InvariantStateView::new(selected, selected.version_id());
    let overlay = OverlayStateView::new(selected, working);
    let after = InvariantStateView::new(&overlay, version);
    let (value, retention) = super::funding::run(execution, budget, |work| {
        let mut plan = consequence_plan::ConsequencePlan::default();
        let mut projection = ProjectionData::default();
        let source_bytes = effect.retained_bytes();
        let controls = super::retention_extent::arc_backing::<AggregateProjection>()
            + super::retention_extent::arc_backing::<ExecutionMemoryReservation>();
        work.retain(source_bytes.saturating_add(controls))?;
        for contract in authority
            .registry()
            .relation_kinds
            .values()
            .filter_map(|kind| kind.relation_integrity.aggregate_identity)
        {
            work.checkpoint(1)?;
            work.retain(super::retention_extent::tree_entry::<
                KindId,
                DefinitionProjection,
            >())?;
            let changed = match cohorts::collect(contract, &before, &after, effect, work)? {
                Some(cohorts) => Some(consequence_plan::append(
                    &mut plan, contract, occurrence, cohorts, &after, work,
                )?),
                None => None,
            };
            projection.definitions.insert(
                contract.source_relation,
                DefinitionProjection { contract, changed },
            );
        }
        if plan.intents.is_empty() {
            return Ok(PreparedValue {
                apply: AuthoritativeApplyPlan {
                    transaction_id: business.transaction_id,
                    version_id: version,
                    merged_intents: Vec::new(),
                },
                demands: plan.demands,
                projection,
                source_bytes,
                controls,
                admission: MergedCommitPlan {
                    transaction_id: business.transaction_id,
                    merged_intents: Vec::new(),
                },
            });
        }
        let count = business
            .merged_intents
            .len()
            .checked_add(plan.intents.len())
            .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
        work.retain(
            (count as u64)
                .checked_mul(
                    std::mem::size_of::<crate::transactions::data::MutationIntent>() as u64,
                )
                .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?,
        )?;
        let mut admission = MergedCommitPlan {
            transaction_id: business.transaction_id,
            merged_intents: Vec::new(),
        };
        admission
            .merged_intents
            .try_reserve_exact(count)
            .map_err(|_| worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
        for intent in &business.merged_intents {
            let bytes = intent.owned_allocation_capacity_bytes();
            work.checkpoint(bytes.saturating_add(1))?;
            work.retain(bytes)?;
            admission.merged_intents.push(intent.clone());
        }
        for intent in &plan.intents {
            work.retain(intent.owned_allocation_capacity_bytes())?;
            admission.merged_intents.push(intent.clone());
        }
        Ok(PreparedValue {
            apply: AuthoritativeApplyPlan {
                transaction_id: business.transaction_id,
                version_id: version,
                merged_intents: plan.intents,
            },
            demands: plan.demands,
            admission,
            projection,
            source_bytes,
            controls,
        })
    })?;
    let retention = retention.map(Arc::new);
    let projection = Arc::new(AggregateProjection {
        data: value.projection,
        _retention: retention.clone(),
    });
    let consequences = (!value.apply.merged_intents.is_empty()).then(|| PreparedConsequences {
        apply: value.apply,
        demands: value.demands,
        admission: AggregateAdmission {
            plan: value.admission,
            _retention: retention,
            storage_retention: None,
        },
    });
    Ok(PreparedAggregateArtifact {
        projection,
        consequences,
    })
}
