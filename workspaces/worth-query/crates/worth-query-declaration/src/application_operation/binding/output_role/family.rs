use super::super::output::{
    ApplicationMutationOutputContract, ApplicationMutationOutputPostureSet,
    ApplicationMutationOutputRoleFamilyDescriptor,
};
use super::{same_text, WorthQueryApplicationOutputRoleNameDenial};
use crate::application_schema::{ApplicationEntityMarkerIdentity, ApplicationSchema};

/// The declaration of one output-role family of an output contract: a
/// variable number of outputs whose role names share [`Self::PREFIX`].
///
/// The contract lists the derived
/// [`WorthQueryApplicationDeclaredOutputRoleFamily::DESCRIPTOR`] in its
/// `ROLE_FAMILIES`. Each member takes one of [`Self::POSTURES`], and a
/// completed mutation binds at least [`Self::MINIMUM`] of them. A member's
/// suffix is chosen at run time and validated when it is named.
pub trait WorthQueryApplicationOutputRoleFamily: 'static {
    type Schema: ApplicationSchema;
    type Contract: ApplicationMutationOutputContract<Self::Schema>;
    type Entity: ApplicationEntityMarkerIdentity<Self::Schema> + 'static;
    const PREFIX: &'static str;
    const POSTURES: ApplicationMutationOutputPostureSet;
    const MINIMUM: usize;
}

/// What an output-role family is as its contract declares it. Implemented for
/// every [`WorthQueryApplicationOutputRoleFamily`] and for nothing else.
pub trait WorthQueryApplicationDeclaredOutputRoleFamily:
    WorthQueryApplicationOutputRoleFamily
{
    /// The installed form of the family, for the contract's `ROLE_FAMILIES`.
    const DESCRIPTOR: ApplicationMutationOutputRoleFamilyDescriptor;
    /// Evaluates only when the contract's `ROLE_FAMILIES` lists this family
    /// with the same prefix, entity, postures and minimum. Query use sites
    /// evaluate it in an inline `const` block, so a disagreeing use fails to
    /// compile.
    const DECLARED: ();

    /// The role name of the member named by `suffix`. A member exists only
    /// through its family, so its name always carries the family prefix.
    fn member_name(suffix: &str) -> Result<String, WorthQueryApplicationOutputRoleNameDenial>;
}

impl<Family> WorthQueryApplicationDeclaredOutputRoleFamily for Family
where
    Family: WorthQueryApplicationOutputRoleFamily,
{
    const DESCRIPTOR: ApplicationMutationOutputRoleFamilyDescriptor =
        ApplicationMutationOutputRoleFamilyDescriptor::declared(
            Family::PREFIX,
            <Family::Entity as ApplicationEntityMarkerIdentity<Family::Schema>>::IDENTIFIER,
            Family::POSTURES,
            Family::MINIMUM,
        );

    const DECLARED: () = {
        let used = <Family as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR;
        let families =
            <Family::Contract as ApplicationMutationOutputContract<Family::Schema>>::ROLE_FAMILIES;
        let mut index = 0;
        let mut declared = false;
        while index < families.len() {
            let family = families[index];
            if same_text(family.prefix(), used.prefix()) {
                if !same_text(family.entity(), used.entity()) {
                    panic!("the output-role family is declared for a different entity");
                }
                if family.postures().bits() != used.postures().bits() {
                    panic!("the output-role family is declared with different postures");
                }
                if family.minimum() != used.minimum() {
                    panic!("the output-role family is declared with a different minimum");
                }
                declared = true;
            }
            index += 1;
        }
        if !declared {
            panic!("the output-role family is not declared in its contract's ROLE_FAMILIES");
        }
    };

    fn member_name(suffix: &str) -> Result<String, WorthQueryApplicationOutputRoleNameDenial> {
        if suffix.is_empty() {
            return Err(WorthQueryApplicationOutputRoleNameDenial::Empty);
        }
        let name = format!("{}{suffix}", Family::PREFIX);
        WorthQueryApplicationOutputRoleNameDenial::validate(&name)?;
        Ok(name)
    }
}
