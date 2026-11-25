//! Lend matching before and after views under the admitted Source budget.
use super::SourceViews;
use crate::validation::candidate_source::preparation_group::SourcePreparationGroup;
use crate::validation::custom_rule::{
    CustomInvariantWorkMeter, StructuralAspectStateView, StructuralRelationView,
};
use crate::validation::engine::{
    state_view::InvariantStateView, InvariantObservation, InvariantRuntimeView,
};
use std::sync::Arc;

pub fn invariant_source_views<'a>(
    runtime: &InvariantRuntimeView<'a>,
    observation: &InvariantObservation<'a>,
    before: InvariantStateView<'a>,
    after: InvariantStateView<'a>,
    execution: worth_execution::ExecutionRequest<'a, 'a>,
    group: &SourcePreparationGroup,
    work: &CustomInvariantWorkMeter,
) -> SourceViews<'a> {
    let access = Arc::new(group.access_contract().clone());
    SourceViews {
        before: StructuralRelationView::new(
            before.clone(),
            execution,
            work.clone(),
            access.clone(),
        )
        .with_structural_source(runtime, observation, true),
        after: StructuralRelationView::new(after.clone(), execution, work.clone(), access.clone())
            .with_structural_source(runtime, observation, false),
        before_fields: StructuralAspectStateView::new(before, work.clone(), access.clone()),
        after_fields: StructuralAspectStateView::new(after, work.clone(), access),
    }
}
