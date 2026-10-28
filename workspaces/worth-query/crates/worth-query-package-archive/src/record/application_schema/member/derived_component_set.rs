//! Explicit versioned derived-source component meaning; legacy38 is unchanged.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationDerivedComponentSetMeaning;

pub(super) fn require_shape(
    definition: &ApplicationDerivedComponentSetMeaning,
) -> Result<(), Denial> {
    definition
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))
}
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    definition: &ApplicationDerivedComponentSetMeaning,
) -> Result<(), Denial> {
    output.u16(1)?;
    output.text(&definition.name)?;
    super::derived_relation::write_member_domain(output, &definition.member)?;
    output.text(&definition.relation)
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaMember, Denial> {
    budget.require_nesting_depth(2)?;
    if input.u16()? != 1 {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    let definition = ApplicationDerivedComponentSetMeaning {
        name: input.text()?.to_owned(),
        member: super::derived_relation::decode_member_domain(input)?,
        relation: input.text()?.to_owned(),
    };
    require_shape(&definition)?;
    Ok(ApplicationSchemaMember::DerivedComponentSet { definition })
}
