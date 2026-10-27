//! Tag44 owns a v1 subgrammar; existing record-frame/member bytes are unchanged.
use super::*;
use crate::record::sequence::{decode_sequence, write_sequence};
use worth_query_declaration::facade::application_schema::{
    ErasedApplicationProjectedMemberDomainDefinition as Definition,
    WorthQueryPortableProjectedMemberDomainParts as Parts,
};

pub(super) fn require_shape(member: &ApplicationSchemaMember) -> Result<(), Denial> {
    if let ApplicationSchemaMember::ProjectedMemberDomain { definition } = member {
        definition
            .validate()
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    }
    Ok(())
}

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    definition: &Definition,
) -> Result<(), Denial> {
    let parts = definition.parts();
    output.u16(parts.semantic_version)?;
    output.text(&parts.domain)?;
    output.text(&parts.carrier)?;
    write_sequence(output, &parts.cases, |output, case| output.text(case))
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaMember, Denial> {
    let semantic_version = input.u16()?;
    // A future subgrammar may have a different payload. Reject before reading
    // text, admitting a sequence or allocating its fields.
    if semantic_version != Definition::CURRENT_VERSION {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    budget.require_nesting_depth(2)?;
    let domain = input.text()?.to_owned();
    let carrier = input.text()?.to_owned();
    let cases = decode_sequence(input, budget, 4, |input, _| Ok(input.text()?.to_owned()))?;
    let member = ApplicationSchemaMember::ProjectedMemberDomain {
        definition: Definition::from_untrusted_parts(Parts {
            semantic_version,
            domain,
            carrier,
            cases,
        }),
    };
    require_shape(&member)?;
    Ok(member)
}
