//! Recompute pure summaries from actual Source support and already qualified children.
use super::*;
use worth_query_declaration::facade::{
    application_query::*, application_schema::ApplicationSchema,
};
use {
    super::super::invocation::provider_result, super::super::WorthQueryManagedIndexNodes,
    super::super::WorthQueryManagedIndexProvider, super::super::WorthQueryManagedIndexSourceRows,
    crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial as Denial,
};

pub(super) fn qualify<S, I, P>(
    execution: &ExecutionRequest<'_, '_>,
    provider: &P,
    source: &WorthQueryManagedIndexSourceRows<'_, S, I>,
    nodes: &WorthQueryManagedIndexNodes<P::Node>,
    tree: &SourceTree,
    affected: Option<&Map<usize, ()>>,
    control: &Control<'_>,
) -> Result<(), Denial>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    let context = SummaryContext::new(tree, source, control);
    summary_frontier::postorder(tree, affected, control, &mut |ordinal| {
        let region = tree.region(ordinal, control)?;
        let node = nodes
            .node_at_ordinal(ordinal, control)?
            .ok_or_else(|| control.refuse(Stop::SourceStructureIncomplete))?;
        let mut parts = [None; 3];
        if let Some(slot) = region.source {
            parts[0] = Some(provider_result(
                execution,
                control,
                provider.source_summary(slot, source.get(slot)?, control),
            )?);
        }
        for (position, child) in region.children.into_iter().enumerate() {
            if let Some(child) = child {
                let child = nodes
                    .node_at_ordinal(child, control)?
                    .ok_or_else(|| control.refuse(Stop::SourceStructureIncomplete))?;
                parts[position + 1] = Some(provider_result(
                    execution,
                    control,
                    provider.node_summary(*child, control),
                )?);
            }
        }
        let sources = WorthQueryManagedIndexSummarySources::new(&context, ordinal);
        let expected = provider_result(
            execution,
            control,
            provider.merge_summaries(parts, &sources, control),
        )?;
        let actual = provider_result(execution, control, provider.node_summary(*node, control))?;
        if actual != expected {
            return Err(control.refuse(Stop::SummaryMismatch).into());
        }
        Ok(())
    })
}
