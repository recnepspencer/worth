//! Pure original-input selectors for one installed homogeneous batch.
use super::{ApplicationFixedActionDelivery, ApplicationQueryBatchProjection};
use crate::{
    application_operation::ApplicationMutationBinding, application_query::ApplicationQueryBinding,
    application_schema::ApplicationSchema,
};

/// Declares an ordered fixed batch whose selectors borrow the original input.
/// Cardinality and the concrete implementation belong to installed meaning;
/// selectors grant no read authority and callers cannot substitute them.
///
/// ```compile_fail,E0308
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationBinding, application_query::ApplicationQueryBinding, application_program::ApplicationFixedBatchDelivery};
/// fn truncate<S, B, D>(input: &B::Input) -> [<D::Read as ApplicationQueryBinding<S>>::Input; 1]
/// where S: ApplicationSchema, B: ApplicationMutationBinding<S>, D: ApplicationFixedBatchDelivery<S, B, 2> {
///     D::read_inputs(input)
/// }
/// ```
/// ```no_run
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationBinding, application_query::ApplicationQueryBinding, application_program::ApplicationFixedBatchDelivery};
/// fn original<S, B, D>(input: &B::Input) -> [<D::Read as ApplicationQueryBinding<S>>::Input; 2]
/// where S: ApplicationSchema, B: ApplicationMutationBinding<S>, D: ApplicationFixedBatchDelivery<S, B, 2> {
///     D::read_inputs(input)
/// }
/// ```
pub trait ApplicationFixedBatchDelivery<S, B, const N: usize>:
    ApplicationFixedActionDelivery<S, B, Projection = ApplicationQueryBatchProjection<Self::Read, N>>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
{
    type Read: ApplicationQueryBinding<S>;
    fn read_inputs(input: &B::Input) -> [<Self::Read as ApplicationQueryBinding<S>>::Input; N];
}
