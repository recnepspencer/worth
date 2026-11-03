//! Sealed test doors over this module's private members.
use super::*;

impl SemanticSource {
    #[doc(hidden)]
    pub fn new_for_test(runtime_authority: u64, schema: ApplicationSchemaBindingIdentity, scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding, output_binding: super::super::super::output_binding_identity::OutputBindingIdentity) -> Self {
        Self { runtime_authority, schema, scope, output_binding }
    }
}
