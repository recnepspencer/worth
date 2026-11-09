//! Bounded, read-only evidence of the source witness consumed by native laws.
use crate::identity::data::KindId;
use crate::transactions::data::EntityReference;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationLawSourceWitness {
    pub relation_kind: KindId,
    /// Complete validation reads the remaining source population on demand;
    /// these collections describe only explicitly prepared coordinates.
    pub all_visible_sources: bool,
    pub sources: BTreeSet<EntityReference>,
    pub successors: BTreeMap<EntityReference, Vec<EntityReference>>,
}
#[derive(Debug, Default)]
pub struct RelationLawSourceObservation {
    pub witnesses: Vec<RelationLawSourceWitness>,
    pub omitted: usize,
}
struct Recorder {
    remaining: usize,
    remaining_key_bytes: u64,
    observation: RelationLawSourceObservation,
}
thread_local! {
    static RECORDER: RefCell<Option<Recorder>> = const { RefCell::new(None) };
}
struct Restore(Option<Recorder>);
impl Drop for Restore {
    fn drop(&mut self) {
        RECORDER.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}
/// Bound witness headers and source/edge coordinates by `maximum_members`.
/// Copied authored-key backing has a separate bound of that many reference
/// layouts; a row exceeding either bound is omitted before cloning.
/// The returned diagnostic is never accepted as validation or graph authority.
pub fn observe_relation_law_sources<T>(
    maximum_members: usize,
    operation: impl FnOnce() -> T,
) -> (T, RelationLawSourceObservation) {
    let restore = Restore(RECORDER.with(|slot| {
        slot.replace(Some(Recorder {
            remaining: maximum_members,
            remaining_key_bytes: (maximum_members as u64)
                .saturating_mul(std::mem::size_of::<EntityReference>() as u64),
            observation: Default::default(),
        }))
    }));
    let value = operation();
    let observation =
        RECORDER.with(|slot| std::mem::take(&mut slot.borrow_mut().as_mut().unwrap().observation));
    drop(restore);
    (value, observation)
}
pub(crate) fn record(
    kind: KindId,
    all_visible_sources: bool,
    sources: &BTreeSet<EntityReference>,
    successors: &BTreeMap<EntityReference, Vec<EntityReference>>,
) {
    RECORDER.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(recorder) = slot.as_mut() else {
            return;
        };
        let members = successors.values().fold(
            1_usize
                .saturating_add(sources.len())
                .saturating_add(successors.len()),
            |n, targets| n.saturating_add(targets.len()),
        );
        if members > recorder.remaining {
            recorder.observation.omitted = recorder.observation.omitted.saturating_add(1);
            return;
        }
        let key_bytes = sources
            .iter()
            .chain(successors.keys())
            .chain(successors.values().flatten())
            .fold(0_u64, |total, reference| {
                total.saturating_add(match reference {
                    EntityReference::Existing(_) => 0,
                    EntityReference::Created(created) => {
                        created.client_key.owned_allocation_capacity_bytes()
                    }
                })
            });
        if key_bytes > recorder.remaining_key_bytes {
            recorder.observation.omitted = recorder.observation.omitted.saturating_add(1);
            return;
        }
        recorder.remaining_key_bytes -= key_bytes;
        recorder.remaining -= members;
        recorder
            .observation
            .witnesses
            .push(RelationLawSourceWitness {
                relation_kind: kind,
                all_visible_sources,
                sources: sources.clone(),
                successors: successors.clone(),
            });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_witnesses_consume_capacity_and_unwinding_restores_the_outer_recorder() {
        let sources = BTreeSet::new();
        let successors = BTreeMap::new();
        let (_, observation) = observe_relation_law_sources(1, || {
            let _ = std::panic::catch_unwind(|| {
                observe_relation_law_sources(1, || {
                    record(KindId(7), false, &sources, &successors);
                    panic!("interrupted diagnostic");
                })
            });
            record(KindId(9), false, &sources, &successors);
            record(KindId(10), false, &sources, &successors);
        });
        assert_eq!(
            observation.witnesses,
            vec![RelationLawSourceWitness {
                relation_kind: KindId(9),
                all_visible_sources: false,
                sources,
                successors
            }]
        );
        assert!(observation.omitted > 0);
        RECORDER.with(|slot| assert!(slot.borrow().is_none()));
    }
}
