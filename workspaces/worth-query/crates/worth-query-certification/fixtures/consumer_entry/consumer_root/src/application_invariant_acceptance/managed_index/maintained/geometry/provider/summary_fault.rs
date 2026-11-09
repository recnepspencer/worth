//! Deliberately correct changed leaves retain stale real ancestors for the court.
use super::*;
pub(super) fn replace_leaves<Index>(
    after: &WorthQueryManagedIndexSourceRows<'_, Catalog, Index>,
    changes: &[WorthQueryManagedIndexSourceChange],
    nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
    calls: &Calls,
) -> Result<(), Stop>
where
    Index: ApplicationManagedIndexBinding<
        Catalog,
        SourceBinding = query::SourceBinding,
        ProbeBinding = CoordinatesBinding,
        Entity = Edge,
    >,
{
    for change in changes {
        let source = change.after().ok_or_else(storage::invalid)?;
        let id = nodes.before_source_node(source)?;
        let mut node = nodes.get(id)?.material;
        if !matches!(node.links, worth_query_consumer_values::BoxNodeLinks::Leaf { source: actual } if actual == source)
        {
            return Err(storage::invalid());
        }
        let bounds = bounds(after, source, calls)?;
        node.bounds = bounds;
        node.last_bounds = bounds;
        nodes.replace(id, node.into())?;
    }
    Ok(())
}
