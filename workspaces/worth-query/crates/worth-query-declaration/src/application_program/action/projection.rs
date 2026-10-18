//! Closed output delivery projection shapes derive installed read meaning.
use crate::{
    application_query::{ApplicationQueryBinding, ApplicationQueryBindingDescriptor},
    application_schema::ApplicationSchema,
};
use std::marker::PhantomData;
mod sealed {
    pub trait Projection {}
}
/// Explicit required-output delivery with no query projection or selector.
pub struct ApplicationNoProjection;
/// One installed typed query projection.
pub struct ApplicationQueryProjection<Q>(PhantomData<fn() -> Q>);
/// One installed query binding with an exact bounded ordered selector array.
pub struct ApplicationQueryBatchProjection<Q, const N: usize>(PhantomData<fn() -> Q>);
impl sealed::Projection for ApplicationNoProjection {}
impl<Q> sealed::Projection for ApplicationQueryProjection<Q> {}
impl<Q, const N: usize> sealed::Projection for ApplicationQueryBatchProjection<Q, N> {}
/// Sealed projection shapes contribute their actual query descriptor, if any.
pub trait ApplicationOutputProjection<S: ApplicationSchema>: sealed::Projection + 'static {
    fn descriptor() -> Option<ApplicationQueryBindingDescriptor>;
    fn batch_items() -> Option<usize> {
        None
    }
}
impl<S: ApplicationSchema, Q: ApplicationQueryBinding<S> + 'static, const N: usize>
    ApplicationOutputProjection<S> for ApplicationQueryBatchProjection<Q, N>
{
    fn descriptor() -> Option<ApplicationQueryBindingDescriptor> {
        Some(Q::descriptor())
    }
    fn batch_items() -> Option<usize> {
        Some(N)
    }
}
impl<S: ApplicationSchema> ApplicationOutputProjection<S> for ApplicationNoProjection {
    fn descriptor() -> Option<ApplicationQueryBindingDescriptor> {
        None
    }
}
impl<S: ApplicationSchema, Q: ApplicationQueryBinding<S> + 'static> ApplicationOutputProjection<S>
    for ApplicationQueryProjection<Q>
{
    fn descriptor() -> Option<ApplicationQueryBindingDescriptor> {
        Some(Q::descriptor())
    }
}
