use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

static NEXT_EXECUTION_ATTEMPT: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryExecutionAttemptIdentity(Arc<str>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryClosedExecutionAttemptIdentity(Arc<str>);

impl WorthQueryExecutionAttemptIdentity {
    pub(super) fn mint() -> Self {
        let ordinal = next_execution_attempt_ordinal(&NEXT_EXECUTION_ATTEMPT)
            .expect("execution attempt identity space must not be exhausted");
        Self(Arc::from(format!("execution-attempt:{ordinal}")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(in crate::domain_computation) fn description_arc(&self) -> &Arc<str> {
        &self.0
    }

    pub(in crate::domain_computation) fn closed_identity(
        &self,
    ) -> WorthQueryClosedExecutionAttemptIdentity {
        WorthQueryClosedExecutionAttemptIdentity(Arc::clone(&self.0))
    }
}

impl WorthQueryClosedExecutionAttemptIdentity {
    pub(in crate::domain_computation) fn as_str(&self) -> &str {
        &self.0
    }
}

fn next_execution_attempt_ordinal(counter: &AtomicU64) -> Option<u64> {
    {
        let update_counter = counter;
        let mut current = update_counter.load(Ordering::Relaxed);
        loop {
            let Some(updated_value) = current.checked_add(1) else {
                break Err(current);
            };
            match update_counter.compare_exchange_weak(
                current,
                updated_value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => current = reloaded_value,
            }
        }
    }
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_attempt_ordinal_exhaustion_cannot_wrap() {
        let counter = AtomicU64::new(u64::MAX - 1);

        assert_eq!(next_execution_attempt_ordinal(&counter), Some(u64::MAX - 1));
        assert_eq!(next_execution_attempt_ordinal(&counter), None);
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
    }
}
