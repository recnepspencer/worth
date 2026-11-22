//! Sealed test doors over this module's private members.
use super::*;

impl InterruptionPoint {
    #[doc(hidden)]
    pub fn completed_for_test(&self) -> &Vec<EntityId> {
        &self.completed
    }
}

impl InterruptionPoint {
    #[doc(hidden)]
    pub fn fresh_for_test(&self) -> &bool {
        &self.fresh
    }
}

impl InterruptionPoint {
    #[doc(hidden)]
    pub fn root_for_test(&self) -> &EntityId {
        &self.root
    }
}

impl DispatchWitness {
    #[doc(hidden)]
    pub fn assert_interruption_point_for_test(&self) {
        Self::assert_interruption_point(self)
    }
}

impl DispatchWitness {
    #[doc(hidden)]
    pub fn interrupt_for_test(&self, interruption: Interrupt) {
        Self::interrupt(self, interruption)
    }
}

impl DispatchWitness {
    #[doc(hidden)]
    pub fn reset_for_test(&self) {
        Self::reset(self)
    }
}
