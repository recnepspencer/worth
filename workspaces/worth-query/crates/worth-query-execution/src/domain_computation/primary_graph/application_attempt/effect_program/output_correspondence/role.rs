use worth_query_declaration::facade::application_operation::{
    ApplicationMutationOutputRoleCardinality, WorthQueryApplicationOptionalOutputRole,
    WorthQueryApplicationOutputAction, WorthQueryApplicationOutputMemberRole,
    WorthQueryApplicationOutputRole,
};

pub(in crate::domain_computation::primary_graph) mod fixed {
    use super::{ApplicationMutationOutputRoleCardinality, WorthQueryApplicationFixedOutputRole};

    /// The argument every sealed role operation takes. Only Query constructs
    /// it, so code outside Query cannot call them, even through a generic
    /// bound.
    pub struct Internal(());

    pub(in crate::domain_computation::primary_graph) const INTERNAL: Internal = Internal(());

    pub trait Sealed {
        /// The cardinality this token reads and writes as.
        fn cardinality(_: Internal) -> ApplicationMutationOutputRoleCardinality;

        /// The role name this token binds and reads.
        fn name(&self, _: Internal) -> &str;

        /// Shape one role lookup. An exactly-one token yields `None` when the
        /// role is unbound, which callers refuse; an at-most-one token always
        /// yields a read, carrying absence as its `None`.
        fn read<T>(
            bound: Option<T>,
            _: Internal,
        ) -> Option<<Self as WorthQueryApplicationFixedOutputRole>::Read<T>>
        where
            Self: WorthQueryApplicationFixedOutputRole;
    }
}

/// A typed output-role token: [`WorthQueryApplicationOutputRole`] for a fixed
/// role every commit binds, [`WorthQueryApplicationOptionalOutputRole`] for a
/// fixed role a commit may leave unbound, and
/// [`WorthQueryApplicationOutputMemberRole`] for one member of a declared role
/// family.
///
/// The token's cardinality decides what a read returns, so absence is typed:
/// `Read<T>` is `T` for an exactly-one token and `Option<T>` for an at-most-one
/// token. A fixed token is the role's declaration, so its cardinality, action
/// and entity are the ones the binding lists; Query still refuses a token the
/// installed binding does not declare.
///
/// The trait is sealed, and the operations Query uses to shape reads are not
/// callable outside Query, even through a generic bound:
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationFixedOutputRole;
///
/// fn cannot_shape_a_read<Role: WorthQueryApplicationFixedOutputRole>() -> Option<Role::Read<u8>> {
///     Role::read(Some(1))
/// }
/// ```
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationFixedOutputRole;
///
/// fn cannot_read_the_cardinality<Role: WorthQueryApplicationFixedOutputRole>() {
///     let _ = Role::cardinality();
/// }
/// ```
///
/// A read through an at-most-one token is an `Option`, never the entity
/// itself:
///
/// ```compile_fail,E0308
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryApplicationOptionalOutputRole, WorthQueryApplicationOutputCorrespondence,
///     WorthQueryApplicationOutputEntity, WorthQueryCreateOutput,
/// };
///
/// fn exactly_one<Binding: 'static, Entity: 'static>(
///     correspondence: &WorthQueryApplicationOutputCorrespondence,
///     role: WorthQueryApplicationOptionalOutputRole<Binding, Entity, WorthQueryCreateOutput>,
/// ) -> WorthQueryApplicationOutputEntity<Binding, Entity, WorthQueryCreateOutput> {
///     correspondence.entity(role).unwrap()
/// }
/// ```
///
/// Naming the read shape through the bound is ordinary:
///
/// ```
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationFixedOutputRole;
///
/// fn read_shape<Role: WorthQueryApplicationFixedOutputRole>(read: Role::Read<u8>) -> Role::Read<u8> {
///     read
/// }
/// ```
pub trait WorthQueryApplicationFixedOutputRole: fixed::Sealed {
    type Binding: 'static;
    type Entity: 'static;
    type Action: WorthQueryApplicationOutputAction;
    /// What reading this role yields when the value read is `T`.
    type Read<T>;
}

macro_rules! output_role_token {
    ($token:ident, $cardinality:ident, $read:ty, |$bound:ident| $shape:expr) => {
        impl<Binding, Entity, Action> fixed::Sealed for $token<Binding, Entity, Action>
        where
            Binding: 'static,
            Entity: 'static,
            Action: WorthQueryApplicationOutputAction,
        {
            fn cardinality(_: fixed::Internal) -> ApplicationMutationOutputRoleCardinality {
                ApplicationMutationOutputRoleCardinality::$cardinality
            }

            fn name(&self, _: fixed::Internal) -> &str {
                $token::name(self)
            }

            fn read<T>(
                $bound: Option<T>,
                _: fixed::Internal,
            ) -> Option<<Self as WorthQueryApplicationFixedOutputRole>::Read<T>> {
                $shape
            }
        }

        impl<Binding, Entity, Action> WorthQueryApplicationFixedOutputRole
            for $token<Binding, Entity, Action>
        where
            Binding: 'static,
            Entity: 'static,
            Action: WorthQueryApplicationOutputAction,
        {
            type Binding = Binding;
            type Entity = Entity;
            type Action = Action;
            type Read<T> = $read;
        }
    };
}

output_role_token!(WorthQueryApplicationOutputRole, ExactlyOne, T, |bound| {
    bound
});
output_role_token!(
    WorthQueryApplicationOptionalOutputRole,
    AtMostOne,
    Option<T>,
    |bound| Some(bound)
);
output_role_token!(
    WorthQueryApplicationOutputMemberRole,
    ExactlyOne,
    T,
    |bound| bound
);
