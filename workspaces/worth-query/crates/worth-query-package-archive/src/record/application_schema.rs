use worth_query_declaration::facade::application_schema::{
    WorthQueryPortableApplicationSchemaParts, WorthQueryPortableApplicationSchemaRecord,
};
use worth_query_installation::facade::WorthQueryPortablePackageRecord;

use crate::binary_encoding::{BinaryEncodingMeasure, BinaryEncodingSink};
use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::denial::WorthQueryPackageArchiveDenial as Denial;
use crate::denial::WorthQueryPackageArchiveDenialKind as Kind;
use crate::limits::WorthQueryPackageArchiveLimits;

use super::decode_budget::RecordDecodeAttempt;
use super::encoding_budget::RecordPayloadEncodingWork;
use super::sequence::{decode_sequence, write_sequence};

mod authorization_path;
mod contribution;
mod member;
mod wire_vocabulary;

const CONTRIBUTION_EXTENSION_MARKER: &[u8; 4] = b"WQAS";
const CONTRIBUTION_EXTENSION_VERSION: u16 = 1;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(super) fn payload_byte_length(
    record: &WorthQueryPortableApplicationSchemaRecord,
    limits: WorthQueryPackageArchiveLimits,
) -> Result<u64, Denial> {
    Ok(payload_encoding_work(record, limits)?.payload_bytes())
}

pub(super) fn payload_encoding_work(
    record: &WorthQueryPortableApplicationSchemaRecord,
    limits: WorthQueryPackageArchiveLimits,
) -> Result<RecordPayloadEncodingWork, Denial> {
    let limits = limits.narrowed();
    require_nesting_depth(record, limits.maximum_nesting_depth())?;
    let mut measure = BinaryEncodingMeasure::default();
    write_record(&mut measure, record)?;
    RecordPayloadEncodingWork::from_measure(&measure, limits)
}

pub(super) fn write_payload(
    record: &WorthQueryPortableApplicationSchemaRecord,
    output: &mut BinaryOutput,
    limits: WorthQueryPackageArchiveLimits,
) -> Result<(), Denial> {
    require_nesting_depth(record, limits.narrowed().maximum_nesting_depth())?;
    write_record(output, record)
}

pub(super) fn decode_payload(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryPortablePackageRecord, Denial> {
    let owner = input.text()?.to_owned();
    let name = input.text()?.to_owned();
    let major = input.u32()?;
    let minor = input.u32()?;
    let members = decode_sequence(input, budget, 2, member::decode)?;
    let contributions = if input.remaining_starts_with(CONTRIBUTION_EXTENSION_MARKER) {
        input.take(CONTRIBUTION_EXTENSION_MARKER.len())?;
        if input.u16()? != CONTRIBUTION_EXTENSION_VERSION {
            return Err(Denial::new(Kind::UnsupportedRecordVersion));
        }
        decode_sequence(input, budget, 8, contribution::decode)?
    } else {
        Vec::new()
    };
    Ok(WorthQueryPortablePackageRecord::ApplicationSchema(
        WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
            WorthQueryPortableApplicationSchemaParts {
                owner,
                name,
                major,
                minor,
                members,
                contributions,
            },
        ),
    ))
}

fn write_record(
    output: &mut dyn BinaryEncodingSink,
    record: &WorthQueryPortableApplicationSchemaRecord,
) -> Result<(), Denial> {
    output.text(record.owner())?;
    output.text(record.name())?;
    output.u32(record.major())?;
    output.u32(record.minor())?;
    write_sequence(output, record.members(), member::write)?;
    if record.contributions().is_empty() {
        Ok(())
    } else {
        output.raw_bytes(CONTRIBUTION_EXTENSION_MARKER)?;
        output.u16(CONTRIBUTION_EXTENSION_VERSION)?;
        write_sequence(output, record.contributions(), contribution::write)
    }
}

fn require_nesting_depth(
    record: &WorthQueryPortableApplicationSchemaRecord,
    maximum_depth: u32,
) -> Result<(), Denial> {
    for schema_member in record.members() {
        member::require_nesting_depth(schema_member, maximum_depth)?;
    }
    Ok(())
}
