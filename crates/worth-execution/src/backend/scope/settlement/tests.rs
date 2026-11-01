//! What a hold is left holding once a held scope's kernel has returned.
use super::*;
use crate::backend::meter::KernelContext;
use crate::SerialMemoryBudget;

/// A settled payload whose kept measure and destructor are hostile.
struct Settled {
    kept: fn() -> Option<u64>,
    discards: u64,
}

impl ScopeBody for Settled {
    fn invoke(
        &mut self,
        _: &mut KernelContext<'_, '_>,
        _: Option<crate::ExecutionResultMemory<'_>>,
    ) -> KernelVerdict {
        unreachable!("settlement follows the kernel")
    }

    fn payload_bytes(&self) -> u64 {
        0
    }

    fn kept_bytes(&self) -> Option<u64> {
        (self.kept)()
    }

    fn discard(&mut self) {
        self.discards += 1;
        panic!("hostile destructor");
    }
}

fn settled(kept: fn() -> Option<u64>) -> Settled {
    Settled { kept, discards: 0 }
}

fn held(budget: &SerialMemoryBudget) -> ExecutionMemoryReservation {
    budget.reserve(64).expect("the budget admits the entry")
}

#[test]
fn a_hold_keeps_only_what_a_completed_result_keeps() {
    let budget = SerialMemoryBudget::new(None);
    let mut hold = held(&budget);
    let mut body = settled(|| Some(8));
    assert!(settle_hold(&mut hold, Ok(HeldPayload), &mut body).is_ok());
    assert_eq!(hold.bytes(), 8);
    assert_eq!(body.discards, 0);
}

#[test]
fn a_failed_scope_empties_its_hold_without_asking_what_it_keeps() {
    let budget = SerialMemoryBudget::new(None);
    let mut hold = held(&budget);
    let mut body = settled(|| panic!("nothing was kept"));
    let failed = Err(ScopeStop::Failure(KernelFailure::Panic));
    assert!(settle_hold(&mut hold, failed, &mut body).is_err());
    assert_eq!(hold.bytes(), 0);
    assert_eq!(body.discards, 0);
}

#[test]
fn a_panicking_kept_measure_fails_the_scope_and_empties_its_hold() {
    let budget = SerialMemoryBudget::new(None);
    let mut hold = held(&budget);
    let mut body = settled(|| panic!("hostile kept measure"));
    assert!(matches!(
        settle_hold(&mut hold, Ok(HeldPayload), &mut body),
        Err(ScopeStop::Failure(KernelFailure::Panic))
    ));
    assert_eq!(hold.bytes(), 0);
    assert_eq!(body.discards, 1);
}

#[test]
fn a_kept_size_past_the_counter_refuses_the_result_and_empties_its_hold() {
    let budget = SerialMemoryBudget::new(None);
    let mut hold = held(&budget);
    let mut body = settled(|| None);
    assert!(matches!(
        settle_hold(&mut hold, Ok(HeldPayload), &mut body),
        Err(ScopeStop::Admission(_))
    ));
    assert_eq!(hold.bytes(), 0);
    assert_eq!(body.discards, 1);
}

#[test]
fn a_kept_size_past_the_budget_refuses_the_result_and_empties_its_hold() {
    let budget = SerialMemoryBudget::new(Some(64));
    let mut hold = held(&budget);
    let mut body = settled(|| Some(65));
    assert!(matches!(
        settle_hold(&mut hold, Ok(HeldPayload), &mut body),
        Err(ScopeStop::Admission(LeaseDenial::MemoryExhausted(_)))
    ));
    assert_eq!(hold.bytes(), 0);
    assert_eq!(body.discards, 1);
    drop(hold);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
