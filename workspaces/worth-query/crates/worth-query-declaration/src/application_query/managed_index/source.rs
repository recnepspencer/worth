//! Explicit source-to-parent correspondence. These descriptions cannot select a source.
use crate::application_query::ApplicationQueryBindingReference;

/// Reuse the parent session’s exact admitted principal and scope.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationManagedIndexSourceScope {
    SameAdmittedScope,
}
/// Source projection has no replayed or inferred parameters.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationManagedIndexSourceParameters {
    NoParameters,
}

/// Installation compares these routes with the complete actual source and parent bindings.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationManagedIndexSourceMeaning {
    pub binding: ApplicationQueryBindingReference,
    pub scope: ApplicationManagedIndexSourceScope,
    pub parameters: ApplicationManagedIndexSourceParameters,
}
impl ApplicationManagedIndexSourceMeaning {
    pub fn same_scope_without_parameters(binding: ApplicationQueryBindingReference) -> Self {
        Self {
            binding,
            scope: ApplicationManagedIndexSourceScope::SameAdmittedScope,
            parameters: ApplicationManagedIndexSourceParameters::NoParameters,
        }
    }
}
