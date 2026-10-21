//! Lazy semantic settlement/ordinal merge over physical projection bindings.
use super::super::super::fact_key::FactPostingKey;
use super::super::super::logical_marking::WorthQueryNativeMarkingWork;
use super::super::super::mark_state::FactPosting;
use super::super::super::mark_state::MarkState;
use im::OrdMap;
use std::ops::Bound;
use std::sync::Arc;
use worth_relational::facade::mvcc::CompanionPreflightStop;

pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) struct LogicalPostings
{
    ready: OrdMap<FactPosting, u64>,
}
impl LogicalPostings {
    /// One cursor per matched physical projection, never all matching ordinals.
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn prepare(
        state: &MarkState,
        key: &FactPostingKey,

        counts: &mut WorthQueryNativeMarkingWork,
    ) -> Result<Option<Self>, CompanionPreflightStop> {
        let mut ready = OrdMap::new();
        let Some(groups) = state.postings.get(key) else {
            return Ok(None);
        };
        {
            for id in groups {
                let binding = state
                    .posting_bindings
                    .get(id)
                    .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
                let Some(binding) = binding.settlements() else {
                    continue;
                };

                let ordinals = binding
                    .projection
                    .ordinals()
                    .get(key)
                    .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;

                let (Some(settlement), Some(ordinal)) =
                    (binding.settlements.get_min(), ordinals.get_min())
                else {
                    return Err(CompanionPreflightStop::SelectedSourceMismatch);
                };
                observed_match(counts)?;

                ready.insert(
                    FactPosting {
                        settlement: Arc::clone(settlement),
                        ordinal: *ordinal,
                    },
                    *id,
                );
            }
        }
        Ok(Some(Self { ready }))
    }
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn next(
        &mut self,
        state: &MarkState,
        key: &FactPostingKey,

        counts: &mut WorthQueryNativeMarkingWork,
    ) -> Result<Option<FactPosting>, CompanionPreflightStop> {
        let Some((posting, id)) = self
            .ready
            .get_min()
            .map(|(posting, id)| (posting.clone(), *id))
        else {
            return Ok(None);
        };

        self.ready.remove(&posting);

        let binding = state
            .posting_bindings
            .get(&id)
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
        let binding = binding
            .settlements()
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;

        let ordinals = binding
            .projection
            .ordinals()
            .get(key)
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;

        let following = if let Some(ordinal) = ordinals
            .range((Bound::Excluded(posting.ordinal), Bound::Unbounded))
            .next()
        {
            Some(FactPosting {
                settlement: Arc::clone(&posting.settlement),
                ordinal: *ordinal,
            })
        } else {
            binding
                .settlements
                .range((
                    Bound::Excluded(Arc::clone(&posting.settlement)),
                    Bound::Unbounded,
                ))
                .next()
                .map(|settlement| FactPosting {
                    settlement: Arc::clone(settlement),
                    ordinal: *ordinals.get_min().expect("nonempty projection"),
                })
        };
        if let Some(following) = following {
            observed_match(counts)?;

            if self.ready.insert(following, id).is_some() {
                return Err(CompanionPreflightStop::SelectedSourceMismatch);
            }
        }
        Ok(Some(posting))
    }
}

// A queued pair is an actual exact binding match, even when a later request
// stop precedes marking it. Each cursor discovers each pair exactly once.
fn observed_match(counts: &mut WorthQueryNativeMarkingWork) -> Result<(), CompanionPreflightStop> {
    counts.matched_fact_postings = counts
        .matched_fact_postings
        .checked_add(1)
        .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
    Ok(())
}
