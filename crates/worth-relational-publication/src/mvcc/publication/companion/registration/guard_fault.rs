//! Arm one fault on the registry whose real acquisition the court exercises.
use super::CompanionRegistry;
use std::sync::atomic::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Door {
    Fork = 1,
    Enter,
    Begin,
    Remove,
    Activate,
    Head,
}

impl CompanionRegistry {
    #[doc(hidden)]
    pub fn arm_guard_fault_for_test(&self, door: Door) {
        assert!(self
            .guard_fault
            .compare_exchange(0, door as u8, Ordering::Release, Ordering::Relaxed)
            .is_ok());
    }

    pub fn after_guard_acquisition(&self, door: Door) {
        let code = door as u8;
        if self.guard_fault.load(Ordering::Acquire) == code
            && self
                .guard_fault
                .compare_exchange(code, 0, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
        {
            panic!("registry guard fault: {door:?}");
        }
    }
}
