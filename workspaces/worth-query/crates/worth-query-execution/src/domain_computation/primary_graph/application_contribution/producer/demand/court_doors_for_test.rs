//! Sealed test doors over this module's private members.
use super::*;

impl WorthQuerySelectedApplicationProducer {
    #[doc(hidden)]
    pub fn new_for_test(
        identity: String,
        applicability: WorthQueryProducerApplicability,
        exact_retained_output: bool,
        retained_idempotency_key: Option<[u8; 32]>,
        retained_output_binding: Option<TypeId>,
        reuses_live_output_only: bool,
    ) -> Self {
        Self {
            identity,
            applicability,
            exact_retained_output,
            retained_idempotency_key,
            retained_output_binding,
            reuses_live_output_only,
        }
    }
}
