//! Body v17 carries created-subject enrollment meaning, never live authority.
use super::{BinaryEncodingSink, BinaryInput, Denial, Kind};
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyEnrollment as Enrollment;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    enrollment: Option<&Enrollment>,
) -> Result<(), Denial> {
    output.u8(u8::from(enrollment.is_some()))?;
    if let Some(enrollment) = enrollment {
        output.text(enrollment.workflow_spec())?;
        output.text(enrollment.binding())?;
        output.text(enrollment.role())?;
    }
    Ok(())
}

pub(super) fn decode(input: &mut BinaryInput<'_>) -> Result<Option<Enrollment>, Denial> {
    match input.u8()? {
        0 => Ok(None),
        1 => Ok(Some(Enrollment::from_untrusted(
            input.text()?.to_owned(),
            input.text()?.to_owned(),
            input.text()?.to_owned(),
        ))),
        _ => Err(Denial::new(Kind::InvalidBooleanEncoding)),
    }
}
