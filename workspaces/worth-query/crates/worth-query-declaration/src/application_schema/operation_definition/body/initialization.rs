//! Scalar initializer authoring for root and child creation sites.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial,
    ApplicationOperationBodyCreatedField as Field,
};
use crate::application_schema::fragment::RecordSelection;

pub(super) fn literal<Selection: RecordSelection>(
    field: Selection,
    value: Selection::Value,
) -> Result<Field, Denial> {
    let checked = super::record_authoring::resolve(field)?;
    Ok(Field::Literal {
        field: checked.locus().clone(),
        binding: checked.recipe().binding_identity().clone(),
        value: checked.encode(&value)?,
    })
}
pub(super) fn input<Selection: RecordSelection>(
    field: Selection,
    member: &str,
    optional: bool,
) -> Result<Field, Denial> {
    if member.is_empty() || member.trim() != member {
        return Err(Denial::InvalidInputMember);
    }
    let checked = super::record_authoring::resolve(field)?;
    if optional
        && checked.presence() != crate::application_schema::ApplicationFieldPresence::Optional
    {
        return Err(Denial::IncompatibleSourceBinding);
    }
    let recipe = checked.recipe();
    Ok(if optional {
        Field::OptionalInput {
            field: checked.locus().clone(),
            binding: recipe.binding_identity().clone(),
            scalar_family: recipe.scalar_family(),
            member: member.to_owned(),
        }
    } else {
        Field::Input {
            field: checked.locus().clone(),
            binding: recipe.binding_identity().clone(),
            scalar_family: recipe.scalar_family(),
            member: member.to_owned(),
        }
    })
}
