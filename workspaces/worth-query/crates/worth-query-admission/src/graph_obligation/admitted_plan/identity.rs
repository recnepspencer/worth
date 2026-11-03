//! One process-local plan issuer and its exact exhausted counter refusal.
use super::super::WorthQueryGraphWorkAdmissionDenial as Denial;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
static NEXT_GRAPH_WORK_PLAN_IDENTITY: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorthQueryGraphWorkPlanIdentity(u64);
impl WorthQueryGraphWorkPlanIdentity {
    pub(in crate::graph_obligation) fn mint() -> Result<Self, Denial> {
        mint_from(&NEXT_GRAPH_WORK_PLAN_IDENTITY)
    }
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}
fn mint_from(counter: &AtomicU64) -> Result<WorthQueryGraphWorkPlanIdentity, Denial> {
    let mut current = counter.load(Relaxed);
    loop {
        let next = current
            .checked_add(1)
            .ok_or(Denial::PlanIdentityExhausted)?;
        match counter.compare_exchange_weak(current, next, Relaxed, Relaxed) {
            Ok(previous) => return Ok(WorthQueryGraphWorkPlanIdentity(previous)),
            Err(observed) => current = observed,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn graph_plan_identity_exhaustion_names_its_counter_and_never_wraps() {
        let counter = AtomicU64::new(u64::MAX - 1);
        assert_eq!(
            mint_from(&counter).expect("last plan identity").as_u64(),
            u64::MAX - 1
        );
        for _ in 0..3 {
            assert_eq!(mint_from(&counter), Err(Denial::PlanIdentityExhausted));
            assert_eq!(counter.load(Relaxed), u64::MAX);
        }
    }
}
