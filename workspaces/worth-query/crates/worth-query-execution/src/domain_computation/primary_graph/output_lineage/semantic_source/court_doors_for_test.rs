//! Sealed test doors over this module's private members.
use super::*;

impl SemanticSource {
    #[doc(hidden)]
    pub fn new_for_test(
        runtime_authority: u64,
        schema: ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        output_binding: super::super::super::output_binding_identity::OutputBindingIdentity,
    ) -> Self {
        Self {
            runtime_authority,
            schema,
            scope,
            output_binding,
        }
    }
}

impl SemanticSource {
    #[doc(hidden)]
    pub fn output_binding_for_test(
        &self,
    ) -> &super::super::super::output_binding_identity::OutputBindingIdentity {
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
    ) -> &mut super::super::super::output_binding_identity::OutputBindingIdentity {
        &mut self.output_binding
    }
}
