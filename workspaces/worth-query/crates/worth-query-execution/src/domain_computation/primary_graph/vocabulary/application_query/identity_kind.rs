//! Moved whole from `domain_computation::primary_graph::application_query::observed_source::source_identity::identity_kind`: the items below and every impl they own.

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorthQueryRuntimeSourceIdentity([u8; 32]);

impl WorthQueryRuntimeSourceIdentity {
    pub(in crate::domain_computation::primary_graph) const fn new(identity: [u8; 32]) -> Self {
        Self(identity)
    }

    pub(in crate::domain_computation::primary_graph) const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorthQueryCheckpointSourceIdentity([u8; 32]);

impl WorthQueryCheckpointSourceIdentity {
    pub(in crate::domain_computation::primary_graph) const fn new(identity: [u8; 32]) -> Self {
        Self(identity)
    }

    pub(in crate::domain_computation::primary_graph) const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

impl WorthQueryCheckpointSourceIdentity {
    #[doc(hidden)]
    pub fn new_for_test(identity: [u8; 32]) -> Self {
        Self::new(identity)
    }
}

impl WorthQueryRuntimeSourceIdentity {
    #[doc(hidden)]
    pub fn new_for_test(identity: [u8; 32]) -> Self {
        Self::new(identity)
    }
}

mod court_doors_for_test;
