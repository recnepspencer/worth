//! Output meaning of the exact registered provider effect program.

use std::any::TypeId;

use super::WorthQueryPrimaryGraphApplicationAttempt;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationOutputPosture;

impl WorthQueryPrimaryGraphApplicationAttempt {
    pub(in crate::domain_computation::primary_graph) fn native_witness_roles(
        &self,
    ) -> impl ExactSizeIterator<Item = (&str, WorthQueryApplicationOutputPosture, &str)> + Clone
    {
        self.effects.native_witness_roles()
    }

    pub(in crate::domain_computation::primary_graph) const fn output_binding_type(
        &self,
    ) -> Option<TypeId> {
        self.effects.output_binding_type()
    }
}
