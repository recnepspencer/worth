//! Output-role declarations. A role is a marker type: implementing
//! [`WorthQueryApplicationOutputRole`] declares its contract, entity, action,
//! cardinality and name, and [`WorthQueryApplicationDeclaredOutputRole`]
//! derives the descriptor the contract lists. Every Query use site names the
//! marker and evaluates [`WorthQueryApplicationDeclaredOutputRole::DECLARED`],
//! so a role its contract does not declare exactly as used fails to compile.

use super::output::{
    ApplicationMutationOutputContract, ApplicationMutationOutputPostureSet,
    ApplicationMutationOutputRoleDescriptor,
};
use crate::application_schema::{ApplicationEntityMarkerIdentity, ApplicationSchema};

mod action;
mod cardinality;
mod family;
mod name;

pub use action::{
    WorthQueryApplicationOutputAction, WorthQueryCreateOutput, WorthQueryPreserveOutput,
    WorthQueryRetireOutput,
};
pub use cardinality::{
    WorthQueryApplicationOutputCardinality, WorthQueryAtMostOneOutput, WorthQueryExactlyOneOutput,
};
pub use family::{
    WorthQueryApplicationDeclaredOutputRoleFamily, WorthQueryApplicationOutputRoleFamily,
};
pub use name::WorthQueryApplicationOutputRoleNameDenial;

/// The declaration of one fixed output role of an output contract.
///
/// The role belongs to the contract, so one marker serves every binding whose
/// `Output` is that contract. The contract lists the derived
/// [`WorthQueryApplicationDeclaredOutputRole::DESCRIPTOR`] in its `ROLES`.
///
/// ```
/// use worth_query_declaration::facade::application_operation::{
///     ApplicationMutationOutputContract, ApplicationMutationOutputRoleCardinality,
///     ApplicationMutationOutputRoleDescriptor, WorthQueryApplicationDeclaredOutputRole,
///     WorthQueryApplicationOutputRole, WorthQueryAtMostOneOutput, WorthQueryCreateOutput,
/// };
/// use worth_query_declaration::facade::application_schema::{
///     ApplicationEntityMarkerIdentity, ApplicationSchema, ApplicationSchemaDeclaration,
///     ApplicationSchemaDeclarationDenial,
/// };
///
/// struct Ledger;
/// impl ApplicationSchema for Ledger {
///     const OWNER: &'static str = "doc";
///     const NAME: &'static str = "ledger";
///     const MAJOR: u32 = 1;
///     const MINOR: u32 = 0;
///     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
///         unimplemented!()
///     }
/// }
/// struct Account;
/// impl ApplicationEntityMarkerIdentity<Ledger> for Account {
///     const IDENTIFIER: &'static str = "Account";
/// }
///
/// struct OpenOutputs;
/// impl ApplicationMutationOutputContract<Ledger> for OpenOutputs {
///     const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[OpenedAccount::DESCRIPTOR];
/// }
///
/// struct OpenedAccount;
/// impl WorthQueryApplicationOutputRole for OpenedAccount {
///     type Schema = Ledger;
///     type Contract = OpenOutputs;
///     type Entity = Account;
///     type Action = WorthQueryCreateOutput;
///     type Cardinality = WorthQueryAtMostOneOutput;
///     const NAME: &'static str = "opened-account";
/// }
///
/// const _: () = OpenedAccount::DECLARED;
/// assert_eq!(
///     OpenedAccount::DESCRIPTOR.cardinality(),
///     ApplicationMutationOutputRoleCardinality::AtMostOne,
/// );
/// ```
pub trait WorthQueryApplicationOutputRole: 'static {
    type Schema: ApplicationSchema;
    type Contract: ApplicationMutationOutputContract<Self::Schema>;
    type Entity: ApplicationEntityMarkerIdentity<Self::Schema> + 'static;
    type Action: WorthQueryApplicationOutputAction;
    type Cardinality: WorthQueryApplicationOutputCardinality;
    /// The role name. It describes correspondence; it never supplies or
    /// reconstructs an entity identity.
    const NAME: &'static str;
}

/// What an output role is as its contract declares it. Implemented for every
/// [`WorthQueryApplicationOutputRole`] and for nothing else, so the descriptor
/// is always derived from the declaration.
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::application_operation::{
///     ApplicationMutationOutputRoleDescriptor, WorthQueryApplicationDeclaredOutputRole,
/// };
///
/// struct Forged;
/// impl WorthQueryApplicationDeclaredOutputRole for Forged {
///     const DESCRIPTOR: ApplicationMutationOutputRoleDescriptor = unimplemented!();
///     const DECLARED: () = ();
/// }
/// ```
pub trait WorthQueryApplicationDeclaredOutputRole: WorthQueryApplicationOutputRole {
    /// The installed form of the role, for the contract's `ROLES`.
    const DESCRIPTOR: ApplicationMutationOutputRoleDescriptor;
    /// Evaluates only when the contract's `ROLES` lists this role with the
    /// same name, entity, posture and cardinality. Query use sites evaluate
    /// it in an inline `const` block, so a disagreeing use fails to compile.
    const DECLARED: ();
}

impl<Role> WorthQueryApplicationDeclaredOutputRole for Role
where
    Role: WorthQueryApplicationOutputRole,
{
    const DESCRIPTOR: ApplicationMutationOutputRoleDescriptor =
        ApplicationMutationOutputRoleDescriptor::declared(
            Role::NAME,
            <Role::Entity as ApplicationEntityMarkerIdentity<Role::Schema>>::IDENTIFIER,
            <Role::Action as WorthQueryApplicationOutputAction>::POSTURE,
            <Role::Cardinality as WorthQueryApplicationOutputCardinality>::CARDINALITY,
        );

    const DECLARED: () = {
        let used = <Role as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR;
        let roles = <Role::Contract as ApplicationMutationOutputContract<Role::Schema>>::ROLES;
        let mut index = 0;
        let mut declared = false;
        while index < roles.len() {
            let role = roles[index];
            if same_text(role.name(), used.name()) {
                if !same_text(role.entity(), used.entity()) {
                    panic!("the output role is declared for a different entity");
                }
                if ApplicationMutationOutputPostureSet::one(role.posture()).bits()
                    != ApplicationMutationOutputPostureSet::one(used.posture()).bits()
                {
                    panic!("the output role is declared with a different posture");
                }
                if role.cardinality().admits_absence() != used.cardinality().admits_absence() {
                    panic!("the output role is declared with a different cardinality");
                }
                declared = true;
            }
            index += 1;
        }
        if !declared {
            panic!("the output role is not declared in its contract's ROLES");
        }
    };
}

/// Byte equality usable in a const context.
const fn same_text(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}
