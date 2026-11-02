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
fn content_stop(cause: WorthQueryManagedIndexContentReadDenial) -> Stop {
    match cause {
        WorthQueryManagedIndexContentReadDenial::Controlled(cause) => cause.into(),
        WorthQueryManagedIndexContentReadDenial::Projection(_) => Stop::Domain(Denial),
    }
}
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
        if self.0.swallowed_projection.load(Ordering::SeqCst) != 0 {
            let invalid = ApplicationQueryResultFieldRef::<
                query::SourceQuery,
                query::ContentSlot,
                Schema,
                Record,
                Samples,
                Values,
                ContentRefId,
                ReadWrite,
                NoEqualityPredicate,
                NoApplicationUnit,
            >::new("missing", Values::reference());
            let ignored = row.with_content(invalid, codec::GridFormat, |_| ());
            assert!(matches!(
                ignored,
                Err(WorthQueryManagedIndexContentReadDenial::Projection(_))
            ));
            return Ok(1);
        }
        row.with_content(query::content(), codec::GridFormat, |bytes| {
            assert!(bytes.len() >= 8);
        })
        .map_err(content_stop)?;
        self.0.content_reads.fetch_add(1, Ordering::SeqCst);
        Ok(1)
    }
    fn merge_summaries(
        &self,
        parts: [Option<usize>; 3],
        _: &WorthQueryManagedIndexSummarySources<'_, '_, Schema, Index>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(3)?;
        parts.into_iter().flatten().try_fold(0usize, |sum, n| {
            sum.checked_add(n).ok_or(Stop::Domain(Denial))
        })
    }
    fn node_summary(
        &self,
        node: Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        Ok(node.count)
    }
    fn summary_may_match(
        &self,
        _: usize,
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
        node: Node,
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
        _: &WorthQueryManagedIndexSourceRows<'_, Schema, Index>,
        after: &WorthQueryManagedIndexSourceRows<'_, Schema, Index>,
        changes: &[WorthQueryManagedIndexSourceChange],
        mut root: Option<WorthQueryManagedIndexNodeId>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.updates.fetch_add(1, Ordering::SeqCst);
        for change in changes {
            control.checkpoint(1)?;
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
impl WorthQueryManagedIndexRefinementProvider<Schema, Index> for Provider {
    const REFINEMENT_IDENTITY: &'static str = "content-source.minimum-sample";
    const REFINEMENT_VERSION: u32 = 1;
    fn validate_refinement_probe(
        &self,
        _: &u64,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        control.checkpoint(1).map_err(Into::into)
    }
    fn refine_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, Schema, Index>,
        minimum: &u64,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        self.0.exact_reads.fetch_add(1, Ordering::SeqCst);
        row.with_content(query::content(), codec::GridFormat, |payload| {
            let mut accepted = false;
            let (samples, remainder) = payload[8..].as_chunks::<8>();
            if !remainder.is_empty() {
                return Err(Stop::Domain(Denial));
            }
            for sample in samples {
                control.checkpoint(8)?;
                let value = i64::from_le_bytes(*sample);
                accepted |= i128::from(value) >= i128::from(*minimum);
            }
            Ok(accepted)
        })
        .map_err(content_stop)?
    }
}
