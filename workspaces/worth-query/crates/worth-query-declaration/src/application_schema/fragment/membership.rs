//! Tuple position is inferred from a value marker, never authored by a caller.

use std::marker::PhantomData;

#[diagnostic::on_unimplemented(message = "`{Self}` is not an aspect of `{Owner}`")]
pub trait In<Members, Witness, Owner>: private::Sealed<Members, Witness, Owner> {}
impl<T, Members, Witness, Owner> In<Members, Witness, Owner> for T where
    T: private::Sealed<Members, Witness, Owner>
{
}
mod private {
    pub trait Sealed<Members, Witness, Owner> {}
}

#[doc(hidden)]
pub struct Position<const N: usize>(PhantomData<()>);

macro_rules! tuple_positions {
    ($($member:ident:$position:literal),+) => {
        tuple_positions!(@each (); $($member:$position),+);
    };
    (@each ($($before:ident,)*); $member:ident:$position:literal $(,$rest:ident:$next:literal)*) => {
        impl<$($before,)* $member, $($rest,)* Owner> private::Sealed<($($before,)* $member, $($rest,)*), (Position<$position>, super::lowering::RequiredWitness), Owner> for $member {}
        impl<$($before,)* $member, $($rest,)* Owner> private::Sealed<($($before,)* Option<$member>, $($rest,)*), (Position<$position>, super::lowering::OptionalWitness), Owner> for $member {}
        impl<$($before,)* GroupType: super::OneOf, $($rest,)* Owner> private::Sealed<($($before,)* super::Exclusive<GroupType>, $($rest,)*), (Position<$position>, super::one_of::GroupWitness), Owner> for GroupType {}
        impl<$($before,)* GroupType: super::OneOf, $($rest,)* Owner> private::Sealed<($($before,)* Option<super::Exclusive<GroupType>>, $($rest,)*), (Position<$position>, super::one_of::OptionalGroupWitness), Owner> for GroupType {}
        impl<$($before,)* GroupType: super::OneOf, CaseType, CasePosition, $($rest,)* Owner> private::Sealed<($($before,)* super::Exclusive<GroupType>, $($rest,)*), (Position<$position>, super::one_of::CaseWitness<CasePosition>), Owner> for CaseType
        where CaseType: In<GroupType::Cases, CasePosition, Owner> {}
        impl<$($before,)* GroupType: super::OneOf, CaseType, CasePosition, $($rest,)* Owner> private::Sealed<($($before,)* Option<super::Exclusive<GroupType>>, $($rest,)*), (Position<$position>, super::one_of::OptionalCaseWitness<CasePosition>), Owner> for CaseType
        where CaseType: In<GroupType::Cases, CasePosition, Owner> {}
        tuple_positions!(@each ($($before,)* $member,); $($rest:$next),*);
    };
    (@each ($($before:ident,)*);) => {};
}

tuple_positions!(A:0);
tuple_positions!(A:0, B:1);
tuple_positions!(A:0, B:1, C:2);
tuple_positions!(A:0, B:1, C:2, D:3);
tuple_positions!(A:0, B:1, C:2, D:3, E:4);
tuple_positions!(A:0, B:1, C:2, D:3, E:4, F:5);
tuple_positions!(A:0, B:1, C:2, D:3, E:4, F:5, G:6);
tuple_positions!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7);
