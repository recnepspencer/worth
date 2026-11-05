//! Receiver-owned interleavings and read-only caller-pass observations.

use std::cell::RefCell;
thread_local! {
    static PUBLICATIONS: RefCell<std::collections::BTreeMap<(u64, worth_relational::facade::identity::EntityId), u64>> = const { RefCell::new(std::collections::BTreeMap::new()) };
    static PUBLICATION_RECORDS: RefCell<std::collections::BTreeMap<(u64, worth_relational::facade::identity::EntityId), u64>> = const { RefCell::new(std::collections::BTreeMap::new()) };
}

thread_local! {
    static CALLER_PASSES: RefCell<Vec<(Result<(), crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind>, u64)>> = const { RefCell::new(Vec::new()) };
}
/// Observes pass results without adding an execution scope or changing control flow.
pub fn caller_pass_reports_on_this_thread_for_test() -> Vec<(
    Result<(), crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind>,
    u64,
)>{
    CALLER_PASSES.with(|passes| std::mem::take(&mut *passes.borrow_mut()))
}
pub(super) fn record_caller_pass_for_test<R>(
    result: &Result<R, crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial>,
) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
    CALLER_PASSES.with(|passes| {
        passes.borrow_mut().push((
            result.as_ref().map(|_| ()).map_err(|denial| denial.kind()),
            worth_signal::facade::observed_signal_request_work_on_this_thread_for_test(),
        ))
    });
}

pub(super) fn observe_publication(
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
}
pub trait CallerPassObservationOps<Schema>: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> {
#[doc(hidden)]
    fn producer_publications_at_root_on_this_thread_for_test(
        &self,
        root: worth_relational::facade::identity::EntityId,
    ) -> u64 {
        PUBLICATIONS.with(|publications| {
            publications
                .borrow()
                .get(&(self.authority_identity().as_u64(), root))
                .copied()
                .unwrap_or(0)
        })
    }

#[doc(hidden)]
    fn producer_recorded_publications_at_root_on_this_thread_for_test(
        &self,
        root: worth_relational::facade::identity::EntityId,
    ) -> u64 {
        PUBLICATION_RECORDS.with(|records| {
            records
                .borrow()
                .get(&(self.authority_identity().as_u64(), root))
                .copied()
                .unwrap_or(0)
        })
    }

#[doc(hidden)]
    #[doc(hidden)]
    #[doc(hidden)]
    fn interleave_before_owned_reobservation_for_test(
        &self,
        action: impl FnOnce() + Send + 'static,
    ) {
        self.primary_provider()
            .application_attempt_operation_control
            .caller_pass
            .arm_interleaving(action);
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> + ?Sized> CallerPassObservationOps<Schema> for HostRuntime {}

thread_local! { static CLEANUP_REGISTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
fn clear_observations() {
    CLEANUP_REGISTERED.with(|registered| registered.set(false));
    PUBLICATIONS.with(|records| *records.borrow_mut() = Default::default());
    PUBLICATION_RECORDS.with(|records| *records.borrow_mut() = Default::default());
    CALLER_PASSES.with(|records| records.borrow_mut().clear());
}
