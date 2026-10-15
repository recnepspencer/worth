//! Version-thirteen signing declarations, never live authentication proof.
use super::{ApplicationFieldBindingLocus, BinaryEncodingSink, BinaryInput, Denial, Kind};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyCreatedField as Field, ApplicationOperationBodyProgram,
    ApplicationOperationBodyRequirementSource as Source,
    ApplicationOperationBodySigningSpec as Spec,
};

pub(super) fn uses_signed_sources(body: &ApplicationOperationBodyProgram) -> bool {
    body.creation().is_some_and(|creation| {
        creation.signing().is_some()
            || creation.fields().iter().any(|field| {
                matches!(
                    field,
                    Field::PresentScopeField { .. } | Field::SigningProvenance { .. }
                )
            })
    }) || body
        .requirements()
        .iter()
        .flat_map(|requirement| requirement.operands())
        .any(|operand| matches!(operand.source(), Source::PrincipalIdentity { .. }))
}

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    spec: Option<&Spec>,
) -> Result<(), Denial> {
    output.u8(u8::from(spec.is_some()))?;
    if let Some(spec) = spec {
        output.text(spec.purpose())?;
        for field in [spec.actor(), spec.provenance()] {
            output.text(field.entity())?;
            output.text(field.aspect())?;
            output.text(field.field())?;
        }
    }
    Ok(())
}

pub(super) fn decode(input: &mut BinaryInput<'_>) -> Result<Option<Spec>, Denial> {
    match input.u8()? {
        0 => Ok(None),
        1 => {
            let purpose = input.text()?.to_owned();
            let actor =
                ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?);
            let provenance =
                ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?);
            Ok(Some(Spec::from_untrusted(purpose, actor, provenance)))
        }
        _ => Err(Denial::new(Kind::InvalidBooleanEncoding)),
    }
}
