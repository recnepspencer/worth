//! Certification observations for output readmission and settlement registration.

use std::cell::Cell;

thread_local! {
    static SETTLEMENT_REGISTRATIONS: Cell<u64> = const { Cell::new(0) };
    static READMITTED_CHAINS: Cell<u64> = const { Cell::new(0) };
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



/// A settlement registration was prepared into a mark image.
pub fn record_settlement_registration() {
    count(&SETTLEMENT_REGISTRATIONS);
}

/// A readmitted consumer chain left its consumer Clean at the selected head.
pub fn record_readmitted_chain() {
    count(&READMITTED_CHAINS);
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
    SETTLEMENT_REGISTRATIONS.set(0);
    READMITTED_CHAINS.set(0);
}
