//! Contact intervals belong to one open demand handle.

#[derive(Default)]
pub(super) struct Reading {
    contacts: usize,
}

impl Reading {
    /// Executions initiated since this handle's preceding settled reading.
    pub(super) fn contacts(&mut self, lifetime: usize) -> usize {
        let delta = lifetime
            .checked_sub(self.contacts)
            .expect("a handle's lifetime contact count never decreases");
        self.contacts = lifetime;
        delta
    }
}
