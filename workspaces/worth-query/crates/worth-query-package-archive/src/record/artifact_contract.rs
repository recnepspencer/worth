use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_installation::facade::{
    WorthQueryPortableArtifactContractParts, WorthQueryPortableArtifactContractRecord,
    WorthQueryPortablePackageRecord,
};

use crate::binary_encoding::{BinaryEncodingMeasure, BinaryEncodingSink};
use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::denial::WorthQueryPackageArchiveDenial as Denial;
use crate::limits::WorthQueryPackageArchiveLimits;

use super::decode_budget::RecordDecodeAttempt;
use super::encoding_budget::RecordPayloadEncodingWork;
use super::sequence::{decode_sequence, require_canonical_sequence, write_sequence};

mod access_path;
mod carriage;
mod counters;
mod decisions;
mod governance;
mod identity;
mod occurrence;
mod search;
mod transformation;

pub(super) fn payload_encoding_work(
    record: &WorthQueryPortableArtifactContractRecord,
    limits: WorthQueryPackageArchiveLimits,
) -> Result<RecordPayloadEncodingWork, Denial> {
    let limits = limits.narrowed();
    let mut measure = BinaryEncodingMeasure::default();
    write_record(&mut measure, record.parts())?;
    RecordPayloadEncodingWork::from_measure(&measure, limits)
}

pub(super) fn write_payload(
    record: &WorthQueryPortableArtifactContractRecord,
    output: &mut BinaryOutput,
) -> Result<(), Denial> {
    write_record(output, record.parts())
}

pub(super) fn decode_payload(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryPortablePackageRecord, Denial> {
    let (family, schema_version, protocol_version) = identity::decode_family(input)?;
    let content_identity = identity::decode_content_identity(input)?;
    let ownership = identity::decode_ownership(input)?;
    let occurrence = occurrence::decode_occurrence(input, budget)?;
    let evidence = identity::decode_evidence(input)?;
    let reproducibility = occurrence::decode_reproducibility(input, budget)?;
    let search = search::decode_search(input)?;
    let convergence = search::decode_convergence(input)?;
    let transformation = transformation::decode_transformation(input)?;
    let access_path = access_path::decode_access_path(input, budget)?;
    let carriage = carriage::decode_carriage(input)?;
    let lifecycle = carriage::decode_lifecycle(input)?;
    let counters = counters::decode_counters(input, budget)?;
    let decisions = decisions::decode_decisions(input, budget)?;
    let governance = governance::decode_governance(input, budget)?;
    let compatibility = governance::decode_compatibility(input, budget)?;
    let producer_roles = decode_roles(input, budget)?;
    let consumer_roles = decode_roles(input, budget)?;
    Ok(WorthQueryPortablePackageRecord::ArtifactContract(
        WorthQueryPortableArtifactContractRecord::from_untrusted_parts(
            ARCHIVE_DECODE,
            WorthQueryPortableArtifactContractParts {
                family,
                schema_version,
                protocol_version,
                content_identity,
                ownership,
                occurrence,
                evidence,
                reproducibility,
                search,
                convergence,
                transformation,
                access_path,
                carriage,
                lifecycle,
                counters,
                decisions,
                governance,
                compatibility,
                producer_roles,
                consumer_roles,
            },
        ),
    ))
}

fn write_record(
    output: &mut dyn BinaryEncodingSink,
    parts: &WorthQueryPortableArtifactContractParts,
) -> Result<(), Denial> {
    identity::write_family(
        output,
        &parts.family,
        parts.schema_version,
        parts.protocol_version,
    )?;
    identity::write_content_identity(output, &parts.content_identity)?;
    identity::write_ownership(output, &parts.ownership)?;
    occurrence::write_occurrence(output, &parts.occurrence)?;
    identity::write_evidence(output, &parts.evidence)?;
    occurrence::write_reproducibility(output, &parts.reproducibility)?;
    search::write_search(output, &parts.search)?;
    search::write_convergence(output, &parts.convergence)?;
    transformation::write_transformation(output, &parts.transformation)?;
    access_path::write_access_path(output, &parts.access_path)?;
    carriage::write_carriage(output, parts.carriage)?;
    carriage::write_lifecycle(output, parts.lifecycle)?;
    counters::write_counters(output, &parts.counters)?;
    decisions::write_decisions(output, &parts.decisions)?;
    governance::write_governance(output, &parts.governance)?;
    governance::write_compatibility(output, &parts.compatibility)?;
    write_roles(output, &parts.producer_roles)?;
    write_roles(output, &parts.consumer_roles)
}

fn write_roles(output: &mut dyn BinaryEncodingSink, roles: &[String]) -> Result<(), Denial> {
    write_sequence(output, roles, |output, role| output.text(role))
}

fn decode_roles(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Vec<String>, Denial> {
    let roles = decode_sequence(input, budget, 4, |input, _| Ok(input.text()?.to_owned()))?;
    require_canonical_sequence(&roles)?;
    Ok(roles)
}
