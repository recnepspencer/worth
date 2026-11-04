//! Sealed test doors over this module's private members.
use super::*;

impl OutputRoleUse {
    #[doc(hidden)]
    pub fn fixed_for_test<Role>() -> Self
    where
        Role: WorthQueryApplicationOutputRole,
    {
        Self::fixed::<Role>()
    }
}

impl OutputRoleUse {
    #[doc(hidden)]
    pub fn member_for_test<Family, Action>(
        suffix: &str,
    ) -> Result<Self, WorthQueryApplicationOutputRoleNameDenial>
    where
        Family: WorthQueryApplicationOutputRoleFamily,
        Action: WorthQueryApplicationOutputAction,
    {
        Self::member::<Family, Action>(suffix)
    }
}
