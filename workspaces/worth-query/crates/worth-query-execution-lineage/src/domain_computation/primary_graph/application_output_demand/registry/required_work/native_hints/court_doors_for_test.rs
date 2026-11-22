//! Sealed test doors over this module's private members.
use super::*;

impl NativeBranch {
    #[doc(hidden)]
    pub fn branch_id_for_test(&self) -> &BranchId {
        Self::branch_id(self)
    }
}

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
