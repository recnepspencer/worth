use super::*;
#[derive(Clone, Copy)]
pub struct Node {
    slot: WorthQueryManagedIndexSourceSlot,
    previous: Option<WorthQueryManagedIndexNodeId>,
    count: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Denial;
impl std::fmt::Display for Denial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid sample frame")
    }
}
impl std::error::Error for Denial {}
type Stop = WorthQueryManagedIndexProviderStop<Denial>;
pub struct Provider(pub Arc<Calls>);
impl WorthQueryManagedIndexProvider<Schema, Index> for Provider {
    type Node = Node;
    type Summary = usize;
    type Denial = Denial;
    const IDENTITY: &'static str = Index::PROVIDER;
    const VERSION: u32 = 1;
    fn source_summary(
        &self,
        _: WorthQueryManagedIndexSourceSlot,
        row: WorthQueryManagedIndexSourceRow<'_, Schema, Index>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        row.optional_aspect_field(query::content())
            .map_err(|_| Stop::Domain(Denial))?;
        Ok(1)
    }
    fn merge_summaries(
        &self,
        parts: [Option<&usize>; 3],
        _: &WorthQueryManagedIndexSummarySources<'_, '_, Schema, Index>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(3)?;
        parts.into_iter().flatten().try_fold(0usize, |sum, n| {
            sum.checked_add(*n).ok_or(Stop::Domain(Denial))
        })
    }
    fn node_summary<'node>(
        &self,
        node: &'node Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<&'node usize, Stop> {
        control.checkpoint(1)?;
        Ok(&node.count)
    }
    fn summary_may_match(
        &self,
        _: &usize,
        _: &WorthQueryManagedIndexSummarySources<'_, '_, Schema, Index>,
        _: &u64,
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
        sources: &WorthQueryManagedIndexSourceRows<'_, Schema, Index>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        let mut root = None;
        for (n, slot) in sources.slots().enumerate() {
            control.checkpoint(1)?;
            let slot = slot?;
            let summary = self.source_summary(slot, sources.get(slot)?, control)?;
            assert_eq!(summary, 1);
            root = Some(nodes.push(Node {
                slot,
                previous: root,
                count: n + 1,
            })?);
        }
        Ok(root)
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
        _: WorthQueryManagedIndexSourceRow<'_, Schema, Index>,
        _: &u64,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        control.checkpoint(1)?;
        Ok(true)
    }
    fn update(
        &self,
        before: &WorthQueryManagedIndexSourceRows<'_, Schema, Index>,
        after: &WorthQueryManagedIndexSourceRows<'_, Schema, Index>,
        changes: &[WorthQueryManagedIndexSourceChange],
        mut root: Option<WorthQueryManagedIndexNodeId>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.optional_updates.fetch_add(1, Ordering::SeqCst);
        for change in changes {
            control.checkpoint(1)?;
            let observe = |rows: &WorthQueryManagedIndexSourceRows<'_, Schema, Index>,
                           slot|
             -> Result<query::Row, Stop> {
                let row = rows.get(slot)?;
                Ok(query::Row {
                    key: row.field(query::key()).map_err(|_| Stop::Domain(Denial))?,
                    content: row
                        .optional_aspect_field(query::content())
                        .map_err(|_| Stop::Domain(Denial))?
                        .map(|value| value.0),
                })
            };
            let old = change
                .before()
                .map(|slot| observe(before, slot))
                .transpose()?;
            let new = change
                .after()
                .map(|slot| observe(after, slot))
                .transpose()?;
            self.0.optional_changes.lock().unwrap().push((old, new));
            match (change.before(), change.after()) {
                (None, Some(slot)) => {
                    assert_eq!(self.source_summary(slot, after.get(slot)?, control)?, 1);
                    let count = match root {
                        Some(root) => nodes.get(root)?.count + 1,
                        None => 1,
                    };
                    root = Some(nodes.push(Node {
                        slot,
                        previous: root,
                        count,
                    })?);
                }
                (Some(_), None) => return Err(Stop::Domain(Denial)),
                (Some(_), Some(slot)) => {
                    assert_eq!(self.source_summary(slot, after.get(slot)?, control)?, 1);
                }
                (None, None) => return Err(Stop::Domain(Denial)),
            }
        }
        if after.is_empty() {
            assert!(root.is_none());
        }
        Ok(root)
    }
}
