//! One controlled prefix provider; schema owners supply only their typed field loan.
use super::prefix_observation::{Calls, OBSERVED_COORDINATES};
use super::prefix_probe::Probe;
use super::{prefix_cost_stages, structure_fault};
use std::sync::{atomic::Ordering, Arc};
use worth_query_consumer_values::{
    insert_prefix_source, remove_prefix_source, PrefixIndexDenial, PrefixNode,
};
use worth_query_decl::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_host::facade::primary_graph::*;
mod source_loan;
pub(in crate::application_invariant_acceptance) use source_loan::PrefixSourceIndex;
mod predicate;
mod refinement;
mod storage;
mod structure;
mod summary;
pub type Node = PrefixNode<WorthQueryManagedIndexNodeId, WorthQueryManagedIndexSourceSlot>;
pub type Stop = WorthQueryManagedIndexProviderStop<PrefixIndexDenial>;
pub(in crate::application_invariant_acceptance) fn text_denial(
    cause: WorthQueryManagedIndexTextReadDenial,
) -> Stop {
    match cause {
        WorthQueryManagedIndexTextReadDenial::Controlled(cause) => Stop::Controlled(cause),
        WorthQueryManagedIndexTextReadDenial::Projection(_) => {
            Stop::Domain(PrefixIndexDenial::InvalidTopology)
        }
    }
}
pub struct Provider(pub Arc<Calls>);
impl<S: ApplicationSchema, I: PrefixSourceIndex<S>> WorthQueryManagedIndexProvider<S, I>
    for Provider
{
    fn decompose(
        &self,
        node: Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<WorthQueryManagedIndexNodeRegion, Stop> {
        structure::describe(&self.0, node, control)
    }
    type Node = Node;
    type Summary =
        worth_query_consumer_values::PrefixSourceBounds<WorthQueryManagedIndexSourceSlot>;
    type Denial = PrefixIndexDenial;
    const IDENTITY: &'static str = I::PROVIDER;
    const VERSION: u32 = I::PROVIDER_VERSION;
    fn source_summary(
        &self,
        slot: WorthQueryManagedIndexSourceSlot,
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, Stop> {
        control.checkpoint(1)?;
        #[cfg(test)]
        I::inspect_text(&self.0, &row)?;
        #[cfg(not(test))]
        let _ = row;
        Ok(worth_query_consumer_values::PrefixSourceBounds::point(slot))
    }
    fn merge_summaries(
        &self,
        parts: [Option<Self::Summary>; 3],
        sources: &WorthQueryManagedIndexSummarySources<'_, '_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, Stop> {
        #[cfg(test)]
        if self.0.summary_read_first.load(Ordering::SeqCst) {
            I::with_name(
                &sources.get(
                    self.0
                        .first_source
                        .lock()
                        .unwrap()
                        .expect("the build observed a genuine slot"),
                )?,
                |_| (),
            )
            .map_err(text_denial)?;
        }
        #[cfg(test)]
        self.0
            .cost_stages
            .record(prefix_cost_stages::Stage::MergeBegin);
        let result = summary::merge(parts, sources, control, &self.0);
        #[cfg(test)]
        self.0
            .cost_stages
            .record(prefix_cost_stages::Stage::MergeEnd);
        result
    }
    fn node_summary(
        &self,
        node: Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, Stop> {
        control.checkpoint(1)?;
        Ok(node.bounds)
    }
    fn summary_may_match(
        &self,
        bounds: Self::Summary,
        sources: &WorthQueryManagedIndexSummarySources<'_, '_, S, I>,
        probe: &Probe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        self.0.summary_comparisons.fetch_add(1, Ordering::SeqCst);
        #[cfg(test)]
        self.0
            .cost_stages
            .record(prefix_cost_stages::Stage::BoundsBegin);
        let result = summary::may_match(bounds, sources, probe, control, &self.0)?;
        #[cfg(test)]
        {
            for _ in 0..self.0.summary_repetitions.load(Ordering::SeqCst) {
                assert_eq!(
                    summary::may_match(bounds, sources, probe, control, &self.0)?,
                    result
                );
            }
            if self.0.query_read_first.load(Ordering::SeqCst) {
                I::with_name(
                    &sources.get(
                        self.0
                            .first_source
                            .lock()
                            .unwrap()
                            .expect("genuine build-issued source"),
                    )?,
                    |_| (),
                )
                .map_err(text_denial)?;
                self.0.query_region_reads.fetch_add(1, Ordering::SeqCst);
            }
        }
        #[cfg(test)]
        self.0
            .cost_stages
            .record(prefix_cost_stages::Stage::BoundsEnd);
        Ok(result)
    }
    fn node_capacity(
        &self,
        source_count: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        self.0.capacity.fetch_add(1, Ordering::SeqCst);
        Ok(source_count)
    }
    fn build(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, S, I>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.builds.fetch_add(1, Ordering::SeqCst);
        // This assertion uses genuine full-GEN row identities, never their labels.
        // The independent ordinary Source query deliberately has another order.
        let mut previous = None;
        for (ordinal, slot) in sources.slots().enumerate() {
            let slot = slot?;
            #[cfg(test)]
            if ordinal == 0 {
                *self.0.first_source.lock().unwrap() = Some(slot);
            }
            let row = sources.get(slot)?;
            let actual = row.entity_id();
            assert!(previous.is_none_or(|previous| previous < actual));
            previous = Some(actual);
            if ordinal < OBSERVED_COORDINATES {
                self.0.canonical_coordinates.lock().unwrap()[ordinal] = (
                    actual.partition_value(),
                    actual.local_slot_value(),
                    actual.generation_value(),
                );
            }
        }
        self.0.canonical_rows.store(sources.len(), Ordering::SeqCst);
        #[cfg(test)]
        if *self.0.structure_fault.lock().unwrap() == structure_fault::Fault::DisconnectedCycle {
            return structure::cyclic_sources(sources, nodes);
        }
        let mut storage = storage::Build {
            sources,
            nodes,
            control,
            after_first_node: &self.0.after_first_node,
        };
        let mut root = None;
        for slot in sources.slots() {
            root = Some(insert_prefix_source(root, slot?, &mut storage)?);
        }
        Ok(root)
    }
    fn update(
        &self,
        before: &WorthQueryManagedIndexSourceRows<'_, S, I>,
        after: &WorthQueryManagedIndexSourceRows<'_, S, I>,
        changes: &[WorthQueryManagedIndexSourceChange],
        mut root: Option<WorthQueryManagedIndexNodeId>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.updates.fetch_add(1, Ordering::SeqCst);
        #[cfg(test)]
        self.0
            .cost_stages
            .record(prefix_cost_stages::Stage::UpdateBegin);
        #[cfg(test)]
        if *self.0.structure_fault.lock().unwrap() == structure_fault::Fault::DuplicateChild {
            // A genuine value-only write of the exact predecessor node. Its
            // first child is therefore a retained edge when describe duplicates
            // it. No pure edit can replace that child before the hostile check.
            if let Some(root) = root {
                let node = nodes.get(root)?;
                nodes.replace(root, node)?;
            }
            return Ok(root);
        }
        // All removals see the committed value order; only then may proposed
        // values participate. The pages belong to this isolated staged edit.
        for change in changes {
            if let Some(slot) = change.before() {
                root = remove_prefix_source(
                    root,
                    slot,
                    &mut storage::Build {
                        sources: before,
                        nodes,
                        control,
                        after_first_node: &self.0.after_first_node,
                    },
                )?;
            }
        }
        for change in changes {
            if let Some(slot) = change.after() {
                root = Some(insert_prefix_source(
                    root,
                    slot,
                    &mut storage::Build {
                        sources: after,
                        nodes,
                        control,
                        after_first_node: &self.0.after_first_node,
                    },
                )?);
            }
        }
        #[cfg(test)]
        self.0
            .cost_stages
            .record(prefix_cost_stages::Stage::UpdateEnd);
        Ok(root)
    }
    fn query(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, S, I>,
        _nodes: &WorthQueryManagedIndexNodes<Node>,
        probe: &Probe,
        candidates: &mut WorthQueryManagedIndexCandidateWriter<'_>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        <Self as WorthQueryManagedIndexProvider<S, I>>::validate_probe(self, probe, control)?;
        assert!(candidates.is_empty());
        #[cfg(test)]
        if let Some(slot) = *self.0.foreign_candidate.lock().unwrap() {
            candidates.emit(slot)?;
        }
        let mut count = 0;
        for ordinal in (0..sources.len()).rev() {
            let (slot, row) = sources.at(ordinal)?;
            if self.matches_source(row, probe, control)? {
                candidates.emit(slot)?;
                #[cfg(test)]
                if self.0.repeat_candidates.load(Ordering::SeqCst) {
                    candidates.emit(slot)?;
                    candidates.emit(slot)?;
                }
                count += 1;
                assert_eq!(candidates.len(), count);
            }
        }
        assert_eq!(candidates.is_empty(), count == 0);
        Ok(())
    }
    fn validate_probe(
        &self,
        _probe: &Probe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        self.0.probe_validations.fetch_add(1, Ordering::SeqCst);
        control.checkpoint(1).map_err(Into::into)
    }
    fn matches_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        probe: &Probe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        let order = I::with_name(&row, |name| {
            worth_query_consumer_values::prefix_order(name, &probe.prefix, |units| {
                control.checkpoint(units).map_err(Stop::Controlled)
            })
        })
        .map_err(text_denial)??;
        let previous = self.0.matched_sources.fetch_add(1, Ordering::SeqCst);
        #[cfg(test)]
        if previous == 1 {
            let observe = self.0.after_second_match.lock().unwrap().take();
            if let Some(observe) = observe {
                observe();
            }
        }
        #[cfg(not(test))]
        let _ = previous;
        let matches = order == worth_query_consumer_values::PrefixOrder::Matches;
        if matches {
            self.0.successful_matches.fetch_add(1, Ordering::SeqCst);
        }
        Ok(matches)
    }
}
