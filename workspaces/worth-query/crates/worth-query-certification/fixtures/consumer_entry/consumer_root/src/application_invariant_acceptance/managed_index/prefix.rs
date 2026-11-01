//! Entry adapter: pure prefix mechanics borrow actual query rows and finite storage.
use super::{query, schema::Catalog};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_query_consumer_values::{
    insert_prefix_source, remove_prefix_source, PrefixIndexDenial, PrefixNode,
};
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_host::facade::primary_graph::*;

#[cfg(test)]
#[path = "prefix/cost_stages.rs"]
pub(super) mod cost_stages;
#[path = "prefix/installed_version.rs"]
mod installed_version;
#[path = "prefix/storage.rs"]
mod storage;
pub use installed_version::IncompatibleVersion;
#[path = "prefix/definition.rs"]
mod definition;
#[path = "prefix/predicate.rs"]
mod predicate;
#[path = "prefix/refinement.rs"]
mod refinement;
#[path = "prefix/structure.rs"]
pub(super) mod structure;
#[path = "prefix/summary.rs"]
mod summary;
#[cfg(test)]
#[path = "prefix/text_read.rs"]
mod text_read;
pub use definition::PrefixIndex;
const OBSERVED_COORDINATES: usize = 8;
pub type Node = PrefixNode<WorthQueryManagedIndexNodeId, WorthQueryManagedIndexSourceSlot>;
pub type Stop = WorthQueryManagedIndexProviderStop<PrefixIndexDenial>;
fn text_denial(cause: WorthQueryManagedIndexTextReadDenial) -> Stop {
    match cause {
        WorthQueryManagedIndexTextReadDenial::Controlled(cause) => Stop::Controlled(cause),
        WorthQueryManagedIndexTextReadDenial::Projection(_) => {
            Stop::Domain(PrefixIndexDenial::InvalidTopology)
        }
    }
}
#[derive(Default)]
pub struct Calls {
    #[cfg(test)]
    pub omit_refinement: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub reject_refinement: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub after_refinement: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    #[cfg(test)]
    pub predicate_regions: AtomicUsize,
    #[cfg(test)]
    pub(super) cost_stages: cost_stages::CostStages,
    #[cfg(test)]
    pub query_read_first: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub query_region_reads: AtomicUsize,
    #[cfg(test)]
    pub summary_repetitions: AtomicUsize,
    #[cfg(test)]
    pub inspect_text: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub wrong_text_binding: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub text_reads: AtomicUsize,
    #[cfg(test)]
    pub after_text: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    #[cfg(test)]
    pub summary_read_first: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub(super) first_source: Mutex<Option<WorthQueryManagedIndexSourceSlot>>,
    #[cfg(test)]
    pub(super) after_second_match: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    #[cfg(test)]
    pub(super) repeat_candidates: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub(super) foreign_candidate: Mutex<Option<WorthQueryManagedIndexSourceSlot>>,
    pub matched_sources: AtomicUsize,
    pub successful_matches: AtomicUsize,
    pub summary_comparisons: AtomicUsize,
    #[cfg(test)]
    pub(super) structure_fault: Mutex<structure::Fault>,
    pub capacity: AtomicUsize,
    pub builds: AtomicUsize,
    pub probe_validations: AtomicUsize,
    pub updates: AtomicUsize,
    pub canonical_rows: AtomicUsize,
    // Consumer fixture observation, invoked only after a genuine first node write.
    pub after_first_node: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    pub canonical_coordinates: Mutex<[(u32, u64, u32); OBSERVED_COORDINATES]>,
}
impl Calls {
    pub fn totals(&self) -> (usize, usize, usize) {
        (
            self.capacity.load(Ordering::SeqCst),
            self.builds.load(Ordering::SeqCst),
            self.probe_validations.load(Ordering::SeqCst),
        )
    }
}
pub struct Provider(pub Arc<Calls>);
impl WorthQueryManagedIndexProvider<Catalog, PrefixIndex> for Provider {
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
    const IDENTITY: &'static str = PrefixIndex::PROVIDER;
    const VERSION: u32 = PrefixIndex::PROVIDER_VERSION;
    fn source_summary(
        &self,
        slot: WorthQueryManagedIndexSourceSlot,
        row: WorthQueryManagedIndexSourceRow<'_, Catalog, PrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, Stop> {
        control.checkpoint(1)?;
        #[cfg(test)]
        text_read::inspect(&self.0, &row)?;
        #[cfg(not(test))]
        let _ = row;
        Ok(worth_query_consumer_values::PrefixSourceBounds::point(slot))
    }
    fn merge_summaries(
        &self,
        parts: [Option<Self::Summary>; 3],
        sources: &WorthQueryManagedIndexSummarySources<'_, '_, Catalog, PrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, Stop> {
        #[cfg(test)]
        if self.0.summary_read_first.load(Ordering::SeqCst) {
            sources
                .get(
                    self.0
                        .first_source
                        .lock()
                        .unwrap()
                        .expect("the build observed a genuine slot"),
                )?
                .with_text(query::name(), |_| ())
                .map_err(text_denial)?;
        }
        #[cfg(test)]
        self.0.cost_stages.record(cost_stages::Stage::MergeBegin);
        let result = summary::merge(parts, sources, control, &self.0);
        #[cfg(test)]
        self.0.cost_stages.record(cost_stages::Stage::MergeEnd);
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
        sources: &WorthQueryManagedIndexSummarySources<'_, '_, Catalog, PrefixIndex>,
        probe: &query::Probe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        self.0.summary_comparisons.fetch_add(1, Ordering::SeqCst);
        #[cfg(test)]
        self.0.cost_stages.record(cost_stages::Stage::BoundsBegin);
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
                sources
                    .get(
                        self.0
                            .first_source
                            .lock()
                            .unwrap()
                            .expect("genuine build-issued source"),
                    )?
                    .with_text(query::name(), |_| ())
                    .map_err(text_denial)?;
                self.0.query_region_reads.fetch_add(1, Ordering::SeqCst);
            }
        }
        #[cfg(test)]
        self.0.cost_stages.record(cost_stages::Stage::BoundsEnd);
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
        sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, PrefixIndex>,
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
        if *self.0.structure_fault.lock().unwrap() == structure::Fault::DisconnectedCycle {
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
        before: &WorthQueryManagedIndexSourceRows<'_, Catalog, PrefixIndex>,
        after: &WorthQueryManagedIndexSourceRows<'_, Catalog, PrefixIndex>,
        changes: &[WorthQueryManagedIndexSourceChange],
        mut root: Option<WorthQueryManagedIndexNodeId>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        self.0.updates.fetch_add(1, Ordering::SeqCst);
        #[cfg(test)]
        self.0.cost_stages.record(cost_stages::Stage::UpdateBegin);
        #[cfg(test)]
        if *self.0.structure_fault.lock().unwrap() == structure::Fault::DuplicateChild {
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
        self.0.cost_stages.record(cost_stages::Stage::UpdateEnd);
        Ok(root)
    }
    fn query(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, Catalog, PrefixIndex>,
        _nodes: &WorthQueryManagedIndexNodes<Node>,
        probe: &query::Probe,
        candidates: &mut WorthQueryManagedIndexCandidateWriter<'_>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        self.validate_probe(probe, control)?;
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
        _probe: &query::Probe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        self.0.probe_validations.fetch_add(1, Ordering::SeqCst);
        control.checkpoint(1).map_err(Into::into)
    }
    fn matches_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, Catalog, PrefixIndex>,
        probe: &query::Probe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        let order = row
            .with_text(query::name(), |name| {
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
