//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::semantic_source::SemanticSource;

impl SemanticSource {
    #[doc(hidden)]
    pub fn output_binding_for_test(
        &self,
    ) -> &crate::domain_computation::primary_graph::vocabulary::output_binding_identity::OutputBindingIdentity{
        &self.output_binding
    }
}

impl SemanticSource {
    #[doc(hidden)]
    pub fn runtime_authority_for_test(&self) -> &u64 {
        &self.runtime_authority
    }
}

impl SemanticSource {
    #[doc(hidden)]
    pub fn schema_for_test(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }
}

impl SemanticSource {
    #[doc(hidden)]
    pub fn output_binding_mut_for_test(
        &mut self,
    ) -> &mut crate::domain_computation::primary_graph::vocabulary::output_binding_identity::OutputBindingIdentity{
        &mut self.output_binding
    }
}
