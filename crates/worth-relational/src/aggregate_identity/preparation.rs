//! Request-funded preparation of deterministic, atomic aggregate consequences.
use worth_execution::{ChargedBytes, ExecutionMemoryReservation};

use crate::authority::mutation::MutationEffect;
use crate::branch::{RelationalBranchRootSchemaAuthority, SelectedRelationalBranchState};
use crate::history::data::CommitId;
use crate::identity::data::VersionId;
use crate::storage::overlay::{OverlayStateView, WorkingState};
use crate::transactions::data::{AuthoritativeApplyPlan, MergedCommitPlan, TransactionCommitError};
use crate::validation::engine::state_view::InvariantStateView;

pub(super) use super::funding::{AggregateWork, BuildResult, PlanDenial};
use super::{cohorts, consequence_plan};

pub(crate) struct AggregateAdmission {
    pub(crate) plan: MergedCommitPlan,
    _retention: Option<ExecutionMemoryReservation>,
    pub(super) storage_retention: Option<ExecutionMemoryReservation>,
}
pub(crate) struct PreparedConsequences {
    pub(crate) apply: AuthoritativeApplyPlan,
    pub(crate) demands: consequence_plan::RecordDemands,
    pub(crate) admission: AggregateAdmission,
}
struct PreparedValue {
    apply: AuthoritativeApplyPlan,
    demands: consequence_plan::RecordDemands,
    admission: MergedCommitPlan,
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
                self.demands.owned_bytes().saturating_add(backing),
                |bytes, intent| bytes.saturating_add(intent.owned_allocation_capacity_bytes()),
            )
    }
}

pub(crate) fn prepare(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    authority: &RelationalBranchRootSchemaAuthority,
    selected: &SelectedRelationalBranchState,
    working: &WorkingState,
    business: &MergedCommitPlan,
    effect: &MutationEffect,
    version: VersionId,
    occurrence: CommitId,
) -> Result<Option<PreparedConsequences>, TransactionCommitError> {
    if !authority
        .registry()
        .relation_kinds
        .values()
        .any(|kind| kind.relation_integrity.aggregate_identity.is_some())
    {
        return Ok(None);
    }
    let before = InvariantStateView::new(selected, selected.version_id());
    let overlay = OverlayStateView::new(selected, working);
    let after = InvariantStateView::new(&overlay, version);
    let (value, retention) =
        super::funding::run(execution, runtime.commit_work_budget.as_ref(), |work| {
            let mut plan = consequence_plan::ConsequencePlan::default();
            for contract in authority
                .registry()
                .relation_kinds
                .values()
                .filter_map(|kind| kind.relation_integrity.aggregate_identity)
            {
                work.checkpoint(1)?;
                let Some(cohorts) = cohorts::collect(contract, &before, &after, effect, work)?
                else {
                    continue;
                };
                consequence_plan::append(&mut plan, contract, occurrence, cohorts, &after, work)?;
            }
            if plan.intents.is_empty() {
                return Ok(PreparedValue {
                    apply: AuthoritativeApplyPlan {
                        transaction_id: business.transaction_id,
                        version_id: version,
                        merged_intents: Vec::new(),
                    },
                    demands: plan.demands,
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
            })
        })?;
    if value.apply.merged_intents.is_empty() {
        return Ok(None);
    }
    Ok(Some(PreparedConsequences {
        apply: value.apply,
        demands: value.demands,
        admission: AggregateAdmission {
            plan: value.admission,
            _retention: retention,
            storage_retention: None,
        },
    }))
}
