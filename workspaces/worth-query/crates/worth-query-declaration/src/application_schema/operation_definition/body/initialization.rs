//! One scalar initializer authoring funnel for root and child creation sites.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial,
    ApplicationOperationBodyCreatedField as Field,
};
use crate::application_schema::{
    ApplicationFieldBindingLocus as Locus, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationScalarValueBinding, DeclaredApplicationFieldValue, OperationWrites,
};
pub(super) fn literal<S, O, E, A, F, V, W, Q, U>(
    field: ApplicationFieldRef<S, E, A, F, V, W, Q, U>,
    value: V,
) -> Result<Field, Denial>
where
    F: OperationWrites<O> + DeclaredApplicationFieldValue<Value = V>,
    U: ApplicationFieldUnit,
{
    F::Binding::validate(&value)
        .map_err(crate::application_schema::ApplicationValueEncodeDenial::from)?;
    Ok(Field::Literal {
        field: Locus::new(field.entity(), field.aspect(), field.field()),
        binding: F::Binding::IDENTITY,
        value: F::Binding::encode(&value)?,
    })
}
pub(super) fn input<S, O, E, A, F, V, W, Q, U>(
    field: ApplicationFieldRef<S, E, A, F, V, W, Q, U>,
    member: &str,
    optional: bool,
) -> Result<Field, Denial>
where
    F: OperationWrites<O> + DeclaredApplicationFieldValue<Value = V>,
    U: ApplicationFieldUnit,
{
    if member.is_empty() || member.trim() != member {
        return Err(Denial::InvalidInputMember);
    }
    let locus = Locus::new(field.entity(), field.aspect(), field.field());
    Ok(if optional {
        Field::OptionalInput {
            field: locus,
            binding: F::Binding::IDENTITY,
            scalar_family: F::Binding::SCALAR_FAMILY,
            member: member.to_owned(),
        }
    } else {
        Field::Input {
            field: locus,
            binding: F::Binding::IDENTITY,
            scalar_family: F::Binding::SCALAR_FAMILY,
            member: member.to_owned(),
        }
    })
}
