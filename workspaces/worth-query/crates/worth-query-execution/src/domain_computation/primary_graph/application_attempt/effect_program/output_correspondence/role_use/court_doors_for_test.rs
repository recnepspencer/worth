//! Sealed test doors over this module's private members.
use super::*;

impl OutputRoleUse {
    #[doc(hidden)]
    pub fn fixed_for_test<Role>() -> Self
    where
        Role: WorthQueryApplicationOutputRole, {
        Self::fixed::<Role>()
    }
}
