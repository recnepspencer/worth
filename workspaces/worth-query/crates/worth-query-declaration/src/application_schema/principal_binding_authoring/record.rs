//! Principal roles select checked fields from records, including indexed paths.
use super::*;
use crate::application_schema::fragment::{Aspect, Entity, Fragment, In, RecordField, Scalar};
use crate::application_schema::ApplicationSchemaDeclarationDenial as Denial;
use std::any::TypeId;

impl<S, E: Entity> ApplicationPrincipalMappingIdentityRequirement<S, E> {
    pub fn from_record<A: Aspect>(
        field: RecordField<E, A, WorthQueryExternalPrincipalIdentity>,
    ) -> Result<Self, Denial> {
        posture(&field, false, true)?;
        Ok(Self {
            entity: E::NAME,
            aspect: A::NAME,
            field: field.locus().field().to_owned(),
            _marker: PhantomData,
        })
    }
}
impl<S, E: Entity> ApplicationPrincipalMappingStatusRequirement<S, E> {
    pub fn from_record<A: Aspect>(
        field: RecordField<E, A, WorthQueryPrincipalMappingStatus>,
    ) -> Result<Self, Denial> {
        posture(&field, true, false)?;
        Ok(Self {
            aspect: A::NAME,
            field: field.locus().field().to_owned(),
            _marker: PhantomData,
        })
    }
}
impl<S, E: Entity, T: 'static, B: ApplicationIdentityScalarValueBinding<Value = T>>
    ApplicationPrincipalIdentityRequirement<S, E, T, B>
{
    pub fn from_record<F: Fragment, A, EP, AP>(
        _: F,
        field: RecordField<E, A, T>,
    ) -> Result<Self, Denial>
    where
        F::Bindings: Scalar<T, Binding = B>,
        E: In<F::Members, EP, F>,
        A: Aspect + In<E::Aspects, AP, E>,
    {
        posture(&field, false, true)?;
        if field.recipe().binding_type() != TypeId::of::<B>() {
            return Err(Denial::InvalidRecordField {
                field: field.locus().clone(),
            });
        }
        Ok(Self {
            aspect: A::NAME,
            field: field.locus().field().to_owned(),
            scalar_family: B::SCALAR_FAMILY,
            value_type: B::IDENTITY_NAME,
            binding_recipe: field.recipe().clone(),
            _marker: PhantomData,
        })
    }
}

fn posture<E, A, T>(
    field: &RecordField<E, A, T>,
    writable: bool,
    equality: bool,
) -> Result<(), Denial> {
    if field.writable() != writable || (equality && !field.equality_queryable()) {
        return Err(Denial::InvalidRecordField {
            field: field.locus().clone(),
        });
    }
    Ok(())
}
