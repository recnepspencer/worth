use super::super::super::unique_component_pin::ExactComponentBasis;

/// Unit-test observations for owner contacts, terminations and flight races.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct RetentionTestCounters {
    pub(super) owner_release_contacts: u64,
    pub(super) owner_drop_releases: u64,
    pub(super) dependency_releases: u64,
    pub(super) single_flight_joins: u64,
    relational_contacts: u64,
    signal_contacts: u64,
}

impl RetentionTestCounters {
    pub(crate) const fn owner_acquisition_contacts(self) -> u64 {
        self.relational_contacts
            .saturating_add(self.signal_contacts)
    }
    pub(crate) const fn owner_release_contacts(self) -> u64 {
        self.owner_release_contacts
    }
    pub(crate) const fn owner_drop_releases(self) -> u64 {
        self.owner_drop_releases
    }
    pub(crate) const fn dependency_releases(self) -> u64 {
        self.dependency_releases
    }
    pub(crate) const fn single_flight_joins(self) -> u64 {
        self.single_flight_joins
    }
    pub(crate) const fn relational_contacts(self) -> u64 {
        self.relational_contacts
    }
    pub(crate) const fn signal_contacts(self) -> u64 {
        self.signal_contacts
    }

    pub(super) fn record_component_contact(&mut self, component: ExactComponentBasis<'_>) {
        match component {
            ExactComponentBasis::Relational(_) => {
                self.relational_contacts = self.relational_contacts.saturating_add(1);
            }
            ExactComponentBasis::Signal(_) => {
                self.signal_contacts = self.signal_contacts.saturating_add(1);
            }
        }
    }
}
