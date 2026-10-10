//! A constructed writer interleaving at the owned-publication continuation.
use super::InvalidationEditAdmission;
use std::cell::RefCell;
thread_local! {
    static PUBLICATIONS: RefCell<std::collections::BTreeMap<(u64, worth_relational::facade::identity::EntityId), u64>> = const { RefCell::new(std::collections::BTreeMap::new()) };
    static PUBLICATION_RECORDS: RefCell<std::collections::BTreeMap<(u64, worth_relational::facade::identity::EntityId), u64>> = const { RefCell::new(std::collections::BTreeMap::new()) };
    static EXHAUST_AFTER_COMMIT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static READ_DEBITS: RefCell<Vec<(u64, u64)>> = const { RefCell::new(Vec::new()) };
    static REQUEST_NAVIGATION: RefCell<Vec<(u64,u64)>> = const { RefCell::new(Vec::new()) };
    static ADMISSION_WORK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    static REQUEST_WORK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}
pub(super) fn observe_request_work(admission: &InvalidationEditAdmission) {
    let work = admission.charged_work();
    REQUEST_NAVIGATION.with(|requests| {
        requests.borrow_mut().push((
            admission.charged_navigation(),
            admission.charged_ordered_operations(),
        ))
    });
    REQUEST_WORK.with(|requests| requests.borrow_mut().push(work));
}
pub(super) fn exhaust_after_commit(
    admission: &mut InvalidationEditAdmission,
    published: bool,
    recorded: bool,
    runtime: u64,
    root: worth_relational::facade::identity::EntityId,
) {
    if published {
        PUBLICATIONS.with(|publications| {
            *publications
                .borrow_mut()
                .entry((runtime, root))
                .or_default() += 1;
        });
    }
    if published && recorded {
        PUBLICATION_RECORDS.with(|records| {
            *records.borrow_mut().entry((runtime, root)).or_default() += 1;
        });
    }
    if EXHAUST_AFTER_COMMIT.with(|armed| armed.replace(false)) {
        admission
            .charge_external_work(admission.remaining_work() as u64)
            .expect("the constructed exhaustion consumes exactly the remaining allowance");
    }
}
pub(super) fn observe_read_debit(executed: u64, debited: u64) {
    READ_DEBITS.with(|reads| reads.borrow_mut().push((executed, debited)));
}
thread_local! { static INTERLEAVING: RefCell<Option<Box<dyn FnOnce()>>> = const { RefCell::new(None) }; }
pub(super) fn run_interleaving() {
    let action = INTERLEAVING.with(|slot| slot.borrow_mut().take());
    if let Some(action) = action {
        action();
    }
}
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Counts new authoritative commits, independently of readiness marks.
    #[doc(hidden)]
    pub fn producer_publications_at_root_on_this_thread_for_test(
        &self,
        root: worth_relational::facade::identity::EntityId,
    ) -> u64 {
        PUBLICATIONS.with(|publications| {
            publications
                .borrow()
                .get(&(self.runtime.authority_identity().as_u64(), root))
                .copied()
                .unwrap_or(0)
        })
    }
    /// Counts publication handoffs whose member is sealed in the advance's record.
    #[doc(hidden)]
    pub fn producer_recorded_publications_at_root_on_this_thread_for_test(
        &self,
        root: worth_relational::facade::identity::EntityId,
    ) -> u64 {
        PUBLICATION_RECORDS.with(|records| {
            records
                .borrow()
                .get(&(self.runtime.authority_identity().as_u64(), root))
                .copied()
                .unwrap_or(0)
        })
    }
    #[doc(hidden)]
    pub fn exhaust_request_at_performed_commit_for_test(&self) {
        EXHAUST_AFTER_COMMIT.with(|armed| armed.set(true));
    }
    #[doc(hidden)]
    pub fn retained_read_request_debits_for_test(&self) -> Vec<(u64, u64)> {
        READ_DEBITS.with(|reads| std::mem::take(&mut *reads.borrow_mut()))
    }
    #[doc(hidden)]
    pub fn caller_request_navigation_for_test(&self) -> Vec<(u64, u64)> {
        REQUEST_NAVIGATION.with(|requests| std::mem::take(&mut *requests.borrow_mut()))
    }
    #[doc(hidden)]
    pub fn caller_request_work_for_test(&self) -> Vec<u64> {
        REQUEST_WORK.with(|requests| std::mem::take(&mut *requests.borrow_mut()))
    }
    #[doc(hidden)]
    pub fn interleave_before_owned_reobservation_for_test(&self, action: impl FnOnce() + 'static) {
        INTERLEAVING.with(|slot| *slot.borrow_mut() = Some(Box::new(action)));
    }
}

pub(super) fn observe_admission_work(admission: &InvalidationEditAdmission) {
    ADMISSION_WORK.with(|work| work.borrow_mut().push(admission.charged_work()));
}
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[doc(hidden)]
    pub fn demand_admission_work_for_test(&self) -> Vec<u64> {
        ADMISSION_WORK.with(|work| std::mem::take(&mut *work.borrow_mut()))
    }
}

impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[doc(hidden)]
    pub const fn maximum_ordered_descent_work_for_test() -> u64 {
        worth_relational::facade::indexes::SelectedIndexReadWork::MAXIMUM_ORDERED_DESCENT_WORK
    }
}
