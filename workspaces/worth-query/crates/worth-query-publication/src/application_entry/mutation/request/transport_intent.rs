//! Stable transport input augments the authored application intent.
use super::*;
impl<Schema, Intent, Preparation>
    WorthQueryApplicationMutationRequestWithIdempotency<'_, '_, '_, '_, Schema, Intent, Preparation>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    /// Binds the complete transport key/caller namespace and stable intent in
    /// addition to the typed application key and input. Both are co-committed.
    pub fn with_transport_idempotency(mut self, key: [u8; 32], intent: [u8; 32]) -> Self {
        self.transport_key = Some(key);
        self.transport_intent = Some(intent);
        self
    }

    /// Includes the transport's admitted stable intent in the co-committed
    /// binding. Request IDs, current basis observations and live session IDs
    /// must not participate. The typed input and operation remain bound too.
    pub fn with_transport_intent(mut self, intent: [u8; 32]) -> Self {
        self.transport_intent = Some(intent);
        self
    }
}
