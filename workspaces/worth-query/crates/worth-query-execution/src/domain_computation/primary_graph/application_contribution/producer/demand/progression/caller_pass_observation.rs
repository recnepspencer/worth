//! Receiver-owned interleavings and read-only caller-pass observations.

use std::cell::RefCell;
thread_local! {
    static PUBLICATIONS: RefCell<std::collections::BTreeMap<(u64, worth_relational::facade::identity::EntityId), u64>> = const { RefCell::new(std::collections::BTreeMap::new()) };
    static PUBLICATION_RECORDS: RefCell<std::collections::BTreeMap<(u64, worth_relational::facade::identity::EntityId), u64>> = const { RefCell::new(std::collections::BTreeMap::new()) };
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
    #[doc(hidden)]
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

thread_local! { static CLEANUP_REGISTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
fn clear_observations() {
    CLEANUP_REGISTERED.with(|registered| registered.set(false));
    PUBLICATIONS.with(|records| *records.borrow_mut() = Default::default());
    PUBLICATION_RECORDS.with(|records| *records.borrow_mut() = Default::default());
}

#[cfg(test)]
mod scope_tests {
    #[test]
    fn dropping_the_arming_guard_clears_the_interleaving() {
        let guard = crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread();
        let world =
            crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
                true,
            );
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
        control.run_interleaving();
    }
}
