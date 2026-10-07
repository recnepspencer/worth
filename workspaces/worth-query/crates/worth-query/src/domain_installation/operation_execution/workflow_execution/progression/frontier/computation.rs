use super::{
    ComputedWorkflowFrontier, ComputedWorkflowStage, PreparedWorkflowFrontier,
    WorkflowStageComputation, WorkflowStagePreparation, WorthQueryWorkflowStageComputed,
    WorthQueryWorkflowStageTask,
};

impl PreparedWorkflowFrontier {
    pub(in crate::domain_installation::operation_execution) fn compute(
        self,
    ) -> ComputedWorkflowFrontier {
        self.map_tasks(|tasks, compute| {
            tasks
                .into_iter()
                .map(|(index, task)| (index, compute(task)))
                .collect()
        })
    }

    // The owned-input map enters here in the next slice. The callback receives
    // only inert tasks and a static function, never owner state or an executor.
    pub(super) fn map_tasks(
        self,
        map: impl FnOnce(
            Vec<(usize, WorthQueryWorkflowStageTask)>,
            fn(WorthQueryWorkflowStageTask) -> WorthQueryWorkflowStageComputed,
        ) -> Vec<(usize, WorthQueryWorkflowStageComputed)>,
    ) -> ComputedWorkflowFrontier {
        let mut tasks = Vec::new();
        let mut members = Vec::with_capacity(self.members.len());
        let mut prepared_members = self.members;
        for (index, stage) in self.order.as_slice().iter().enumerate() {
            let member = prepared_members
                .remove(stage)
                .expect("sealed membership owns every canonical key");
            let computation = match member.preparation {
                WorkflowStagePreparation::Ready(task) => {
                    tasks.push((index, task));
                    None
                }
                WorkflowStagePreparation::Failed(failure) => {
                    Some(WorkflowStageComputation::Failed(failure))
                }
                WorkflowStagePreparation::Denied(kind) => {
                    Some(WorkflowStageComputation::Denied(kind))
                }
                WorkflowStagePreparation::Unstarted => Some(WorkflowStageComputation::Unstarted),
            };
            members.push((stage.clone(), member.input, computation));
        }
        for (index, computed) in map(tasks, self.compute) {
            let slot = &mut members[index].2;
            assert!(
                slot.is_none(),
                "an owned task produces exactly one indexed result"
            );
            *slot = Some(WorkflowStageComputation::Ready(computed));
        }
        let members = members
            .into_iter()
            .map(|(stage, input, computation)| {
                (
                    stage,
                    ComputedWorkflowStage {
                        input,
                        computation: computation
                            .expect("every prepared member is computed before application"),
                    },
                )
            })
            .collect();
        ComputedWorkflowFrontier {
            identity: self.identity,
            owner: self.owner,
            order: self.order,
            members,
        }
    }
}
