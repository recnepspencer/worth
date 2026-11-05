//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_output_demand::native_hints::NativeBranch;
use crate::domain_computation::primary_graph::vocabulary::application_output_demand::native_hints::NativeHint;

impl RequiredWorkMembership {
    #[doc(hidden)]
    pub fn prepared_native_branch_for_test(branch: BranchId) -> Arc<NativeBranch> {
        Self::prepared_native_branch(branch)
    }
}

impl RequiredWorkMembership {
    #[doc(hidden)]
    pub fn prepared_native_hint_for_test(
        observer: CompanionPublicationCompletionObserver,
        branch: Arc<NativeBranch>,
    ) -> Box<NativeHint> {
        Self::prepared_native_hint(observer, branch)
    }
}
