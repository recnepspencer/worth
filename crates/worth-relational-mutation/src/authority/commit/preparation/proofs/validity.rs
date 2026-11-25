use std::sync::Arc;

use crate::authority::commit::preparation::planning::context::PreparationPlanningContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparationProofValidity {
    pub(crate) context: Arc<PreparationPlanningContext>,
}
