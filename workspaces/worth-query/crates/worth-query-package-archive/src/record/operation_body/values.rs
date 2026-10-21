//! V28 carries named UInt64 expressions once; sink records reference their names.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyValueProgram;
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    values: &[ApplicationOperationBodyValueProgram],
) -> Result<(), Denial> {
    write_sequence(output, values, |output, value| {
        output.text(value.identity())?;
        output.text(value.binding().as_str())?;
        requirements::write_operands(output, value.operands())?;
        requirements::write_draft(output, value.expression().draft())
    })
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Vec<ApplicationOperationBodyValueProgram>, Denial> {
    decode_sequence(input, budget, 16, |input, budget| {
        let identity = input.text()?.to_owned();
        let binding = WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.to_owned());
        let operands = requirements::decode_operands(input, budget, 28)?;
        let draft = requirements::draft(input, budget)?;
        ApplicationOperationBodyValueProgram::from_untrusted(identity, binding, operands, draft)
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))
    })
}
