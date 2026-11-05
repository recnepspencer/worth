//! Sealed test doors over this module's private members.
use super::*;

impl<'a, Schema> ConditionalDefinitionAdmission<'a, Schema> {
    #[doc(hidden)]
    pub fn new_for_test(
        application: &'a dyn crate::domain_computation::primary_graph::handler::host::HandlerHost<
            Schema,
        >,
        product: Option<&'a WorthQueryProductBranchLease>,
    ) -> Self {
        Self::new(application, product)
    }
}
