//! Output-role tokens. A token is the declaration of one output role: the
//! binding's output contract lists its `descriptor()`, and the handler writes
//! and reads through the same token, so the cardinality, posture and entity a
//! commit binds are the ones the binding declares.

use std::fmt;
use std::marker::PhantomData;

use super::output::{
    ApplicationMutationOutputPosture, ApplicationMutationOutputRoleCardinality,
    ApplicationMutationOutputRoleDescriptor,
};
use crate::application_schema::{ApplicationEntityMarkerIdentity, ApplicationSchema};

mod family;
mod name;

pub use family::{WorthQueryApplicationOutputMemberRole, WorthQueryApplicationOutputRoleFamily};
pub use name::WorthQueryApplicationOutputRoleNameDenial;

mod action {
    pub trait Sealed {}
}

/// Output action: the role names a record the mutation keeps, neither
/// creating nor retiring it.
pub struct WorthQueryPreserveOutput;
/// Output action: the role names a record the mutation creates.
pub struct WorthQueryCreateOutput;
/// Output action: the role names a record the mutation retires.
pub struct WorthQueryRetireOutput;

/// The action an output role performs on its record: implemented only by
/// [`WorthQueryPreserveOutput`], [`WorthQueryCreateOutput`] and
/// [`WorthQueryRetireOutput`], and sealed against other implementations.
pub trait WorthQueryApplicationOutputAction: action::Sealed {
    const POSTURE: ApplicationMutationOutputPosture;
}

impl action::Sealed for WorthQueryPreserveOutput {}
impl WorthQueryApplicationOutputAction for WorthQueryPreserveOutput {
    const POSTURE: ApplicationMutationOutputPosture = ApplicationMutationOutputPosture::Preserve;
}

impl action::Sealed for WorthQueryCreateOutput {}
impl WorthQueryApplicationOutputAction for WorthQueryCreateOutput {
    const POSTURE: ApplicationMutationOutputPosture = ApplicationMutationOutputPosture::Create;
}

impl action::Sealed for WorthQueryRetireOutput {}
impl WorthQueryApplicationOutputAction for WorthQueryRetireOutput {
    const POSTURE: ApplicationMutationOutputPosture = ApplicationMutationOutputPosture::Retire;
}

macro_rules! fixed_output_role_token {
    ($(#[$doc:meta])* $token:ident, $cardinality:ident) => {
        $(#[$doc])*
        pub struct $token<Binding, Entity, Action> {
            name: &'static str,
            entity: &'static str,
            _marker: PhantomData<fn() -> (Binding, Entity, Action)>,
        }

        impl<Binding, Entity, Action> $token<Binding, Entity, Action>
        where
            Action: WorthQueryApplicationOutputAction,
        {
            /// Declare the role `name` for `Entity` in `Schema`. List the
            /// token's [`Self::descriptor`] in the binding's output contract.
            pub const fn for_entity<Schema>(name: &'static str) -> Self
            where
                Schema: ApplicationSchema,
                Entity: ApplicationEntityMarkerIdentity<Schema>,
            {
                Self {
                    name,
                    entity: Entity::IDENTIFIER,
                    _marker: PhantomData,
                }
            }

            /// The installed form of this declaration, for the output
            /// contract's `ROLES`.
            pub const fn descriptor(&self) -> ApplicationMutationOutputRoleDescriptor {
                ApplicationMutationOutputRoleDescriptor::declared(
                    self.name,
                    self.entity,
                    Action::POSTURE,
                    ApplicationMutationOutputRoleCardinality::$cardinality,
                )
            }
        }

        impl<Binding, Entity, Action> $token<Binding, Entity, Action> {
            pub const fn name(&self) -> &'static str {
                self.name
            }
        }

        impl<Binding, Entity, Action> Clone for $token<Binding, Entity, Action> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<Binding, Entity, Action> Copy for $token<Binding, Entity, Action> {}

        impl<Binding, Entity, Action> PartialEq for $token<Binding, Entity, Action> {
            fn eq(&self, other: &Self) -> bool {
                self.name == other.name && self.entity == other.entity
            }
        }

        impl<Binding, Entity, Action> Eq for $token<Binding, Entity, Action> {}

        impl<Binding, Entity, Action> fmt::Debug for $token<Binding, Entity, Action> {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_struct(stringify!($token))
                    .field("name", &self.name)
                    .field("entity", &self.entity)
                    .finish()
            }
        }
    };
}

fixed_output_role_token!(
    /// The declaration of a fixed output role every commit binds exactly
    /// once. The name describes correspondence; it never supplies or
    /// reconstructs an entity identity. Reading through it yields the entity
    /// itself.
    ///
    /// A token comes only from its declaration, never from a bare name:
    ///
    /// ```compile_fail,E0599
    /// use worth_query_declaration::facade::application_operation::{
    ///     WorthQueryApplicationOutputRole, WorthQueryCreateOutput,
    /// };
    ///
    /// let _ = WorthQueryApplicationOutputRole::<(), (), WorthQueryCreateOutput>::from_static("created");
    /// ```
    ///
    /// ```compile_fail,E0599
    /// use worth_query_declaration::facade::application_operation::{
    ///     WorthQueryApplicationOutputRole, WorthQueryCreateOutput,
    /// };
    ///
    /// let _ = WorthQueryApplicationOutputRole::<(), (), WorthQueryCreateOutput>::try_new("created");
    /// ```
    ///
    /// An at-most-one declaration is not an exactly-one token:
    ///
    /// ```compile_fail,E0308
    /// use worth_query_declaration::facade::application_operation::{
    ///     WorthQueryApplicationOptionalOutputRole, WorthQueryApplicationOutputRole,
    ///     WorthQueryCreateOutput,
    /// };
    /// use worth_query_declaration::facade::application_schema::{
    ///     ApplicationEntityMarkerIdentity, ApplicationSchema,
    /// };
    ///
    /// const fn closing<Schema, Entity>() -> WorthQueryApplicationOutputRole<(), Entity, WorthQueryCreateOutput>
    /// where
    ///     Schema: ApplicationSchema,
    ///     Entity: ApplicationEntityMarkerIdentity<Schema>,
    /// {
    ///     WorthQueryApplicationOptionalOutputRole::for_entity::<Schema>("closing")
    /// }
    /// ```
    ///
    /// Declaring a role and listing it is ordinary:
    ///
    /// ```
    /// use worth_query_declaration::facade::application_operation::{
    ///     ApplicationMutationOutputRoleCardinality, ApplicationMutationOutputRoleDescriptor,
    ///     WorthQueryApplicationOutputRole, WorthQueryCreateOutput,
    /// };
    /// use worth_query_declaration::facade::application_schema::{
    ///     ApplicationEntityMarkerIdentity, ApplicationSchema,
    /// };
    ///
    /// const fn anchor<Schema, Entity>() -> WorthQueryApplicationOutputRole<(), Entity, WorthQueryCreateOutput>
    /// where
    ///     Schema: ApplicationSchema,
    ///     Entity: ApplicationEntityMarkerIdentity<Schema>,
    /// {
    ///     WorthQueryApplicationOutputRole::for_entity::<Schema>("anchor")
    /// }
    ///
    /// fn listed<Schema, Entity>() -> ApplicationMutationOutputRoleDescriptor
    /// where
    ///     Schema: ApplicationSchema,
    ///     Entity: ApplicationEntityMarkerIdentity<Schema>,
    /// {
    ///     let descriptor = anchor::<Schema, Entity>().descriptor();
    ///     assert_eq!(descriptor.cardinality(), ApplicationMutationOutputRoleCardinality::ExactlyOne);
    ///     descriptor
    /// }
    /// ```
    WorthQueryApplicationOutputRole,
    ExactlyOne
);

fixed_output_role_token!(
    /// The declaration of a fixed output role a commit binds at most once.
    /// Reading through it yields `Option`, so an absent role is a value,
    /// never a denial.
    WorthQueryApplicationOptionalOutputRole,
    AtMostOne
);
