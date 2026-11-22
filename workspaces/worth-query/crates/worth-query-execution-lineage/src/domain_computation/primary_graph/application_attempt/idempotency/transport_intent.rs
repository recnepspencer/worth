//! Additional stable transport meaning participates in the same durable intent.
use super::WorthQueryApplicationIdempotencyBinding;
use sha2::{Digest, Sha256};

impl WorthQueryApplicationIdempotencyBinding {
    /// Extends the declared key namespace with the admitted complete transport
    /// key and caller scope. No narrowing or process identity participates.
    pub fn bind_transport_key(mut self, key: &[u8; 32]) -> Self {
        let mut digest = Sha256::new();
        digest.update(b"worth.query.transport-mutation-key.v1");
        digest.update(self.key_identity);
        digest.update(key);
        self.key_identity = digest.finalize().into();
        self
    }

    /// Binds transport-admitted meaning in addition to the canonically encoded
    /// application input. This grants no operation or publication authority.
    pub fn bind_transport_intent(mut self, intent: &[u8; 32]) -> Self {
        self.transport_intent_identity = Some(*intent);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transport_intent_preserves_handler_input_and_changes_recorded_meaning() {
        let binding = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
        let left = binding.bind_transport_intent(&[3; 32]);
        let right = binding.bind_transport_intent(&[4; 32]);
        assert_eq!(binding.key_identity(), left.key_identity());
        assert_eq!(binding.intent_identity(), left.intent_identity());
        assert_ne!(binding.intent_text(), left.intent_text());
        assert_ne!(left.intent_text(), right.intent_text());
        assert_eq!(
            left.match_recorded_intent(&left.intent_text()),
            Ok(super::super::WorthQueryRecordedIntentMatch::Same)
        );
        assert_eq!(
            left.match_recorded_intent(&right.intent_text()),
            Ok(super::super::WorthQueryRecordedIntentMatch::Drift)
        );
        assert_eq!(left, binding.bind_transport_intent(&[3; 32]));
    }
    #[test]
    fn every_transport_key_byte_participates_in_the_declared_namespace() {
        let binding = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
        let base = binding.bind_transport_key(&[0; 32]);
        for position in 0..32 {
            let mut key = [0; 32];
            key[position] = 1;
            let scoped = binding.bind_transport_key(&key);
            assert_ne!(scoped.key_identity(), base.key_identity());
            assert_eq!(scoped.intent_identity(), binding.intent_identity());
        }
        assert_ne!(
            base.key_identity(),
            WorthQueryApplicationIdempotencyBinding::new([3; 32], [2; 32])
                .bind_transport_key(&[0; 32])
                .key_identity()
        );
        assert_eq!(base, binding.bind_transport_key(&[0; 32]));
    }
}
