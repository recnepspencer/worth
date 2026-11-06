//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::computation_routing::ComputationRead;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::fact::WorthQueryApplicationFactKey;

impl ComputationFactIndex {
    #[doc(hidden)]
    pub fn keys_for_test(
        &self,
        read: ComputationRead,
    ) -> impl Iterator<Item = &WorthQueryApplicationFactKey> {
        Self::keys(self, read)
    }
}

impl ComputationFactIndex {
    #[doc(hidden)]
    pub fn calls_for_test(
        &self,
    ) -> &ExecutionOrderedMap<ComputationRead, Vec<WorthQueryApplicationFactKey>> {
        &self.calls
    }
}
