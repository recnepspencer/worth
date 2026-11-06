//! Optional facts and outgoing relations preserve creation completeness at the declaring entity's owner.
use super::{Aspect, Entity, Exclusive};
use std::marker::PhantomData;

pub struct Extend<E: Entity, Members: OptionalExtensionMembers<E>, Relations = ()>(
    PhantomData<fn() -> (E, Members, Relations)>,
);

#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "extension `{Self}` of entity `{E}` contains a required member",
    note = "Extend accepts only Option<Aspect> and Option<Exclusive<Group>> members"
)]
pub trait OptionalExtensionMembers<E>: private::Optional<E> {}
impl<E: Entity, T: private::Optional<E>> OptionalExtensionMembers<E> for T {}
mod private {
    #[diagnostic::on_unimplemented(
        message = "`{Self}` is a required member in Extend for entity `{Owner}`",
        note = "wrap the aspect or Exclusive<Group> in Option; extension members must be optional"
    )]
    pub trait Optional<Owner> {}
}
impl<Owner, A: Aspect> private::Optional<Owner> for Option<A> {}
impl<Owner, G: super::OneOf> private::Optional<Owner> for Option<Exclusive<G>> {}
impl<Owner> private::Optional<Owner> for () {}
macro_rules! optional_tuples {
    ($($member:ident),+) => {
        impl<Owner, $($member: private::Optional<Owner>),+> private::Optional<Owner> for ($($member,)+) {}
    };
}
optional_tuples!(A);
optional_tuples!(A, B);
optional_tuples!(A, B, C);
optional_tuples!(A, B, C, D);
optional_tuples!(A, B, C, D, E);
optional_tuples!(A, B, C, D, E, F);
optional_tuples!(A, B, C, D, E, F, G);
optional_tuples!(A, B, C, D, E, F, G, H);
