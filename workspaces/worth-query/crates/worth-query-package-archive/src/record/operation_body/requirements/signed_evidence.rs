//! V16 retains the exact completed signed producer reference, not live evidence authority.
use super::{ApplicationFieldBindingLocus, BinaryEncodingSink, BinaryInput, Denial, Kind};
use worth_query_declaration::facade::application_schema::ApplicationSigningEvidenceSource;
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    source: &ApplicationSigningEvidenceSource,
) -> Result<(), Denial> {
    for value in [
        &source.mutation_binding,
        &source.operation,
        &source.evidence_entity,
        source.provenance.entity(),
        source.provenance.aspect(),
        source.provenance.field(),
        &source.purpose,
        &source.producer_meaning,
    ] {
        output.text(value)?;
    }
    super::super::signing_lifecycle::write(output, Some(&source.lifecycle))
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationSigningEvidenceSource, Denial> {
    Ok(ApplicationSigningEvidenceSource {
        mutation_binding: input.text()?.into(),
        operation: input.text()?.into(),
        evidence_entity: input.text()?.into(),
        provenance: ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?),
        purpose: input.text()?.into(),
        producer_meaning: input.text()?.into(),
        lifecycle: super::super::signing_lifecycle::decode(input)?
            .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?,
    })
}
