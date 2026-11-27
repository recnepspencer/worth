//! Public #358: a transported row's consumed evidence, re-identified at the
//! snapshot where readmission proved it, kept per physical branch. The
//! provenance field is never written; a reader without an entry for its
//! branch reads provenance, which is unaligned there and only loses reuse.

use std::sync::Arc;

use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::{RecordedOutput, RecordedOutputMutable};
use crate::domain_computation::primary_graph::invariant_projection::consumed_output::verification::at_observation::reads_at_or_after;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;

/// At most this many branches keep a re-rooted set; the oldest is evicted
/// and its branch falls back to provenance.
const BRANCHES: usize = 8;

/// The evidence readmission re-identified, rooted at the snapshot where it
/// proved the whole chain current.
#[derive(Clone, Debug)]
pub struct RerootedConsumed {
    pub root: Arc<PositionedRelationalSnapshot>,
    pub evidence: Arc<[ConsumedOutputEvidence]>,
}

impl RerootedConsumed {
    fn same_branch(&self, selected: &PositionedRelationalSnapshot) -> bool {
        self.root.runtime_instance_id() == selected.runtime_instance_id()
            && self.root.branch_id() == selected.branch_id()
    }
}

/// A transported row's coordinate and its re-rooted sets, captured together
/// by carriers built before a snapshot is selected. `None` for performed rows.
#[derive(Clone, Debug)]
pub struct TransportedView {
    pub coordinate: crate::domain_computation::primary_graph::output_lineage::SelectionCoordinate,
    pub rerooted: Arc<[RerootedConsumed]>,
}

impl TransportedView {
    /// The coordinate of a carrier's transported row, if any.
    pub fn coordinate(
        view: Option<&Self>,
    ) -> Option<&crate::domain_computation::primary_graph::output_lineage::SelectionCoordinate>
    {
        view.map(|view| &view.coordinate)
    }

    /// The evidence a carrier serves at `selected`: the re-rooted set proven
    /// on its branch at or before it, else the carrier's provenance.
    pub fn consumed_outputs_at(
        view: Option<&Self>,
        provenance: &Arc<[ConsumedOutputEvidence]>,
        selected: &PositionedRelationalSnapshot,
    ) -> Arc<[ConsumedOutputEvidence]> {
        Arc::clone(
            view.and_then(|view| serving(&view.rerooted, selected))
                .unwrap_or(provenance),
        )
    }
}

/// Carriers keep their provenance field and serve readers through these.
macro_rules! transported_carrier {
    ($carrier:ty) => {
        impl $carrier {
            /// The consumed evidence that answers at `selected` (public #358).
            pub fn consumed_outputs_at(
                &self,
                selected: &PositionedRelationalSnapshot,
            ) -> Arc<[ConsumedOutputEvidence]> {
                TransportedView::consumed_outputs_at(
                    self.transported.as_ref(),
                    &self.consumed_outputs,
                    selected,
                )
            }

            /// The selection coordinate of a transported row.
            pub fn historical(
                &self,
            ) -> Option<&crate::domain_computation::primary_graph::output_lineage::SelectionCoordinate>
            {
                TransportedView::coordinate(self.transported.as_ref())
            }
        }
    };
}
transported_carrier!(crate::domain_computation::primary_graph::output_lineage::WorthQueryCurrentOutputCandidate);
transported_carrier!(crate::domain_computation::primary_graph::output_lineage::current_output::observation_reader::ObservedRetainedOutput);
transported_carrier!(crate::domain_computation::primary_graph::output_lineage::current_output::RetainedOutputCurrentnessRead);

impl crate::domain_computation::primary_graph::output_lineage::WorthQueryCurrentOutputCandidate {
    /// See `RecordedOutput::consumed_outputs_for_aliases`.
    pub fn consumed_outputs_for_aliases(&self) -> Arc<[ConsumedOutputEvidence]> {
        Arc::clone(
            self.transported
                .as_ref()
                .and_then(|view| view.rerooted.last())
                .map_or(&self.consumed_outputs, |entry| &entry.evidence),
        )
    }
}

fn serving<'a>(
    entries: &'a [RerootedConsumed],
    selected: &PositionedRelationalSnapshot,
) -> Option<&'a Arc<[ConsumedOutputEvidence]>> {
    entries
        .iter()
        .find(|entry| entry.same_branch(selected) && reads_at_or_after(selected, &entry.root))
        .map(|entry| &entry.evidence)
}

impl RecordedOutputMutable {
    pub(super) fn no_rerooted() -> Arc<[RerootedConsumed]> {
        Arc::from([])
    }
}

impl RecordedOutput {
    fn rerooted(&self) -> Arc<[RerootedConsumed]> {
        Arc::clone(
            &self
                .mutable
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .rerooted,
        )
    }

    /// The consumed evidence that answers at `selected`. A performed row has
    /// no re-rooted set and answers with its provenance, as before.
    pub fn consumed_outputs_at(
        &self,
        selected: &PositionedRelationalSnapshot,
    ) -> Arc<[ConsumedOutputEvidence]> {
        if self.historical_selection.is_none() {
            return Arc::clone(&self.consumed_outputs);
        }
        let rerooted = self.rerooted();
        Arc::clone(serving(&rerooted, selected).unwrap_or(&self.consumed_outputs))
    }

    pub fn transported_view(&self) -> Option<TransportedView> {
        Some(TransportedView {
            coordinate: Arc::clone(self.historical_selection.as_ref()?),
            rerooted: self.rerooted(),
        })
    }

    /// Re-rooting never changes whether a row consumed anything.
    pub fn has_consumed_outputs(&self) -> bool {
        !self.consumed_outputs.is_empty()
    }

    /// Supporting aliases need no snapshot: re-identification keeps each
    /// upstream's source and partition, so every set names the same aliases.
    /// The latest set's rows are live at the current coordinate.
    pub fn consumed_outputs_for_aliases(&self) -> Arc<[ConsumedOutputEvidence]> {
        self.rerooted().last().map_or_else(
            || Arc::clone(&self.consumed_outputs),
            |entry| Arc::clone(&entry.evidence),
        )
    }

    /// Keep the earliest root proven on a branch: it serves every later
    /// reader there. A new branch is appended; past `BRANCHES` the oldest
    /// branch is evicted. Copy-on-write, so carriers snapshot it by `Arc`.
    pub fn install_rerooted(&self, entry: RerootedConsumed) {
        let mut row = self
            .mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut entries = row.rerooted.to_vec();
        match entries
            .iter()
            .position(|existing| existing.same_branch(&entry.root))
        {
            Some(at) if reads_at_or_after(&entry.root, &entries[at].root) => return,
            Some(at) => {
                entries.remove(at);
            }
            None if entries.len() == BRANCHES => {
                entries.remove(0);
            }
            None => {}
        }
        entries.push(entry);
        row.rerooted = Arc::from(entries);
    }
}
