use std::fmt;
use std::marker::PhantomData;

use super::super::output::{
    ApplicationMutationOutputPostureSet, ApplicationMutationOutputRoleFamilyDescriptor,
};
use super::{WorthQueryApplicationOutputAction, WorthQueryApplicationOutputRoleNameDenial};
use crate::application_schema::{ApplicationEntityMarkerIdentity, ApplicationSchema};

/// The declaration of one output-role family: a variable number of outputs
/// whose role names share one prefix. The binding's output contract lists
/// its [`Self::descriptor`]; the handler names each output through
/// [`Self::member`], and a prior-output read resolves the whole family.
pub struct WorthQueryApplicationOutputRoleFamily<Binding, Entity> {
    prefix: &'static str,
    entity: &'static str,
    postures: ApplicationMutationOutputPostureSet,
    minimum: usize,
    _marker: PhantomData<fn() -> (Binding, Entity)>,
}

impl<Binding, Entity> WorthQueryApplicationOutputRoleFamily<Binding, Entity> {
    /// Declare the family `prefix` for `Entity` in `Schema`: its members take
    /// one of `postures`, and a completed mutation binds at least `minimum`.
    pub const fn for_entity<Schema>(
        prefix: &'static str,
        postures: ApplicationMutationOutputPostureSet,
        minimum: usize,
    ) -> Self
    where
        Schema: ApplicationSchema,
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        Self {
            prefix,
            entity: Entity::IDENTIFIER,
            postures,
            minimum,
            _marker: PhantomData,
        }
    }

    /// The installed form of this declaration, for the output contract's
    /// `ROLE_FAMILIES`.
    pub const fn descriptor(&self) -> ApplicationMutationOutputRoleFamilyDescriptor {
        ApplicationMutationOutputRoleFamilyDescriptor::declared(
            self.prefix,
            self.entity,
            self.postures,
            self.minimum,
        )
    }

    pub const fn prefix(&self) -> &'static str {
        self.prefix
    }

    /// The member of this family named by `suffix`. A member exists only
    /// through its family, so its name always carries the family prefix.
    pub fn member<Action>(
        &self,
        suffix: &str,
    ) -> Result<
        WorthQueryApplicationOutputMemberRole<Binding, Entity, Action>,
        WorthQueryApplicationOutputRoleNameDenial,
    >
    where
        Action: WorthQueryApplicationOutputAction,
    {
        if suffix.is_empty() {
            return Err(WorthQueryApplicationOutputRoleNameDenial::Empty);
        }
        let name = format!("{}{suffix}", self.prefix);
        WorthQueryApplicationOutputRoleNameDenial::validate(&name)?;
        Ok(WorthQueryApplicationOutputMemberRole {
            name,
            _marker: PhantomData,
        })
    }
}

impl<Binding, Entity> Clone for WorthQueryApplicationOutputRoleFamily<Binding, Entity> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Binding, Entity> Copy for WorthQueryApplicationOutputRoleFamily<Binding, Entity> {}

impl<Binding, Entity> PartialEq for WorthQueryApplicationOutputRoleFamily<Binding, Entity> {
    fn eq(&self, other: &Self) -> bool {
        self.descriptor() == other.descriptor()
    }
}

impl<Binding, Entity> Eq for WorthQueryApplicationOutputRoleFamily<Binding, Entity> {}

impl<Binding, Entity> fmt::Debug for WorthQueryApplicationOutputRoleFamily<Binding, Entity> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorthQueryApplicationOutputRoleFamily")
            .field("prefix", &self.prefix)
            .field("entity", &self.entity)
            .field("postures", &self.postures)
            .field("minimum", &self.minimum)
            .finish()
    }
}

/// One member of a declared output-role family, bound exactly once by the
/// commit that names it. Only [`WorthQueryApplicationOutputRoleFamily::member`]
/// makes one:
///
/// ```compile_fail,E0599
/// use worth_query_declaration::facade::application_operation::{
///     WorthQueryApplicationOutputMemberRole, WorthQueryCreateOutput,
/// };
///
/// let _ = WorthQueryApplicationOutputMemberRole::<(), (), WorthQueryCreateOutput>::try_new("created.a");
/// ```
///
/// ```compile_fail,E0451
/// use std::marker::PhantomData;
/// use worth_query_declaration::facade::application_operation::{
///     WorthQueryApplicationOutputMemberRole, WorthQueryCreateOutput,
/// };
///
/// let _ = WorthQueryApplicationOutputMemberRole::<(), (), WorthQueryCreateOutput> {
///     name: "created.a".to_owned(),
///     _marker: PhantomData,
/// };
/// ```
pub struct WorthQueryApplicationOutputMemberRole<Binding, Entity, Action> {
    name: String,
    _marker: PhantomData<fn() -> (Binding, Entity, Action)>,
}

impl<Binding, Entity, Action> WorthQueryApplicationOutputMemberRole<Binding, Entity, Action> {
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<Binding, Entity, Action> Clone
    for WorthQueryApplicationOutputMemberRole<Binding, Entity, Action>
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            _marker: PhantomData,
        }
    }
}

impl<Binding, Entity, Action> PartialEq
    for WorthQueryApplicationOutputMemberRole<Binding, Entity, Action>
{
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<Binding, Entity, Action> Eq
    for WorthQueryApplicationOutputMemberRole<Binding, Entity, Action>
{
}

impl<Binding, Entity, Action> fmt::Debug
    for WorthQueryApplicationOutputMemberRole<Binding, Entity, Action>
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorthQueryApplicationOutputMemberRole")
            .field("name", &self.name)
            .finish()
    }
}
