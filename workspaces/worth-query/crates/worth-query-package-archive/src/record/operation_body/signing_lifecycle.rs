//! Body v15 carries the bounded native supersession declaration, never authority.
use super::{ApplicationFieldBindingLocus as Locus, BinaryEncodingSink, BinaryInput, Denial, Kind};
use worth_query_declaration::facade::application_schema::ApplicationOperationBodySigningLifecycleSpec as Spec;
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    spec: Option<&Spec>,
) -> Result<(), Denial> {
    output.u8(u8::from(spec.is_some()))?;
    if let Some(spec) = spec {
        output.text(&spec.history_relation)?;
        output.text(&spec.current_relation)?;
        output.text(&spec.source_entity)?;
        output.text(&spec.consequence_relation)?;
        output.text(spec.consequence_field.entity())?;
        output.text(spec.consequence_field.aspect())?;
        output.text(spec.consequence_field.field())?;
        output.u64(
            u64::try_from(spec.maximum_evidence)
                .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
        )?;
    }
    Ok(())
}
pub(super) fn decode(input: &mut BinaryInput<'_>) -> Result<Option<Spec>, Denial> {
    match input.u8()? {
        0 => Ok(None),
        1 => Ok(Some(Spec {
            history_relation: input.text()?.to_owned(),
            current_relation: input.text()?.to_owned(),
            source_entity: input.text()?.to_owned(),
            consequence_relation: input.text()?.to_owned(),
            consequence_field: Locus::new(input.text()?, input.text()?, input.text()?),
            maximum_evidence: usize::try_from(input.u64()?)
                .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
        })),
        _ => Err(Denial::new(Kind::InvalidBooleanEncoding)),
    }
}
