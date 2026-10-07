use std::collections::{hash_map::Entry, BTreeMap, HashMap};

use worth_query::facade::runtime::{WorthQueryHandleDenial, WorthQueryRuntimeError};

use crate::{WorthServerQueryOperation, WorthServerResponseFacade, WorthServerResponseInput};

use super::{
    WorthServerExecutedOperationBatch, WorthServerOperationExecutionSlot,
    WorthServerOperationSchedulerCounters, WorthServerScheduledMutationResult,
    WorthServerScheduledOperationOutcome, WorthServerSchedulerCancellationDirective,
    WorthServerSchedulerCancellationPosture, WorthServerSchedulerFailurePosture,
    WorthServerSchedulerLane, WorthServerSchedulerRuntimeFailure,
};

pub(crate) fn execute_slots(
    response_facade: WorthServerResponseFacade,
    slots: Vec<WorthServerOperationExecutionSlot>,
    directives: impl IntoIterator<Item = WorthServerSchedulerCancellationDirective>,
) -> WorthServerExecutedOperationBatch {
    let directives = CancellationPlan::new(directives);
    let prepared = PreparedGroups::new(response_facade, slots, directives);
    let mut counters = WorthServerOperationSchedulerCounters::default();
    counters.set_planned_batch_width(prepared.planned_batch_width);
    counters.absorb(&prepared.early_counters);

    let mut outcomes = prepared.early_outcomes;
    for group in prepared.groups.into_values() {
        let execution = execute_group(group);
        counters.absorb(&execution.counters);
        outcomes.extend(execution.outcomes);
    }

    outcomes.sort_by_key(WorthServerScheduledOperationOutcome::ordinal);
    let counters_snapshot = counters.clone();
    for outcome in &mut outcomes {
        outcome.attach_scheduler_counters(counters_snapshot.clone());
    }
    WorthServerExecutedOperationBatch::new(outcomes, counters)
}

fn execute_group(group: OrderedLaneExecution) -> GroupExecution {
    let mut outcomes = Vec::with_capacity(group.executions.len());
    let mut counters = WorthServerOperationSchedulerCounters::default();
    let mut failed_slot_ordinal = None;
    let mut latest_lane_basis_digest = None::<String>;

    for mut execution in group.executions {
        if let Some(failed_slot_ordinal) = failed_slot_ordinal {
            counters.increment_queue_closed_slot_count();
            outcomes.push(
                WorthServerScheduledOperationOutcome::failed_without_counters(
                    execution.slot,
                    WorthServerSchedulerFailurePosture::OrderedLaneClosed {
                        scheduler_lane: group.lane_key.clone(),
                        failed_slot_ordinal,
                    },
                ),
            );
            continue;
        }

        let stale_basis = match stale_basis(&execution.slot, latest_lane_basis_digest.as_deref()) {
            Ok(stale_basis) => stale_basis,
            Err(denial) => {
                counters.increment_isolated_failure_count();
                failed_slot_ordinal = Some(execution.slot.ordinal());
                outcomes.push(
                    WorthServerScheduledOperationOutcome::failed_without_counters(
                        execution.slot,
                        WorthServerSchedulerFailurePosture::IsolatedRuntimeFailure {
                            runtime_failure:
                                WorthServerSchedulerRuntimeFailure::from_mutation_runtime_error(
                                    denial.into(),
                                ),
                        },
                    ),
                );
                continue;
            }
        };
        if let Some(StaleBasis {
            expected_basis_digest,
            observed_basis_digest,
        }) = stale_basis
        {
            counters.increment_stale_basis_stop_count();
            failed_slot_ordinal = Some(execution.slot.ordinal());
            outcomes.push(
                WorthServerScheduledOperationOutcome::failed_without_counters(
                    execution.slot,
                    WorthServerSchedulerFailurePosture::StaleMutationBasis {
                        expected_basis_digest,
                        observed_basis_digest,
                    },
                ),
            );
            continue;
        }

        increment_admission_counter(&mut counters, execution.slot.scheduler_lane_kind());
        if execution.cancel_after_admission_before_execution {
            counters.increment_cancelled_after_admission_before_execution_count();
            outcomes.push(
                WorthServerScheduledOperationOutcome::cancelled_without_counters(
                    execution.slot,
                    WorthServerSchedulerCancellationPosture::AfterAdmissionBeforeExecution,
                ),
            );
            continue;
        }

        let execution_result = execute_slot_operation(&mut execution.slot);
        match execution_result {
            Ok(mutation_result) => {
                if execution.cancel_during_execution {
                    counters.increment_cancelled_during_execution_count();
                    outcomes.push(
                        WorthServerScheduledOperationOutcome::cancelled_without_counters(
                            execution.slot,
                            WorthServerSchedulerCancellationPosture::DuringExecution,
                        ),
                    );
                    continue;
                }

                increment_completion_counter(&mut counters, execution.slot.scheduler_lane_kind());
                let response_envelope = group.response_facade.shape_with_defaults(
                    WorthServerResponseInput::query_handoff_success(execution.slot.take_handoff()),
                );
                latest_lane_basis_digest = Some(mutation_result.snapshot_basis_digest());
                outcomes.push(
                    WorthServerScheduledOperationOutcome::mutation_success_without_counters(
                        execution.slot,
                        mutation_result,
                        response_envelope,
                    ),
                );
            }
            Err(error) => {
                counters.increment_isolated_failure_count();
                failed_slot_ordinal = Some(execution.slot.ordinal());
                outcomes.push(
                    WorthServerScheduledOperationOutcome::failed_without_counters(
                        execution.slot,
                        WorthServerSchedulerFailurePosture::IsolatedRuntimeFailure {
                            runtime_failure:
                                WorthServerSchedulerRuntimeFailure::from_mutation_runtime_error(
                                    error,
                                ),
                        },
                    ),
                );
            }
        }
    }

    GroupExecution { outcomes, counters }
}

fn execute_slot_operation(
    slot: &mut WorthServerOperationExecutionSlot,
) -> Result<WorthServerScheduledMutationResult, WorthQueryRuntimeError> {
    let query_operation = slot
        .handoff()
        .operation()
        .scheduled_query_operation()
        .expect("ordered mutation execution requires a scheduled query operation")
        .clone();
    let handoff = slot.handoff_mut();
    let workspace = handoff.workspace_mut();
    let mut submission_lane = workspace.submissions()?;

    match query_operation {
        WorthServerQueryOperation::SingleMutation { command, .. } => {
            let (receipt, inspection) = submission_lane.submit_with_inspection(*command)?;
            Ok(WorthServerScheduledMutationResult::Single {
                receipt: Box::new(receipt),
                inspection: Box::new(inspection),
            })
        }
        WorthServerQueryOperation::BatchMutation { commands, .. } => {
            let (receipt, inspection) = submission_lane.submit_batch_with_inspection(commands)?;
            Ok(WorthServerScheduledMutationResult::Batch {
                receipt: Box::new(receipt),
                inspection: Box::new(inspection),
            })
        }
    }
}

fn increment_admission_counter(
    counters: &mut WorthServerOperationSchedulerCounters,
    lane: &WorthServerSchedulerLane,
) {
    match lane {
        WorthServerSchedulerLane::DeterministicSubmission { .. } => {
            counters.increment_admitted_submission_slot_count();
        }
        WorthServerSchedulerLane::ProductDraftMutation { .. }
        | WorthServerSchedulerLane::DurableProductMutation { .. }
        | WorthServerSchedulerLane::ProductSessionCoordination { .. } => {
            counters.increment_admitted_mutation_slot_count();
        }
        WorthServerSchedulerLane::SharedRead => {}
    }
}

fn increment_completion_counter(
    counters: &mut WorthServerOperationSchedulerCounters,
    lane: &WorthServerSchedulerLane,
) {
    match lane {
        WorthServerSchedulerLane::DeterministicSubmission { .. } => {
            counters.increment_completed_submission_slot_count();
        }
        WorthServerSchedulerLane::ProductDraftMutation { .. }
        | WorthServerSchedulerLane::DurableProductMutation { .. }
        | WorthServerSchedulerLane::ProductSessionCoordination { .. } => {
            counters.increment_completed_mutation_slot_count();
        }
        WorthServerSchedulerLane::SharedRead => {}
    }
}

/// The basis a mutation asked for and the one its lane now stands on, when
/// they differ.
struct StaleBasis {
    expected_basis_digest: String,
    observed_basis_digest: String,
}

fn stale_basis(
    slot: &WorthServerOperationExecutionSlot,
    latest_lane_basis_digest: Option<&str>,
) -> Result<Option<StaleBasis>, WorthQueryHandleDenial> {
    let requested_basis_digest = match slot.precondition_posture().compatibility_mutation() {
        Some(precondition) => precondition
            .requested_basis_digest()
            .or_else(|| slot.slot_basis_digest()),
        None => slot.slot_basis_digest(),
    };
    let Some(requested_basis_digest) = requested_basis_digest else {
        return Ok(None);
    };
    let observed_basis_digest = match latest_lane_basis_digest {
        Some(latest_lane_basis_digest) => latest_lane_basis_digest.to_string(),
        None => slot
            .handoff()
            .workspace()
            .snapshot_identity()?
            .terminal_projection_for_reporting(),
    };
    Ok(
        (observed_basis_digest != requested_basis_digest).then(|| StaleBasis {
            expected_basis_digest: requested_basis_digest.to_string(),
            observed_basis_digest,
        }),
    )
}

struct PreparedGroups {
    planned_batch_width: usize,
    early_counters: WorthServerOperationSchedulerCounters,
    early_outcomes: Vec<WorthServerScheduledOperationOutcome>,
    groups: BTreeMap<usize, OrderedLaneExecution>,
}

impl PreparedGroups {
    fn new(
        response_facade: WorthServerResponseFacade,
        slots: Vec<WorthServerOperationExecutionSlot>,
        directives: CancellationPlan,
    ) -> Self {
        let planned_batch_width = slots.len();
        let mut early_counters = WorthServerOperationSchedulerCounters::default();
        let mut early_outcomes = Vec::new();
        let mut grouped = HashMap::<String, OrderedLaneExecution>::new();

        for slot in slots {
            if directives.before_admission(slot.ordinal()) {
                early_counters.increment_cancelled_before_admission_count();
                early_outcomes.push(
                    WorthServerScheduledOperationOutcome::cancelled_without_counters(
                        slot,
                        WorthServerSchedulerCancellationPosture::BeforeAdmission,
                    ),
                );
                continue;
            }

            let lane_key = slot.scheduler_lane_key();
            let execution = SlotExecution {
                cancel_after_admission_before_execution: directives
                    .after_admission_before_execution(slot.ordinal()),
                cancel_during_execution: directives.during_execution(slot.ordinal()),
                slot,
            };
            match grouped.entry(lane_key.clone()) {
                Entry::Occupied(mut occupied) => occupied.get_mut().executions.push(execution),
                Entry::Vacant(vacant) => {
                    vacant.insert(OrderedLaneExecution {
                        first_ordinal: execution.slot.ordinal(),
                        lane_key,
                        response_facade: response_facade.clone(),
                        executions: vec![execution],
                    });
                }
            }
        }

        let groups = grouped
            .into_values()
            .map(|group| (group.first_ordinal, group))
            .collect();

        Self {
            planned_batch_width,
            early_counters,
            early_outcomes,
            groups,
        }
    }
}

struct SlotExecution {
    cancel_after_admission_before_execution: bool,
    cancel_during_execution: bool,
    slot: WorthServerOperationExecutionSlot,
}

struct OrderedLaneExecution {
    first_ordinal: usize,
    lane_key: String,
    response_facade: WorthServerResponseFacade,
    executions: Vec<SlotExecution>,
}

struct GroupExecution {
    outcomes: Vec<WorthServerScheduledOperationOutcome>,
    counters: WorthServerOperationSchedulerCounters,
}

#[derive(Default)]
struct CancellationPlan {
    before_admission: Vec<usize>,
    after_admission_before_execution: Vec<usize>,
    during_execution: Vec<usize>,
}

impl CancellationPlan {
    fn new(
        directives: impl IntoIterator<Item = WorthServerSchedulerCancellationDirective>,
    ) -> Self {
        let mut plan = Self::default();
        for directive in directives {
            match directive {
                WorthServerSchedulerCancellationDirective::BeforeAdmission { slot_ordinal } => {
                    plan.before_admission.push(slot_ordinal);
                }
                WorthServerSchedulerCancellationDirective::AfterAdmissionBeforeExecution {
                    slot_ordinal,
                } => {
                    plan.after_admission_before_execution.push(slot_ordinal);
                }
                WorthServerSchedulerCancellationDirective::DuringExecution { slot_ordinal } => {
                    plan.during_execution.push(slot_ordinal);
                }
            }
        }
        plan
    }

    fn before_admission(&self, slot_ordinal: usize) -> bool {
        self.before_admission.contains(&slot_ordinal)
    }

    fn after_admission_before_execution(&self, slot_ordinal: usize) -> bool {
        self.after_admission_before_execution
            .contains(&slot_ordinal)
    }

    fn during_execution(&self, slot_ordinal: usize) -> bool {
        self.during_execution.contains(&slot_ordinal)
    }
}
