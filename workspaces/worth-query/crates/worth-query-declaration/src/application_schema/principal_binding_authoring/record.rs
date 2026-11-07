//! Principal requirements retain selections; schema composition checks their posture.
use super::*;
use crate::application_schema::fragment::{Aspect, Entity, FieldSelection, Fragment, In, Scalar};

impl<S, E: Entity> ApplicationPrincipalMappingIdentityRequirement<S, E> {
    pub fn from_record<Field>(field: Field) -> Self
    where
        Field: super::super::fragment::RecordSelection<
            Entity = E,
            Value = WorthQueryExternalPrincipalIdentity,
        >,
    {
        Self::from_field(field)
    }
}
impl<S, E: Entity> ApplicationPrincipalMappingStatusRequirement<S, E> {
    pub fn from_record<Field>(field: Field) -> Self
    where
        Field: super::super::fragment::RecordSelection<
            Entity = E,
            Value = WorthQueryPrincipalMappingStatus,
        >,
    {
        Self::from_field(field)
    }
}
impl<S, E: Entity, T: 'static, B: ApplicationIdentityScalarValueBinding<Value = T>>
    ApplicationPrincipalIdentityRequirement<S, E, T, B>
{
    pub fn from_record<F: Fragment, A, EP, AP>(_: F, field: FieldSelection<F, E, A, T>) -> Self
    where
        F::Bindings: Scalar<T, Binding = B> + super::super::fragment::Record<A::Row>,
        A: Aspect + In<F::Members, (EP, AP), (F, E)>,
    {
        Self::from_field(field)
    }
}
