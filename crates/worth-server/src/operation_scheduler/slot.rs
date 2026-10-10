use worth_query::facade::runtime::{WorthQueryRuntimeError, WorthQuerySharedReadContext};

use crate::{
    WorthServerLoweredOperationPlan, WorthServerOperationConcurrencyClass,
    WorthServerOperationExecutionStrategy, WorthServerOperationPlanProof,
    WorthServerOperationPreconditionPosture, WorthServerQueryHandoff,
    WorthServerQueryHandoffOperation,
};

use super::{WorthServerSchedulerConflictDenial, WorthServerSchedulerLane};

#[derive(Debug)]
pub struct WorthServerOperationExecutionSlot {
    ordinal: usize,
    scheduler_lane: WorthServerSchedulerLane,
    dependency_group: Option<String>,
    plan_proof: WorthServerOperationPlanProof,
    workspace_name: String,
    operation_label: String,
    precondition_posture: WorthServerOperationPreconditionPosture,
    handoff: Option<WorthServerQueryHandoff>,
}

impl WorthServerOperationExecutionSlot {
    pub(crate) fn from_lowered_plan(
        ordinal: usize,
        plan: WorthServerLoweredOperationPlan,
    ) -> Result<Self, WorthServerSchedulerConflictDenial> {
        let scheduler_lane = WorthServerSchedulerLane::from_lowered_plan(&plan)?;
        validate_plan_for_scheduler(&plan, &scheduler_lane)?;
        let dependency_group = dependency_group(&plan, &scheduler_lane);
        let precondition_posture = plan.query_handoff().precondition_posture().clone();
        let plan_proof = plan.proof();
        let workspace_name = plan.query_handoff().workspace().name().to_string();
        let operation_label = plan.query_handoff().operation().canonical_label();
        let handoff = plan.into_query_handoff();
        Ok(Self {
            ordinal,
            scheduler_lane,
            dependency_group,
            plan_proof,
            workspace_name,
            operation_label,
            precondition_posture,
            handoff: Some(handoff),
        })
    }

    pub fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub fn scheduler_lane(&self) -> String {
        self.scheduler_lane.canonical_label()
    }

    pub(crate) fn scheduler_lane_key(&self) -> String {
        self.scheduler_lane.lane_scope_key()
    }

    pub(crate) fn scheduler_lane_kind(&self) -> &WorthServerSchedulerLane {
        &self.scheduler_lane
    }

    pub fn dependency_group(&self) -> Option<&str> {
        self.dependency_group.as_deref()
    }

    pub fn plan_proof(&self) -> &WorthServerOperationPlanProof {
        &self.plan_proof
    }

    pub fn workspace_name(&self) -> &str {
        &self.workspace_name
    }

    pub fn operation_label(&self) -> &str {
        &self.operation_label
    }

    pub fn precondition_posture(&self) -> &WorthServerOperationPreconditionPosture {
        &self.precondition_posture
    }

    pub(crate) fn slot_basis_digest(&self) -> Option<&str> {
        self.handoff()
            .operation_admission()
            .operation_request()
            .identity()
            .basis_digest()
    }

    pub(crate) fn mint_shared_read_context(
        &self,
    ) -> Result<WorthQuerySharedReadContext, WorthQueryRuntimeError> {
        self.handoff()
            .workspace()
            .with_execution_request(|execution| {
                self.handoff().workspace().shared_read_context(execution)
            })?
    }

    pub(crate) fn shared_read_hot_path_lock_count(&self) -> usize {
        self.handoff()
            .workspace()
            .shared_read_counters()
            .committed_read_hot_path_lock_count()
    }

    pub(crate) fn record_shared_read_hot_path_lock_for_certification(&self) {
        self.handoff()
            .workspace()
            .record_shared_read_hot_path_lock_for_certification();
    }

    pub(crate) fn handoff(&self) -> &WorthServerQueryHandoff {
        self.handoff
            .as_ref()
            .expect("scheduled slot should retain handoff until materialization")
    }

    pub(crate) fn handoff_mut(&mut self) -> &mut WorthServerQueryHandoff {
        self.handoff
            .as_mut()
            .expect("scheduled slot should retain handoff until materialization")
    }

    pub(crate) fn take_handoff(&mut self) -> WorthServerQueryHandoff {
        self.handoff
            .take()
            .expect("scheduled slot handoff should only be consumed once")
    }
}

fn validate_plan_for_scheduler(
    plan: &WorthServerLoweredOperationPlan,
    scheduler_lane: &WorthServerSchedulerLane,
) -> Result<(), WorthServerSchedulerConflictDenial> {
    match scheduler_lane {
        WorthServerSchedulerLane::SharedRead => validate_shared_read_plan(plan),
        WorthServerSchedulerLane::DeterministicSubmission { .. }
        | WorthServerSchedulerLane::ProductDraftMutation { .. }
        | WorthServerSchedulerLane::DurableProductMutation { .. }
        | WorthServerSchedulerLane::ProductSessionCoordination { .. } => {
            validate_ordered_plan(plan, scheduler_lane)
        }
    }
}

fn validate_shared_read_plan(
    plan: &WorthServerLoweredOperationPlan,
) -> Result<(), WorthServerSchedulerConflictDenial> {
    if plan.strategy() != WorthServerOperationExecutionStrategy::SharedReadExecution {
        return Err(WorthServerSchedulerConflictDenial::non_shared_read_plan(
            format!(
                "shared-read scheduler only admits lowered plans using `shared-read-execution`, got `{}`",
                plan.strategy().as_str()
            ),
        ));
    }
    if plan.receipt().expected_scheduler_lane() != "shared-read" {
        return Err(WorthServerSchedulerConflictDenial::non_shared_read_plan(
            format!(
                "shared-read scheduler only admits lowered plans on the `shared-read` lane, got `{}`",
                plan.receipt().expected_scheduler_lane()
            ),
        ));
    }
    if plan.query_handoff().concurrency_class()
        != WorthServerOperationConcurrencyClass::ConcurrentSharedRead
    {
        return Err(WorthServerSchedulerConflictDenial::non_shared_read_plan(
            "shared-read scheduler requires `ConcurrentSharedRead` concurrency classification",
        ));
    }
    match plan.query_handoff().operation() {
        WorthServerQueryHandoffOperation::QueryRead { .. }
        | WorthServerQueryHandoffOperation::DirectRead { .. }
        | WorthServerQueryHandoffOperation::DirectState { .. }
        | WorthServerQueryHandoffOperation::DirectInspection { .. } => Ok(()),
        unsupported => Err(
            WorthServerSchedulerConflictDenial::unsupported_shared_read_operation(format!(
                "shared-read scheduler cannot execute `{unsupported:?}` from lowered-plan-only state"
            )),
        ),
    }
}

fn validate_ordered_plan(
    plan: &WorthServerLoweredOperationPlan,
    scheduler_lane: &WorthServerSchedulerLane,
) -> Result<(), WorthServerSchedulerConflictDenial> {
    if plan.query_handoff().concurrency_class()
        != WorthServerOperationConcurrencyClass::SerializeDeterministically
    {
        return Err(WorthServerSchedulerConflictDenial::unsupported_ordered_operation(
            "ordered scheduler requires `SerializeDeterministically` concurrency classification",
        ));
    }
    match (plan.strategy(), plan.query_handoff().operation(), scheduler_lane) {
        (
            WorthServerOperationExecutionStrategy::DeterministicSubmission,
            WorthServerQueryHandoffOperation::DirectMutation {
                scheduled_operation: Some(_),
                ..
            }
            | WorthServerQueryHandoffOperation::QueryMutation {
                scheduled_operation: Some(_),
                ..
            },
            WorthServerSchedulerLane::DeterministicSubmission { .. },
        ) => Ok(()),
        (
            WorthServerOperationExecutionStrategy::ProductAdapterExecution,
            WorthServerQueryHandoffOperation::DirectMutation {
                scheduled_operation: Some(_),
                ..
            }
            | WorthServerQueryHandoffOperation::QueryMutation {
                scheduled_operation: Some(_),
                ..
            },
            WorthServerSchedulerLane::ProductDraftMutation { .. },
        ) => Ok(()),
        (
            WorthServerOperationExecutionStrategy::DurableProductMutationExecution,
            WorthServerQueryHandoffOperation::DirectMutation {
                scheduled_operation: Some(_),
                ..
            }
            | WorthServerQueryHandoffOperation::QueryMutation {
                scheduled_operation: Some(_),
                ..
            },
            WorthServerSchedulerLane::DurableProductMutation { .. },
        ) => Ok(()),
        (
            WorthServerOperationExecutionStrategy::SessionCoordination,
            WorthServerQueryHandoffOperation::DirectMutation {
                scheduled_operation: Some(_),
                ..
            }
            | WorthServerQueryHandoffOperation::QueryMutation {
                scheduled_operation: Some(_),
                ..
            },
            WorthServerSchedulerLane::ProductSessionCoordination { .. },
        ) => Ok(()),
        (
            _,
            WorthServerQueryHandoffOperation::DirectMutation {
                scheduled_operation: None,
                ..
            }
            | WorthServerQueryHandoffOperation::QueryMutation {
                scheduled_operation: None,
                ..
            },
            _,
        ) => Err(WorthServerSchedulerConflictDenial::unsupported_ordered_operation(
            "ordered mutation scheduling requires a scheduled query mutation payload in the lowered plan",
        )),
        (_, unsupported, _) => Err(
            WorthServerSchedulerConflictDenial::unsupported_ordered_operation(format!(
                "ordered scheduler cannot execute unsupported operation `{unsupported:?}` from lowered-plan-only state"
            )),
        ),
    }
}

fn dependency_group(
    plan: &WorthServerLoweredOperationPlan,
    scheduler_lane: &WorthServerSchedulerLane,
) -> Option<String> {
    if *scheduler_lane != WorthServerSchedulerLane::SharedRead {
        return None;
    }
    let operation_request = plan
        .query_handoff()
        .operation_admission()
        .operation_request();
    let basis_digest = operation_request
        .identity()
        .basis_digest()
        .unwrap_or("none");
    Some(format!(
        "{}|basis={basis_digest}",
        plan.query_handoff()
            .operation_admission()
            .authority_footprint()
            .canonical_digest()
    ))
}
