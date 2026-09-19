#[derive(Debug)]
pub(crate) struct UiPreparedThemeSwitch {
    pub(super) reservation: u64,
    pub(super) predecessor_generation: u64,
    pub(super) successor: super::UiActiveThemeBinding,
    pub(super) origin: super::UiThemeSwitchOrigin,
    pub(super) owner_affinity: u64,
    pub(super) reservations: std::rc::Weak<
        std::cell::RefCell<
            std::collections::BTreeMap<u64, super::state::UiPreparedThemeReservation>,
        >,
    >,
}

impl UiPreparedThemeSwitch {
    pub(crate) const fn successor(&self) -> &super::UiActiveThemeBinding {
        &self.successor
    }
}

impl Drop for UiPreparedThemeSwitch {
    fn drop(&mut self) {
        if let Some(reservations) = self.reservations.upgrade() {
            let mut reservations = reservations.borrow_mut();
            if reservations
                .get(&self.reservation)
                .is_some_and(|row| row.owner_affinity == self.owner_affinity)
            {
                reservations.remove(&self.reservation);
            }
        }
    }
}

impl PartialEq for UiPreparedThemeSwitch {
    fn eq(&self, other: &Self) -> bool {
        self.reservation == other.reservation
            && self.predecessor_generation == other.predecessor_generation
            && self.successor == other.successor
            && self.origin == other.origin
            && self.owner_affinity == other.owner_affinity
    }
}
impl Eq for UiPreparedThemeSwitch {}
