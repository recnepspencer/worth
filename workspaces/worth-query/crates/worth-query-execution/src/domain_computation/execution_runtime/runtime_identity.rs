use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::vocabulary::execution_runtime::runtime_identity::next_runtime_authority_ordinal;

    #[test]
    fn runtime_authority_identity_exhaustion_cannot_wrap_or_reuse() {
        let counter = AtomicU64::new(u64::MAX - 1);

        assert_eq!(next_runtime_authority_ordinal(&counter), Some(u64::MAX - 1));
        assert_eq!(next_runtime_authority_ordinal(&counter), None);
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
    }
}
