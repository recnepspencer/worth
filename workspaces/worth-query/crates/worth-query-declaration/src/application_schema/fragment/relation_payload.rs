//! A relation's payload aspects are members of its outgoing relation declaration.
use std::marker::PhantomData;

pub struct WithAspects<R, Members>(PhantomData<fn() -> (R, Members)>);
#[doc(hidden)]
pub struct PayloadWitness<W>(PhantomData<W>);
#[doc(hidden)]
pub trait PayloadMembers<R, Position> {
    type Aspects;
}
