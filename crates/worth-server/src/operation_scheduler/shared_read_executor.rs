use std::collections::{hash_map::Entry, BTreeMap, HashMap};

use crate::{
    WorthServerQueryHandoffOperation, WorthServerResponseFacade, WorthServerResponseInput,
};

use super::{
    WorthServerExecutedOperationBatch, WorthServerOperationExecutionSlot,
    WorthServerOperationSchedulerCounters, WorthServerScheduledOperationOutcome,
    WorthServerSchedulerCancellationDirective, WorthServerSchedulerCancellationPosture,
    WorthServerSchedulerCertificationSabotage, WorthServerSchedulerFailurePosture,
    WorthServerSchedulerRuntimeFailure,
};

pub(crate) fn execute_slots(
    response_facade: WorthServerResponseFacade,
    slots: Vec<WorthServerOperationExecutionSlot>,
    directives: impl IntoIterator<Item = WorthServerSchedulerCancellationDirective>,
    sabotage: impl IntoIterator<Item = WorthServerSchedulerCertificationSabotage>,
    parallel: bool,
) -> WorthServerExecutedOperationBatch {
    let directives = CancellationPlan::new(directives);
    let sabotage = CertificationSabotagePlan::new(sabotage);
    let prepared = PreparedGroups::new(response_facade, slots, directives, sabotage);
    let mut counters = WorthServerOperationSchedulerCounters::default();
    counters.set_planned_batch_width(prepared.planned_batch_width);
    counters.absorb(&prepared.early_counters);

    let mut outcomes = prepared.early_outcomes;
    let group_outcomes = if parallel {
        run_parallel_groups(prepared.groups)
    } else {
        run_serialized_groups(prepared.groups)
    };
    for group in group_outcomes {
        counters.absorb(&group.counters);
        outcomes.extend(group.outcomes);
    }

    outcomes.sort_by_key(WorthServerScheduledOperationOutcome::ordinal);
    let counters_snapshot = counters.clone();
    for outcome in &mut outcomes {
        outcome.attach_scheduler_counters(counters_snapshot.clone());
    }
    WorthServerExecutedOperationBatch::new(outcomes, counters)
}

fn run_parallel_groups(groups: BTreeMap<usize, DependencyGroupExecution>) -> Vec<GroupExecution> {
    groups.into_values().map(execute_group).collect()
}

fn run_serialized_groups(groups: BTreeMap<usize, DependencyGroupExecution>) -> Vec<GroupExecution> {
    groups.into_values().map(execute_group).collect()
}

fn execute_group(group: DependencyGroupExecution) -> GroupExecution {
    let mut outcomes = Vec::with_capacity(group.executions.len());
    let mut counters = WorthServerOperationSchedulerCounters::default();
    let mut failed_slot_ordinal = None;

    for mut execution in group.executions {
        if let Some(failed_ordinal) = failed_slot_ordinal {
            counters.increment_dependent_failure_count();
            outcomes.push(
                WorthServerScheduledOperationOutcome::failed_without_counters(
                    execution.slot,
                    WorthServerSchedulerFailurePosture::DependentSharedBasisFailure {
                        shared_basis_key: group.dependency_group.clone(),
                        failed_slot_ordinal: failed_ordinal,
                    },
                ),
            );
            continue;
        }

        let before_lock_count = execution.slot.shared_read_hot_path_lock_count();
        let shared_read_basis_identity = match execution.slot.mint_shared_read_context() {
            Ok(shared_read_context) => {
                counters.increment_admitted_read_slot_count();
                if execution.sabotage_forbidden_global_lock_after_admission {
                    execution
                        .slot
                        .record_shared_read_hot_path_lock_for_certification();
                }
                let after_admission_lock_count = execution.slot.shared_read_hot_path_lock_count();
                counters.add_forbidden_global_lock_acquisitions(
                    after_admission_lock_count.saturating_sub(before_lock_count),
                );
                shared_read_context
                    .inspect_basis()
                    .snapshot_evidence_identity()
                    .terminal_projection_for_reporting()
                    .to_string()
            }
            Err(error) => {
                let after_lock_count = execution.slot.shared_read_hot_path_lock_count();
                counters.add_forbidden_global_lock_acquisitions(
                    after_lock_count.saturating_sub(before_lock_count),
                );
                counters.increment_isolated_failure_count();
                failed_slot_ordinal = Some(execution.slot.ordinal());
                outcomes.push(
                    WorthServerScheduledOperationOutcome::failed_without_counters(
                        execution.slot,
                        WorthServerSchedulerFailurePosture::IsolatedRuntimeFailure {
                            runtime_failure: WorthServerSchedulerRuntimeFailure::opaque(
                                error.to_string(),
                            ),
                        },
                    ),
                );
                continue;
            }
        };

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

        counters.increment_queued_read_slot_count();
        let before_execution_lock_count = execution.slot.shared_read_hot_path_lock_count();
        let execution_result = execute_slot_operation(&mut execution.slot);
        let after_execution_lock_count = execution.slot.shared_read_hot_path_lock_count();
        counters.add_forbidden_global_lock_acquisitions(
            after_execution_lock_count.saturating_sub(before_execution_lock_count),
        );

        match execution_result {
            Ok(execution_digest) => {
                if execution.cancel_during_execution {
                    counters.increment_cancelled_during_execution_count();
                    outcomes.push(
                        WorthServerScheduledOperationOutcome::cancelled_without_counters(
                            execution.slot,
                            WorthServerSchedulerCancellationPosture::DuringExecution,
                        ),
                    );
                } else {
                    counters.increment_completed_read_slot_count();
                    let response_envelope = group.response_facade.shape_with_defaults(
                        WorthServerResponseInput::query_handoff_success(
                            execution.slot.take_handoff(),
                        ),
                    );
                    outcomes.push(
                        WorthServerScheduledOperationOutcome::success_without_counters(
                            execution.slot,
                            shared_read_basis_identity,
                            execution_digest,
                            response_envelope,
                        ),
                    );
                }
            }
            Err(detail) => {
                counters.increment_isolated_failure_count();
                failed_slot_ordinal = Some(execution.slot.ordinal());
                outcomes.push(
                    WorthServerScheduledOperationOutcome::failed_without_counters(
                        execution.slot,
                        WorthServerSchedulerFailurePosture::IsolatedRuntimeFailure {
                            runtime_failure: WorthServerSchedulerRuntimeFailure::opaque(detail),
                        },
                    ),
                );
            }
        }
    }

    GroupExecution { outcomes, counters }
}

fn execute_slot_operation(slot: &mut WorthServerOperationExecutionSlot) -> Result<String, String> {
    let handoff = slot.handoff_mut();
    let operation = handoff.operation().clone();
    handoff
        .workspace_mut()
        .with_execution_request_mut(|execution, workspace| match operation {
            WorthServerQueryHandoffOperation::QueryRead { operation_name }
            | WorthServerQueryHandoffOperation::DirectRead { operation_name } => workspace
                .resolve_live_artifact_target(&operation_name)
                .and_then(|target| workspace.read_live_target(execution, &target))
                .map(|result| result.receipt().result_digest().to_string())
                .map_err(|error| error.to_string()),
            WorthServerQueryHandoffOperation::DirectState { target_label } => workspace
                .resolve_live_artifact_target(&target_label)
                .and_then(|target| workspace.state_live_target(&target))
                .map(|state| {
                    state
                        .state_digest()
                        .terminal_projection_for_reporting()
                        .to_string()
                })
                .map_err(|error| error.to_string()),
            WorthServerQueryHandoffOperation::DirectInspection { target_label } => workspace
                .resolve_live_artifact_target(&target_label)
                .and_then(|target| {
                    workspace
                        .inspections()?
                        .inspect_live_target(execution, &target)
                })
                .map(|inspection| inspection.receipt().result_digest().to_string())
                .map_err(|error| error.to_string()),
            unsupported => Err(format!(
                "shared-read scheduler cannot execute unsupported operation `{unsupported:?}`"
            )),
        })
        .map_err(|error| error.to_string())?
}

struct PreparedGroups {
    planned_batch_width: usize,
    early_counters: WorthServerOperationSchedulerCounters,
    early_outcomes: Vec<WorthServerScheduledOperationOutcome>,
    groups: BTreeMap<usize, DependencyGroupExecution>,
}

impl PreparedGroups {
    fn new(
        response_facade: WorthServerResponseFacade,
        slots: Vec<WorthServerOperationExecutionSlot>,
        directives: CancellationPlan,
        sabotage: CertificationSabotagePlan,
    ) -> Self {
        let planned_batch_width = slots.len();
        let mut early_counters = WorthServerOperationSchedulerCounters::default();
        let mut early_outcomes = Vec::new();
        let mut grouped = HashMap::<String, DependencyGroupExecution>::new();

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

            let dependency_group = slot
                .dependency_group()
                .expect("shared-read scheduler requires a dependency group")
                .to_string();
            let execution = SlotExecution {
                cancel_after_admission_before_execution: directives
                    .after_admission_before_execution(slot.ordinal()),
                cancel_during_execution: directives.during_execution(slot.ordinal()),
                sabotage_forbidden_global_lock_after_admission: sabotage
                    .forbidden_global_lock_after_admission(slot.ordinal()),
                slot,
            };
            match grouped.entry(dependency_group.clone()) {
                Entry::Occupied(mut occupied) => occupied.get_mut().executions.push(execution),
                Entry::Vacant(vacant) => {
                    vacant.insert(DependencyGroupExecution {
                        first_ordinal: execution.slot.ordinal(),
                        dependency_group,
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
    sabotage_forbidden_global_lock_after_admission: bool,
    slot: WorthServerOperationExecutionSlot,
}

struct DependencyGroupExecution {
    first_ordinal: usize,
    dependency_group: String,
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

#[derive(Default)]
struct CertificationSabotagePlan {
    forbidden_global_lock_after_admission: Vec<usize>,
}

impl CertificationSabotagePlan {
    fn new(sabotage: impl IntoIterator<Item = WorthServerSchedulerCertificationSabotage>) -> Self {
        let mut plan = Self::default();
        for directive in sabotage {
            match directive {
                WorthServerSchedulerCertificationSabotage::ForbiddenGlobalLockAfterAdmission {
                    slot_ordinal,
                } => {
                    plan.forbidden_global_lock_after_admission
                        .push(slot_ordinal);
                }
            }
        }
        plan
    }

    fn forbidden_global_lock_after_admission(&self, slot_ordinal: usize) -> bool {
        self.forbidden_global_lock_after_admission
            .contains(&slot_ordinal)
    }
}
