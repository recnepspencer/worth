use super::ApplicationContentBinding;
use crate::application_schema::{ApplicationFieldRef, ApplicationFieldUnit};
use std::marker::PhantomData;
use worth_foundational::facade::ContentRefId;

/// The existing field identity with an installed content-format interpretation.
pub struct ApplicationContentFieldRef<S, E, A, F, W, Q, U, C> {
    field: ApplicationFieldRef<S, E, A, F, ContentRefId, W, Q, U>,
    _format: PhantomData<fn() -> C>,
}
impl<S, E, A, F, W, Q, U, C> Copy for ApplicationContentFieldRef<S, E, A, F, W, Q, U, C> {}
impl<S, E, A, F, W, Q, U, C> Clone for ApplicationContentFieldRef<S, E, A, F, W, Q, U, C> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<S, E, A, F, W, Q, U, C> ApplicationContentFieldRef<S, E, A, F, W, Q, U, C>
where
    U: ApplicationFieldUnit,
{
    pub const fn field(&self) -> ApplicationFieldRef<S, E, A, F, ContentRefId, W, Q, U> {
        self.field
    }
}
/// A typed declaration witness; installation must match its exact format recipe.
///
/// Existing ContentRef fields can supply this witness:
/// ```
/// use worth_query_declaration::facade::application_schema::*;
/// use worth_foundational::facade::ContentRefId;
/// fn bind<S,E,A,F,W,Q,U,C:ApplicationContentBinding>(
///     field:ApplicationFieldRef<S,E,A,F,ContentRefId,W,Q,U>,format:C,
/// ) -> ApplicationContentFieldRef<S,E,A,F,W,Q,U,C> { content_field(field,format) }
/// ```
/// Another scalar cannot be reinterpreted as content:
/// ```compile_fail,E0308
/// use worth_query_declaration::facade::application_schema::*;
/// fn cannot_reinterpret<S,E,A,F,W,Q,U,C:ApplicationContentBinding>(
///     field:ApplicationFieldRef<S,E,A,F,u64,W,Q,U>,format:C,
/// ) { let _=content_field(field,format); }
/// ```
pub const fn content_field<S, E, A, F, W, Q, U, C: ApplicationContentBinding>(
    field: ApplicationFieldRef<S, E, A, F, ContentRefId, W, Q, U>,
    _format: C,
) -> ApplicationContentFieldRef<S, E, A, F, W, Q, U, C> {
    ApplicationContentFieldRef {
        field,
        _format: PhantomData,
    }
}
