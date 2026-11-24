//! Managed-index forward and predicate providers for native String and binary probes.
use super::queries::entry_name;
use super::schema::*;
use std::cmp::Ordering;
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Copy)]
pub struct Node {
    slot: WorthQueryManagedIndexSourceSlot,
    previous: Option<WorthQueryManagedIndexNodeId>,
    count: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IndexDenial;
impl std::fmt::Display for IndexDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("index denial")
    }
}
impl std::error::Error for IndexDenial {}
type Stop = WorthQueryManagedIndexProviderStop<IndexDenial>;

fn build_chain<
    I: worth_query_decl::facade::application_query::ApplicationManagedIndexBinding<CatalogSchema>,
>(
    sources: &WorthQueryManagedIndexSourceRows<'_, CatalogSchema, I>,
    nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
    let mut root = None;
    for (n, slot) in sources.slots().enumerate() {
        control.checkpoint(1)?;
        let slot = slot?;
        root = Some(nodes.push(Node {
            slot,
            previous: root,
            count: n + 1,
        })?);
    }
    Ok(root)
}

fn update_chain<
    I: worth_query_decl::facade::application_query::ApplicationManagedIndexBinding<CatalogSchema>,
>(
    after: &WorthQueryManagedIndexSourceRows<'_, CatalogSchema, I>,
    changes: &[WorthQueryManagedIndexSourceChange],
    mut root: Option<WorthQueryManagedIndexNodeId>,
    nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
    for change in changes {
        control.checkpoint(1)?;
        match (change.before(), change.after()) {
            (None, Some(slot)) => {
                let count = match root {
                    Some(id) => nodes.get(id)?.count + 1,
                    None => 1,
                };
                root = Some(nodes.push(Node {
                    slot,
                    previous: root,
                    count,
                })?);
            }
            (Some(_), Some(_)) => {}
            _ => return Err(Stop::Domain(IndexDenial)),
        }
    }
    if after.is_empty() {
        root = None;
    }
    Ok(root)
}

pub struct NativePrefixProvider;
impl WorthQueryManagedIndexProvider<CatalogSchema, NativePrefixIndex> for NativePrefixProvider {
    type Node = Node;
    type Summary = usize;
    type Denial = IndexDenial;
    const IDENTITY: &'static str = NativePrefixIndex::PROVIDER;
    const VERSION: u32 = NativePrefixIndex::PROVIDER_VERSION;

    fn source_summary(
        &self,
        _: WorthQueryManagedIndexSourceSlot,
        _: WorthQueryManagedIndexSourceRow<'_, CatalogSchema, NativePrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        Ok(1)
    }
    fn merge_summaries(
        &self,
        parts: [Option<&usize>; 3],
        _: &WorthQueryManagedIndexSummarySources<'_, '_, CatalogSchema, NativePrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        Ok(parts.into_iter().flatten().copied().sum())
    }
    fn node_summary<'n>(
        &self,
        node: &'n Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<&'n usize, Stop> {
        control.checkpoint(1)?;
        Ok(&node.count)
    }
    fn summary_may_match(
        &self,
        _: &usize,
        _: &WorthQueryManagedIndexSummarySources<'_, '_, CatalogSchema, NativePrefixIndex>,
        _: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        control.checkpoint(1)?;
        Ok(true)
    }
    fn node_capacity(
        &self,
        count: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        Ok(count)
    }
    fn build(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, CatalogSchema, NativePrefixIndex>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        build_chain(sources, nodes, control)
    }
    fn decompose(
        &self,
        node: &Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<WorthQueryManagedIndexNodeRegion, Stop> {
        control.checkpoint(1)?;
        Ok(WorthQueryManagedIndexNodeRegion::new(
            Some(node.slot),
            [node.previous, None],
        ))
    }
    fn matches_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, CatalogSchema, NativePrefixIndex>,
        probe: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        control.checkpoint(1)?;
        let name = row
            .field(entry_name())
            .map_err(|_| Stop::Domain(IndexDenial))?;
        Ok(name.starts_with(probe.as_str()))
    }
    fn update(
        &self,
        _: &WorthQueryManagedIndexSourceRows<'_, CatalogSchema, NativePrefixIndex>,
        after: &WorthQueryManagedIndexSourceRows<'_, CatalogSchema, NativePrefixIndex>,
        changes: &[WorthQueryManagedIndexSourceChange],
        root: Option<WorthQueryManagedIndexNodeId>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        update_chain(after, changes, root, nodes, control)
    }
}
impl WorthQueryManagedIndexPredicateProvider<CatalogSchema, NativePrefixIndex>
    for NativePrefixProvider
{
    type PredicateSummary = ();
    fn compare_probes(
        &self,
        a: &String,
        b: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, Stop> {
        control.checkpoint(1)?;
        Ok(a.cmp(b))
    }
    fn predicate_summary(
        &self,
        _: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        control.checkpoint(1)?;
        Ok(())
    }
    fn predicate_region_may_match(
        &self,
        _: &(),
        _: [&String; 2],
        _: WorthQueryManagedIndexSourceRow<'_, CatalogSchema, NativePrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        control.checkpoint(1)?;
        Ok(true)
    }
}

pub struct BinaryTagProvider;
impl WorthQueryManagedIndexProvider<CatalogSchema, BinaryTagIndex> for BinaryTagProvider {
    type Node = Node;
    type Summary = usize;
    type Denial = IndexDenial;
    const IDENTITY: &'static str = BinaryTagIndex::PROVIDER;
    const VERSION: u32 = BinaryTagIndex::PROVIDER_VERSION;

    fn source_summary(
        &self,
        _: WorthQueryManagedIndexSourceSlot,
        _: WorthQueryManagedIndexSourceRow<'_, CatalogSchema, BinaryTagIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        Ok(1)
    }
    fn merge_summaries(
        &self,
        parts: [Option<&usize>; 3],
        _: &WorthQueryManagedIndexSummarySources<'_, '_, CatalogSchema, BinaryTagIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        Ok(parts.into_iter().flatten().copied().sum())
    }
    fn node_summary<'n>(
        &self,
        node: &'n Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<&'n usize, Stop> {
        control.checkpoint(1)?;
        Ok(&node.count)
    }
    fn summary_may_match(
        &self,
        _: &usize,
        _: &WorthQueryManagedIndexSummarySources<'_, '_, CatalogSchema, BinaryTagIndex>,
        _: &TagBytes,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        control.checkpoint(1)?;
        Ok(true)
    }
    fn node_capacity(
        &self,
        count: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        Ok(count)
    }
    fn build(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, CatalogSchema, BinaryTagIndex>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        build_chain(sources, nodes, control)
    }
    fn decompose(
        &self,
        node: &Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<WorthQueryManagedIndexNodeRegion, Stop> {
        control.checkpoint(1)?;
        Ok(WorthQueryManagedIndexNodeRegion::new(
            Some(node.slot),
            [node.previous, None],
        ))
    }
    fn matches_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, CatalogSchema, BinaryTagIndex>,
        probe: &TagBytes,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        control.checkpoint(1)?;
        let name = row
            .field(entry_name())
            .map_err(|_| Stop::Domain(IndexDenial))?;
        Ok(name.as_bytes().starts_with(&probe.0))
    }
    fn update(
        &self,
        _: &WorthQueryManagedIndexSourceRows<'_, CatalogSchema, BinaryTagIndex>,
        after: &WorthQueryManagedIndexSourceRows<'_, CatalogSchema, BinaryTagIndex>,
        changes: &[WorthQueryManagedIndexSourceChange],
        root: Option<WorthQueryManagedIndexNodeId>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        update_chain(after, changes, root, nodes, control)
    }
}
impl WorthQueryManagedIndexPredicateProvider<CatalogSchema, BinaryTagIndex> for BinaryTagProvider {
    type PredicateSummary = ();
    fn compare_probes(
        &self,
        a: &TagBytes,
        b: &TagBytes,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, Stop> {
        control.checkpoint(1)?;
        Ok(a.0.cmp(&b.0))
    }
    fn predicate_summary(
        &self,
        _: &TagBytes,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        control.checkpoint(1)?;
        Ok(())
    }
    fn predicate_region_may_match(
        &self,
        _: &(),
        _: [&TagBytes; 2],
        _: WorthQueryManagedIndexSourceRow<'_, CatalogSchema, BinaryTagIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        control.checkpoint(1)?;
        Ok(true)
    }
}
