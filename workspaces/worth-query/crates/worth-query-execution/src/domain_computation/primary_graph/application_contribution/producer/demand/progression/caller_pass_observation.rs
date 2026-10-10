//! A constructed writer interleaving at the owned-publication continuation.
use super::InvalidationEditAdmission;
use std::cell::RefCell;
thread_local! {
    static PUBLICATIONS: RefCell<std::collections::BTreeMap<(u64, worth_relational::facade::identity::EntityId), u64>> = const { RefCell::new(std::collections::BTreeMap::new()) };
    static PUBLICATION_RECORDS: RefCell<std::collections::BTreeMap<(u64, worth_relational::facade::identity::EntityId), u64>> = const { RefCell::new(std::collections::BTreeMap::new()) };
    static READ_DEBITS: RefCell<Vec<(u64, u64)>> = const { RefCell::new(Vec::new()) };
    static REQUEST_NAVIGATION: RefCell<Vec<(u64,u64)>> = const { RefCell::new(Vec::new()) };
    static ADMISSION_WORK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    static REQUEST_WORK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}
pub(super) fn observe_request_work(admission: &InvalidationEditAdmission) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
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
    control: &crate::domain_computation::primary_graph::application_runtime::WorthQueryApplicationAttemptOperationControl,
    admission: &mut InvalidationEditAdmission,
    published: bool,
    recorded: bool,
    runtime: u64,
    root: worth_relational::facade::identity::EntityId,
) {
    // Read-only recording is inert without the owned observer.
    if crate::domain_computation::execution_observation::is_installed() {
        if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
            crate::domain_computation::execution_observation::register_cleanup(clear_observations);
        }
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
    }
    if control.caller_pass.take_exhaustion() {
        admission
            .charge_external_work(admission.remaining_work() as u64)
            .expect("the constructed exhaustion consumes exactly the remaining allowance");
    }
}
pub(super) fn observe_read_debit(executed: u64, debited: u64) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
    READ_DEBITS.with(|reads| reads.borrow_mut().push((executed, debited)));
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
        self.primary_provider
            .application_attempt_operation_control
            .caller_pass
            .arm_exhaustion();
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
    pub fn interleave_before_owned_reobservation_for_test(
        &self,
        action: impl FnOnce() + Send + 'static,
    ) {
        self.primary_provider
            .application_attempt_operation_control
            .caller_pass
            .arm_interleaving(action);
    }
}

pub(super) fn observe_admission_work(admission: &InvalidationEditAdmission) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
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

thread_local! { static CLEANUP_REGISTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
fn clear_observations() {
    CLEANUP_REGISTERED.with(|registered| registered.set(false));
    PUBLICATIONS.with(|records| *records.borrow_mut() = Default::default());
    PUBLICATION_RECORDS.with(|records| *records.borrow_mut() = Default::default());
    READ_DEBITS.with(|records| *records.borrow_mut() = Default::default());
    REQUEST_NAVIGATION.with(|records| *records.borrow_mut() = Default::default());
    ADMISSION_WORK.with(|records| *records.borrow_mut() = Default::default());
    REQUEST_WORK.with(|records| *records.borrow_mut() = Default::default());
}

#[cfg(test)]
mod scope_tests {
    #[test]
    fn dropping_the_arming_guard_clears_both_caller_pass_arms() {
        let guard = crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread();
        let world =
            crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
                true,
            );
        world
            .application
            .exhaust_request_at_performed_commit_for_test();
        world
            .application
            .interleave_before_owned_reobservation_for_test(|| {
                panic!("a dropped arm must never run")
            });
        drop(guard);
        let control = &world
            .application
            .primary_provider
            .application_attempt_operation_control
            .caller_pass;
        assert!(!control.take_exhaustion());
        control.run_interleaving();
    }
}
