use crate::basis_lifecycle::BasisOperationLane;
use std::collections::HashMap;

use super::super::super::{
    WorthQueryWorkflowAdvanceDenialKind, WorthQueryWorkflowPreparationPredecessor,
    WorthQueryWorkflowRun, WorthQueryWorkflowStageComputationFailure,
    WorthQueryWorkflowStageInputFacts, WorthQueryWorkflowStagePreparation, WorthQueryWorkflowValue,
};
use super::{
    PreparedWorkflowFrontier, PreparedWorkflowStage, WorkflowStageComputationIdentity,
    WorkflowStagePreparation,
};

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane> WorthQueryWorkflowRun<D, O, F, L> {
    pub(in crate::domain_installation::operation_execution) fn prepare_frontier_computation(
        &self,
        stages: Vec<(String, WorthQueryWorkflowValue)>,
    ) -> PreparedWorkflowFrontier {
        let order = worth_proof::CanonicalUniqueVec::try_from_sorted_unique(
            stages
                .iter()
                .map(|(identity, _)| identity.clone())
                .collect(),
        )
        .expect("admitted frontier membership is canonical and unique");
        let identity = if stages.len() == 1 {
            // A singleton is canonical by construction; HEAD paid no batch hash.
            format!(
                "{}:admission:{}",
                self.identity, self.counters.stage_admission_checks
            )
        } else {
            crate::identity::hash_parts(&[
                "worth-query-workflow-computation-frontier-v1".into(),
                self.identity.clone(),
                self.bound.binding_identity().into(),
                format!("{:?}", order.as_slice()),
                // Admission advances on every attempt, including effect-free retries.
                format!("admission:{}", self.counters.stage_admission_checks),
            ])
        };
        let mut stopped = false;
        let mut members = HashMap::with_capacity(stages.len());
        for (stage, input) in stages {
            let preparation = if stopped {
                WorkflowStagePreparation::Unstarted
            } else {
                let preparation = self.prepare_frontier_member(&identity, &stage, &input);
                stopped = !matches!(&preparation, WorkflowStagePreparation::Ready(_));
                preparation
            };
            // Membership and input are paired at this sole constructor. The
            // canonical proof owns the keys; no parallel sequence is zipped.
            members.insert(stage, PreparedWorkflowStage { input, preparation });
        }
        PreparedWorkflowFrontier {
            identity,
            owner: self.identity.clone(),
            order,
            members,
            compute: self.executor.computation(),
        }
    }

    fn prepare_frontier_member(
        &self,
        frontier: &str,
        stage_identity: &str,
        input: &WorthQueryWorkflowValue,
    ) -> WorkflowStagePreparation {
        let Some(stage) = self.graph.stage(stage_identity) else {
            return WorkflowStagePreparation::Denied(
                WorthQueryWorkflowAdvanceDenialKind::UnknownStage,
            );
        };
        let predecessors = stage
            .predecessors()
            .iter()
            .map(|name| {
                let receipt = self
                    .receipt_index
                    .get(name)
                    .and_then(|index| self.receipts.get(*index))
                    .ok_or_else(|| {
                        WorthQueryWorkflowAdvanceDenialKind::PredecessorAuthorityMissing(
                            name.clone(),
                        )
                    })?;
                Ok(WorthQueryWorkflowPreparationPredecessor {
                    identity: receipt.identity(),
                    stage: receipt.stage_identity(),
                    output: WorthQueryWorkflowStageInputFacts::from_value(receipt.output()),
                })
            })
            .collect::<Result<Vec<_>, _>>();
        let predecessors = match predecessors {
            Ok(receipts) => receipts,
            Err(kind) => return WorkflowStagePreparation::Denied(kind),
        };
        let expected = WorkflowStageComputationIdentity {
            frontier: frontier.into(),
            stage: stage_identity.into(),
        };
        let view = WorthQueryWorkflowStagePreparation::new(
            expected.clone(),
            WorthQueryWorkflowStageInputFacts::from_value(input),
            predecessors,
        );
        match self.executor.prepare(view) {
            Ok(task) if task.identity() == &expected => WorkflowStagePreparation::Ready(task),
            Ok(_) => WorkflowStagePreparation::Failed(WorthQueryWorkflowStageComputationFailure::new(
                worth_query_installation::facade::WorthQueryOperationFailureClass::Indeterminate,
                "prepared workflow task belongs to another stage occurrence",
            )),
            Err(failure) => WorkflowStagePreparation::Failed(failure),
        }
    }
}
