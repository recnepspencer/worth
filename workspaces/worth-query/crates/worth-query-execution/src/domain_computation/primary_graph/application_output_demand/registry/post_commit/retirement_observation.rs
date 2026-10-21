#[cfg(test)]
thread_local! { static SWEEP_CONTACTS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
pub(super) fn sweep_contacts() -> u64 {
    SWEEP_CONTACTS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(super) fn record_sweep_contact() {
    SWEEP_CONTACTS.with(|count| count.set(count.get() + 1));
}
