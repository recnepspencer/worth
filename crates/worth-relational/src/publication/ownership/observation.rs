//! Thread-scoped read-only contacts; this recorder grants no mutation standing.
use super::read::roster::CurrentPublicationRoster;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::identity::data::EntityId;
use std::cell::RefCell;
/// Actual private rows decoded by a one-hop custody read, with their source loci.
#[derive(Debug)]
pub struct CurrentOwnershipContactsForTest {
    pub publication_rows: Vec<(EntityId, EntityId)>,
    pub presence_rows: Vec<(EntityId, EntityId)>,
    /// Each incidence is charged before its current metadata is read.
    pub scope_links: usize,
    pub position_links: usize,
    /// Actual meter deltas for the per-position incidence checkpoints.
    pub position_work: u64,
}

impl CurrentOwnershipContactsForTest {
    pub(super) fn from_current(
        current: &CurrentPublicationRoster,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Self {
        let storage = basis
            .inner
            .root
            .schema_authority()
            .registry()
            .current_publication_storage
            .as_ref();
        Self {
            position_work: current.observed_position_work,
            publication_rows: current
                .publications
                .iter()
                .chain(&current.codec_retired)
                .map(|r| (r.value.origin.meaning.source_scope, r.entity))
                .collect(),
            presence_rows: current
                .positions
                .iter()
                .map(|(id, p)| (p.meaning.source_scope, *id))
                .collect(),
            scope_links: storage.map_or(0, |s| {
                current
                    .links
                    .iter()
                    .filter(|l| l.kind == s.scope_link)
                    .count()
            }),
            position_links: storage.map_or(0, |s| {
                current
                    .links
                    .iter()
                    .filter(|l| l.kind == s.position_link)
                    .count()
            }),
        }
    }
}

type Observer = Box<dyn Fn(CurrentOwnershipContactsForTest)>;
thread_local! {
    static OBSERVER: RefCell<Option<Observer>> = const { RefCell::new(None) };
}
pub(super) struct Guard(Option<Observer>);
impl Drop for Guard {
    fn drop(&mut self) {
        OBSERVER.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}
pub(super) fn install(observer: Observer) -> Guard {
    Guard(OBSERVER.with(|slot| slot.replace(Some(observer))))
}
pub(super) fn record(
    current: &CurrentPublicationRoster,
    basis: &crate::branch::basis::AdmittedRelationalBranchBasis,
) {
    OBSERVER.with(|slot| {
        if let Some(observer) = slot.borrow().as_ref() {
            observer(CurrentOwnershipContactsForTest::from_current(
                current, basis,
            ));
        }
    });
}

/// Sample only for an installed recorder and an authentic accounted request.
pub(super) fn work(context: &worth_execution::MapKernelContext<'_, '_>) -> Option<u64> {
    OBSERVER
        .with(|slot| slot.borrow().is_some())
        .then(|| context.has_accounting().then(|| context.charged_work()))
        .flatten()
}
