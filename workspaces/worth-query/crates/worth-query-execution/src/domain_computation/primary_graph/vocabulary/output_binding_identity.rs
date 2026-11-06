//! `output_binding_identity`'s vocabulary, moved whole from `primary_graph::output_binding_identity`: its root file's items here, one child per other source file.

use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OutputBindingIdentity(Arc<str>);

impl OutputBindingIdentity {
    pub(in crate::domain_computation::primary_graph) fn declared(name: &str) -> Self {
        Self(Arc::from(name))
    }
    pub(in crate::domain_computation::primary_graph) fn as_str(&self) -> &str {
        &self.0
    }
}

impl OutputBindingIdentity {
    #[doc(hidden)]
    pub fn declared_for_test(name: &str) -> Self {
        Self::declared(name)
    }
}

mod court_doors_for_test;
