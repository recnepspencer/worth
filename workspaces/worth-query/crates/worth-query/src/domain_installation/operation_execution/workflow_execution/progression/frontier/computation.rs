//! One owned-map dispatch, including the serial and singleton postures.
use super::{
    ComputedWorkflowFrontier, ComputedWorkflowStage, PreparedWorkflowFrontier,
    WorkflowFrontierFailure, WorkflowFrontierStop, WorkflowStagePreparation,
    WorthQueryWorkflowAdvanceDenialKind as Denial,
};
use std::collections::{BTreeMap, HashMap};
use worth_execution::{
    ExecutionMap, ExecutionRequest, ExecutionWorkCeiling, KeylessPartition, LeaseDenial,
    MapKernelFailure, MapKernelStop, MapOutcome,
};
use worth_foundational::PartitionIdentity;

impl PreparedWorkflowFrontier {
    pub(in crate::domain_installation::operation_execution) fn compute(
        self,
        request: ExecutionRequest<'_, '_>,
    ) -> ComputedWorkflowFrontier {
        let mut batch = ComputedWorkflowFrontier {
            identity: self.identity,
            owner: self.owner,
            order: self.order,
            prefix: Vec::new(),
            stop: None,
            charged_work: 0,
        };
        if let Some(kind) = self.shape_denial {
            return batch.refused(kind);
        }
        let mut partitions = BTreeMap::new();
        let mut inputs = HashMap::new();
        let mut declared_work = 0_u64;
        let mut preparation_stop = None;
        // The prepare phase owns every input at exactly its canonical rank.
        let mut members = self.members;
        for rank in 0..batch.order.as_slice().len() {
            let Some(member) = members.remove(&rank) else {
                break;
            };
            match member.preparation {
                WorkflowStagePreparation::Ready(task) => {
                    let Some(work) = declared_work.checked_add(task.declared_work()) else {
                        return batch.refused(Denial::ComputationWorkExhausted {
                            stage_identity: None,
                            cause: MapKernelStop::WorkCounterOverflow,
                        });
                    };
                    declared_work = work;
                    let (Some(capacity), Ok(identity)) =
                        (task.result_capacity(), u64::try_from(rank))
                    else {
                        return batch.refused(Denial::ComputationAdmission(
                            LeaseDenial::ChargedBytesOverflow,
                        ));
                    };
                    partitions.insert(
                        PartitionIdentity::new(identity),
                        KeylessPartition {
                            kernel_scratch_bytes: task.scratch_bytes(),
                            max_result_bytes: capacity,
                            value: task,
                        },
                    );
                    inputs.insert(rank, member.input);
                }
                WorkflowStagePreparation::Failed(failure) => {
                    preparation_stop = Some(WorkflowFrontierStop {
                        rank: Some(rank),
                        input: Some(member.input),
                        failure: WorkflowFrontierFailure::Preparation(failure),
                    });
                    break;
                }
                WorkflowStagePreparation::Denied(kind) => {
                    preparation_stop = Some(WorkflowFrontierStop {
                        rank: Some(rank),
                        input: Some(member.input),
                        failure: WorkflowFrontierFailure::Denied(kind),
                    });
                    break;
                }
            }
        }
        let map = match ExecutionMap::<_, ()>::from_keyless_partitions(partitions) {
            Ok(map) => map,
            Err(_) => {
                return batch.refused(Denial::ComputationAdmission(
                    LeaseDenial::ChargedBytesOverflow,
                ))
            }
        };
        let outcome = match request.run(ExecutionWorkCeiling::new(declared_work), |lease| {
            map.run_owned(lease, |task, meter| {
                let computed = (self.compute)(task, meter);
                if computed.failed() {
                    Err(MapKernelFailure::Domain(computed))
                } else {
                    Ok(computed)
                }
            })
        }) {
            Ok((outcome, _)) => outcome,
            Err(cause) => {
                batch.stop = Some(WorkflowFrontierStop {
                    rank: None,
                    input: None,
                    failure: super::stop_conversion::scope(cause),
                });
                return batch;
            }
        };
        batch.charged_work = outcome.report().charged_work();
        let (values, map_stop) = match outcome {
            MapOutcome::Complete { values, .. } => (values, None),
            MapOutcome::Stopped {
                completed_prefix,
                boundary,
                reason,
                ..
            } => {
                let rank = match boundary {
                    Some(identity) => match usize::try_from(identity.value()) {
                        Ok(rank) => Some(rank),
                        Err(_) => return batch.refused(Denial::ParallelFrontierShape),
                    },
                    None => None,
                };
                let stage = rank.and_then(|rank| batch.order.as_slice().get(rank).cloned());
                let failure = super::stop_conversion::map(reason, stage);
                let input = rank.and_then(|rank| inputs.remove(&rank));
                (
                    completed_prefix,
                    Some(WorkflowFrontierStop {
                        rank,
                        input,
                        failure,
                    }),
                )
            }
        };
        for (rank, computed) in values.into_iter().enumerate() {
            let Some(input) = inputs.remove(&rank) else {
                return batch.refused(Denial::ParallelFrontierShape);
            };
            batch.prefix.push(ComputedWorkflowStage {
                rank,
                input,
                computed,
            });
        }
        batch.stop = map_stop.or(preparation_stop);
        batch
    }
}
impl ComputedWorkflowFrontier {
    fn refused(mut self, kind: Denial) -> Self {
        self.prefix.clear();
        self.stop = Some(WorkflowFrontierStop {
            rank: None,
            input: None,
            failure: WorkflowFrontierFailure::Denied(kind),
        });
        self
    }
}
