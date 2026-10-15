use crate::collections::{NonEmpty, Pair};
use crate::composition::{AuthoritativeFamilyMember, CompositionFamilySymbol};
use crate::recipe::{Recipe, Unresolved};

pub fn recipe<T>(payload: T) -> Recipe<Unresolved, T> {
    Recipe::new(payload)
}

pub fn pair<T>(left: T, right: T) -> Pair<T> {
    Pair::new(left, right)
}

pub fn non_empty<T>(head: T, tail: Vec<T>) -> NonEmpty<T> {
    NonEmpty::new(head, tail)
}

pub fn sym<S>(symbol: S) -> CompositionFamilySymbol<S> {
    CompositionFamilySymbol::new(symbol)
}

pub fn member<A>(member: A) -> AuthoritativeFamilyMember<A> {
    AuthoritativeFamilyMember::new(member)
}
