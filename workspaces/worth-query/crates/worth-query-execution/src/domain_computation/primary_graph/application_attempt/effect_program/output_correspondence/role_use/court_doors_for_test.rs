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

impl OutputRoleUse {
    #[doc(hidden)]
    pub fn new_for_test(
        name: String,
        posture: ApplicationMutationOutputPosture,
        cardinality: ApplicationMutationOutputRoleCardinality,
        entity_name: &'static str,
        entity_type: TypeId,
        contract_type: TypeId,
    ) -> Self {
        Self {
            name,
            posture,
            cardinality,
            entity_name,
            entity_type,
            contract_type,
        }
    }
}
