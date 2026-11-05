//! Sealed test doors over this module's private members.
use crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryCheckpointSourceIdentity;

impl WorthQueryCheckpointSourceIdentity {
    #[doc(hidden)]
    pub fn bytes_for_test(self) -> [u8; 32] {
        Self::bytes(self)
    }
}
