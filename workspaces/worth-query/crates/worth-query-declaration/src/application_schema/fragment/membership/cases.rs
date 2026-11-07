//! Both joined lists use the same sealed nested-position membership proof.
use super::{private, In, NestedPosition, Position};
use crate::application_schema::fragment::JoinCases;

macro_rules! branch {
    ($index:literal, $side:ident) => {
        impl<Left, Right, Selected, P, Kind, Owner>
            private::Sealed<
                JoinCases<Left, Right>,
                (NestedPosition<Position<$index>, P>, Kind),
                Owner,
            > for Selected
        where
            Selected: In<$side, (P, Kind), Owner>,
        {
        }
    };
}
branch!(0, Left);
branch!(1, Right);
