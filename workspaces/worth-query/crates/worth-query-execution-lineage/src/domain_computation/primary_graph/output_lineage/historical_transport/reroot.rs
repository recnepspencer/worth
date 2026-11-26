//! Public #358: a readmitted consumer's evidence names the rows its own
//! selection commit transported with it, never the abandoned line's. Pure:
//! it reads immutable rows under the lineage lock and writes nothing; the
//! returned plan is posted and installed only after ownership is checked.
use super::super::invalidation::reestablishment::readmission::ReadmittedNode;
use super::super::recorded_cell::SharedRecordedOutputCell;
use super::super::*;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::invariant_projection::EvidenceView;
use worth_relational::facade::mvcc::RelationalRequiredOutputPublication;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

/// Everything readmission posts and installs for one consumer, computed
/// before any write. Any miss while building it refuses the readmission.
pub struct RerootPlan {
    /// The rows to post, bottom-up; the consumer is last.
    pub nodes: Vec<ReadmittedNode>,
    /// Each re-identified row's committed publication, whose ownership the
    /// caller checks at the head before posting.
    pub publications: Vec<RelationalRequiredOutputPublication>,
    installs: Vec<(SharedRecordedOutputCell, Arc<[ConsumedOutputEvidence]>)>,
    root: Arc<PositionedRelationalSnapshot>,
}

impl RerootPlan {
    /// Install each re-rooted set on its row, after the chain was posted.
    pub fn install(self) {
        for (cell, evidence) in self.installs {
            if let Some(row) = cell.get() {
                row.install_rerooted(RerootedConsumed {
                    root: Arc::clone(&self.root),
                    evidence,
                });
            }
        }
    }
}

struct Walk<'a> {
    lineage: &'a WorthQueryApplicationOutputLineage,
    coordinate: ProductCoordinate,
    selection: &'a HistoricalSelectionCoordinate,
    root: Arc<PositionedRelationalSnapshot>,
    nodes: Vec<ReadmittedNode>,
    publications: Vec<RelationalRequiredOutputPublication>,
    installs: Vec<(SharedRecordedOutputCell, Arc<[ConsumedOutputEvidence]>)>,
    done: BTreeMap<Arc<RecordedSettlementIdentity>, ConsumedOutputEvidence>,
}

impl WorthQueryApplicationOutputLineage {
    /// Re-identify `candidate`'s whole consumed chain at `selected`, after
    /// `is_current` proved it there. `None` on any miss.
    pub fn reroot_consumed(
        &self,
        candidate: &current_output::retained_candidates::WorthQueryRetainedOutputCandidate,
        selected: &PositionedRelationalSnapshot,
    ) -> Option<RerootPlan> {
        let selection = candidate.historical_selection.as_ref()?;
        let consumer = self.cell_of(&candidate.settlement_identity)?;
        let facts = candidate.observed_source_facts.clone()?;
        let witness = Arc::clone(candidate.native_output_witness.as_ref()?);
        witness.get()?;
        let mut walk = Walk {
            lineage: self,
            coordinate: candidate.settlement_identity.coordinate(),
            selection,
            root: Arc::new(selected.clone()),
            nodes: Vec::new(),
            publications: Vec::new(),
            installs: Vec::new(),
            done: BTreeMap::new(),
        };
        let evidence = walk.all(&candidate.consumed_outputs)?;
        walk.nodes.push(ReadmittedNode {
            identity: Arc::clone(&candidate.settlement_identity),
            facts,
            witness,
            upstream: identities(&evidence),
            historical: Some(Arc::clone(selection)),
        });
        walk.installs.push((consumer, evidence));
        Some(RerootPlan {
            nodes: walk.nodes,
            publications: walk.publications,
            installs: walk.installs,
            root: walk.root,
        })
    }

    fn cell_of(&self, identity: &Arc<RecordedSettlementIdentity>) -> Option<SharedRecordedOutputCell> {
        let coordinate = identity.coordinate();
        let cell = self
            .by_source
            .get(identity.source())?
            .get(&coordinate.occurrence)?
            .get(&coordinate.generation)?
            .get(identity.slot())?;
        Arc::ptr_eq(&cell.get()?.settlement_identity, identity)
            .then(|| SharedRecordedOutputCell::from_arc(Arc::clone(cell)))
    }
}

impl Walk<'_> {
    fn all(&mut self, upstream: &[ConsumedOutputEvidence]) -> Option<Arc<[ConsumedOutputEvidence]>> {
        upstream
            .iter()
            .map(|evidence| self.one(evidence))
            .collect::<Option<Vec<_>>>()
            .map(Arc::from)
    }

    fn one(&mut self, evidence: &ConsumedOutputEvidence) -> Option<ConsumedOutputEvidence> {
        if let Some(done) = self.done.get(evidence.identity()) {
            return Some(done.clone());
        }
        let view = EvidenceView::from(evidence);
        let original_cell = self.lineage.cell_of(evidence.identity())?;
        let original = original_cell.get()?;
        let partition = original.source_partition_identity?;
        let cell = if evidence.identity().coordinate() == self.coordinate {
            original_cell.clone()
        } else {
            let slot = self.lineage.partition_index.at_generation(
                evidence.identity().source(),
                self.coordinate.occurrence,
                self.coordinate.generation,
                partition,
            )?;
            let cell = self
                .lineage
                .by_source
                .get(evidence.identity().source())?
                .get(&self.coordinate.occurrence)?
                .get(&self.coordinate.generation)?
                .get(slot)?;
            SharedRecordedOutputCell::from_arc(Arc::clone(cell))
        };
        let live = cell.get()?;
        let already = cell.ptr_eq(&original_cell);
        if !self.lineage.implementation_is_current(live)
            || live.verification_requirement().is_some()
            || !(already || self.same_commit(live.historical_selection.as_deref()?))
        {
            return None;
        }
        let (facts, witness) = content(live)?;
        let (original_facts, original_witness) = content(original)?;
        let same_witness = Arc::ptr_eq(&witness, &original_witness)
            || witness.get()?.same_output_as(original_witness.get()?).ok()?;
        let same_facts = facts.facts().same_backing(original_facts.facts())
            || facts.facts() == original_facts.facts();
        if !same_witness
            || !same_facts
            || live.producing_implementation != original.producing_implementation
            || live.producer_dependency_identity != original.producer_dependency_identity
            || identities(&live.consumed_outputs) != identities(view.upstream())
        {
            return None;
        }
        let upstream = self.all(view.upstream())?;
        self.publications.push(live.required_publication.clone()?);
        self.nodes.push(ReadmittedNode {
            identity: Arc::clone(&live.settlement_identity),
            facts: facts.clone(),
            witness: Arc::clone(&witness),
            upstream: identities(&upstream),
            historical: live.historical_selection.clone(),
        });
        if !upstream.is_empty() {
            self.installs.push((cell.clone(), Arc::clone(&upstream)));
        }
        let rerooted = ConsumedOutputEvidence::new(
            facts.computation(),
            Arc::clone(&live.settlement_identity),
            facts,
            upstream,
            None,
            witness,
            Arc::clone(&self.root),
            live.historical_selection.clone(),
        );
        self.done
            .insert(Arc::clone(evidence.identity()), rerooted.clone());
        Some(rerooted)
    }

    /// The same selection commit transported both rows: every row of one
    /// commit carries the same selected and restored observations.
    fn same_commit(&self, other: &HistoricalSelectionCoordinate) -> bool {
        let same = |left: &worth_relational::facade::mvcc::RelationalBranchObservation,
                    right: &worth_relational::facade::mvcc::RelationalBranchObservation| {
            left.descriptor() == right.descriptor() && left.identity() == right.identity()
        };
        same(&other.selected, &self.selection.selected)
            && same(&other.restored, &self.selection.restored)
    }
}

/// A row's comparable facts and sealed witness, as candidates select them.
fn content(
    row: &RecordedOutput,
) -> Option<(ComparableSourceFacts, Arc<OnceLock<SealedNativeOutputWitness>>)> {
    let facts = row.observed_source_facts()?.for_comparison()?;
    let witness = row
        .completed_computation_record()?
        .native_output_witness_cell()?;
    witness.get()?;
    Some((facts, Arc::clone(witness)))
}

fn identities(evidence: &[ConsumedOutputEvidence]) -> im::OrdSet<Arc<RecordedSettlementIdentity>> {
    evidence
        .iter()
        .map(|evidence| Arc::clone(evidence.identity()))
        .collect()
}
