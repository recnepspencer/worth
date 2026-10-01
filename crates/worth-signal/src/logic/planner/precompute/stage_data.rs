use crate::data::handle::NodeId;
use crate::data::proof::{OrderedStreamItem, SingleConsumer};
use crate::logic::prepared::PreparedEvaluation;

use super::super::types::EligibleTask;

#[derive(Debug, Clone)]
pub(in crate::logic::planner) struct PreparedTaskPatch {
    pub task_index: usize,
    pub node: NodeId,
    pub prepared: PreparedEvaluation,
}

impl OrderedStreamItem for (usize, PreparedEvaluation) {
    type OrderKey = usize;

    fn order_key(&self) -> Self::OrderKey {
        self.0
    }
}

impl OrderedStreamItem for PreparedTaskPatch {
    type OrderKey = usize;

    fn order_key(&self) -> Self::OrderKey {
        self.task_index
    }
}

pub(in crate::logic::planner) enum StageExecutionData {
    Prepared(SingleConsumer<Vec<PreparedEvaluation>>),
}

impl StageExecutionData {
    pub(in crate::logic::planner) fn len(&self) -> usize {
        match self {
            Self::Prepared(prepared) => prepared.as_ref().len(),
        }
    }

    pub(in crate::logic::planner) fn into_patches(
        self,
        tasks: &[EligibleTask],
    ) -> Vec<PreparedTaskPatch> {
        match self {
            Self::Prepared(prepared) => prepared
                .into_inner()
                .into_iter()
                .enumerate()
                .map(|(task_index, prepared)| PreparedTaskPatch {
                    task_index,
                    node: tasks[task_index].node,
                    prepared,
                })
                .collect(),
        }
    }
}
