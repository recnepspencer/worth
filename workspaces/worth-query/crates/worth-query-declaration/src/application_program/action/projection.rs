//! Closed fixed delivery projection shapes derive installed read meaning.
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
impl sealed::Projection for ApplicationNoProjection {}
impl<Q> sealed::Projection for ApplicationQueryProjection<Q> {}
/// Sealed projection shapes contribute their actual query descriptor, if any.
pub trait ApplicationFixedProjection<S: ApplicationSchema>: sealed::Projection + 'static {
    fn descriptor() -> Option<ApplicationQueryBindingDescriptor>;
}
impl<S: ApplicationSchema> ApplicationFixedProjection<S> for ApplicationNoProjection {
    fn descriptor() -> Option<ApplicationQueryBindingDescriptor> {
        None
    }
}
impl<S: ApplicationSchema, Q: ApplicationQueryBinding<S> + 'static> ApplicationFixedProjection<S>
    for ApplicationQueryProjection<Q>
{
    fn descriptor() -> Option<ApplicationQueryBindingDescriptor> {
        Some(Q::descriptor())
    }
}
