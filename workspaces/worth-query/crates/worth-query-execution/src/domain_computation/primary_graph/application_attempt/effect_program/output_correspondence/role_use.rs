//! The one internal form of an output-role use. Every public use site builds
//! it from a declaration marker through [`OutputRoleUse::fixed`] or
//! [`OutputRoleUse::member`], and both evaluate the marker's contract check
//! in an inline `const` block, so a use its contract does not declare fails
//! to compile at the instantiating call.

use std::any::TypeId;

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationOutputPosture, ApplicationMutationOutputRoleCardinality,
    WorthQueryApplicationDeclaredOutputRole, WorthQueryApplicationDeclaredOutputRoleFamily,
    WorthQueryApplicationOutputAction, WorthQueryApplicationOutputCardinality,
    WorthQueryApplicationOutputRole, WorthQueryApplicationOutputRoleFamily,
    WorthQueryApplicationOutputRoleNameDenial,
};
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct OutputRoleUse {
    pub(in crate::domain_computation::primary_graph) name: String,
    pub(in crate::domain_computation::primary_graph) posture: ApplicationMutationOutputPosture,
    pub(in crate::domain_computation::primary_graph) cardinality:
        ApplicationMutationOutputRoleCardinality,
    pub(in crate::domain_computation::primary_graph) entity_name: &'static str,
    pub(in crate::domain_computation::primary_graph) entity_type: TypeId,
    pub(in crate::domain_computation::primary_graph) contract_type: TypeId,
}

impl OutputRoleUse {
    /// The use of one fixed role, checked against its contract at compile
    /// time.
    pub(in crate::domain_computation::primary_graph) fn fixed<Role>() -> Self
    where
        Role: WorthQueryApplicationOutputRole,
    {
        const { <Role as WorthQueryApplicationDeclaredOutputRole>::DECLARED };
        Self {
            name: Role::NAME.to_owned(),
            posture: <Role::Action as WorthQueryApplicationOutputAction>::POSTURE,
            cardinality: <Role::Cardinality as WorthQueryApplicationOutputCardinality>::CARDINALITY,
            entity_name:
                <Role::Entity as ApplicationEntityMarkerIdentity<Role::Schema>>::IDENTIFIER,
            entity_type: TypeId::of::<Role::Entity>(),
            contract_type: TypeId::of::<Role::Contract>(),
        }
    }

    /// The use of the family member named by `suffix` with `Action`. The
    /// family and the action's posture are checked against the contract at
    /// compile time; the suffix is validated here.
    pub(in crate::domain_computation::primary_graph) fn member<Family, Action>(
        suffix: &str,
    ) -> Result<Self, WorthQueryApplicationOutputRoleNameDenial>
    where
        Family: WorthQueryApplicationOutputRoleFamily,
        Action: WorthQueryApplicationOutputAction,
    {
        const { declared_member::<Family, Action>() };
        Ok(Self {
            name: Family::member_name(suffix)?,
            posture: Action::POSTURE,
            cardinality: ApplicationMutationOutputRoleCardinality::ExactlyOne,
            entity_name:
                <Family::Entity as ApplicationEntityMarkerIdentity<Family::Schema>>::IDENTIFIER,
            entity_type: TypeId::of::<Family::Entity>(),
            contract_type: TypeId::of::<Family::Contract>(),
        })
    }
}

/// Evaluates only when the family is declared in its contract and admits the
/// action's posture.
const fn declared_member<Family, Action>()
where
    Family: WorthQueryApplicationOutputRoleFamily,
    Action: WorthQueryApplicationOutputAction,
{
    let () = <Family as WorthQueryApplicationDeclaredOutputRoleFamily>::DECLARED;
    if !Family::POSTURES.allows(Action::POSTURE) {
        panic!("the output-role family does not admit this posture");
    }
}
