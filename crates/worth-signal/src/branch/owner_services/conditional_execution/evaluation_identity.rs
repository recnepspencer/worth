use std::sync::Arc;

pub(super) fn evaluation_identity(ordinal: u64) -> Arc<str> {
    Arc::from(format!("signal-evaluation:{ordinal:020}"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn evaluation_identity_width_is_independent_of_ordinal_magnitude() {
        let first = super::evaluation_identity(0);
        let last = super::evaluation_identity(u64::MAX);

        assert_eq!(first.len(), last.len());
        assert_ne!(first, last);
    }
}
