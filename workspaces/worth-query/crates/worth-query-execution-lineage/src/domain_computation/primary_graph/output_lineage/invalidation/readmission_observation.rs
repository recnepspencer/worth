//! Certification observation of historical readmission (public #358): how
//! often restored evidence was accepted, carried, registered or readmitted.

use std::cell::Cell;

thread_local! {
    static HISTORICAL_ACCEPTANCES: Cell<u64> = const { Cell::new(0) };
    static CARRIED_HISTORICAL_PROJECTIONS: Cell<u64> = const { Cell::new(0) };
    static SETTLEMENT_REGISTRATIONS: Cell<u64> = const { Cell::new(0) };
    static READMITTED_CHAINS: Cell<u64> = const { Cell::new(0) };
    static DECLINED_READMISSIONS: Cell<u64> = const { Cell::new(0) };
    /// The readmission kind the armed door declines: consumers or roots.
    static DECLINE_CONSUMERS: Cell<Option<bool>> = const { Cell::new(None) };
}

fn registered() -> bool {
    if !crate::domain_computation::execution_observation::is_installed() {
        return false;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
    true
}

fn count(counter: &'static std::thread::LocalKey<Cell<u64>>) {
    if !registered() {
        return;
    }
    counter.with(|value| {
        value.set(
            value
                .get()
                .checked_add(1)
                .expect("readmission observation counter overflow"),
        );
    });
}

/// A moved fact or output aspect held only through its row's selection evidence.
pub fn record_historical_acceptance() {
    count(&HISTORICAL_ACCEPTANCES);
}

/// An equal successor took an output projection that carries a coordinate.
pub fn record_carried_historical_projection() {
    count(&CARRIED_HISTORICAL_PROJECTIONS);
}

/// A settlement registration was prepared into a mark image.
pub fn record_settlement_registration() {
    count(&SETTLEMENT_REGISTRATIONS);
}

/// A readmitted consumer chain left its consumer Clean at the selected head.
pub fn record_readmitted_chain() {
    count(&READMITTED_CHAINS);
}

/// Whether the armed certification door declines this verified readmission
/// of a consumer (`true`) or a root (`false`); a decline is counted. With no
/// door armed every readmission proceeds.
pub fn historical_readmission_declined(consumer: bool) -> bool {
    let declined = DECLINE_CONSUMERS.with(Cell::get) == Some(consumer);
    if declined {
        count(&DECLINED_READMISSIONS);
    }
    declined
}

/// Public #358 liveness courts: until the installed observation guard drops,
/// every historical readmission of a consumer (`true`) or a root (`false`)
/// that verified current declines, as any stop after currentness would.
#[doc(hidden)]
pub fn decline_historical_readmissions_on_this_thread_for_test(consumers: bool) {
    assert!(
        registered(),
        "the decline door needs an installed execution observation"
    );
    DECLINE_CONSUMERS.set(Some(consumers));
}

/// Verified historical readmissions the armed door declined on this thread.
#[doc(hidden)]
pub fn declined_historical_readmissions_on_this_thread_for_test() -> u64 {
    DECLINED_READMISSIONS.with(Cell::get)
}

/// Moved facts or aspects on this thread accepted only by selection evidence.
#[doc(hidden)]
pub fn historical_acceptances_on_this_thread_for_test() -> u64 {
    HISTORICAL_ACCEPTANCES.with(Cell::get)
}

/// Output projections with a coordinate carried onto equal successors on this thread.
#[doc(hidden)]
pub fn carried_historical_projections_on_this_thread_for_test() -> u64 {
    CARRIED_HISTORICAL_PROJECTIONS.with(Cell::get)
}

/// Settlement registrations prepared on this thread, refused installs included.
#[doc(hidden)]
pub fn settlement_registrations_on_this_thread_for_test() -> u64 {
    SETTLEMENT_REGISTRATIONS.with(Cell::get)
}

/// Consumer chains readmitted with marks at their selected head on this thread.
#[doc(hidden)]
pub fn readmitted_chains_on_this_thread_for_test() -> u64 {
    READMITTED_CHAINS.with(Cell::get)
}

// Law 2, scope: passive counts end with their owned observation guard.
thread_local! { static CLEANUP_REGISTERED: Cell<bool> = const { Cell::new(false) }; }
fn clear_observations() {
    CLEANUP_REGISTERED.set(false);
    HISTORICAL_ACCEPTANCES.set(0);
    CARRIED_HISTORICAL_PROJECTIONS.set(0);
    SETTLEMENT_REGISTRATIONS.set(0);
    READMITTED_CHAINS.set(0);
    DECLINED_READMISSIONS.set(0);
    DECLINE_CONSUMERS.set(None);
}
