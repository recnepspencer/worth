use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;
use worth_query_installation::facade::{
    WorthQueryPortableApplicationConditionalOperationBinding,
    WorthQueryPortableApplicationConditionalOperationBindingParts, WorthQueryPortablePackageRecord,
};

use crate::binary_encoding::{BinaryEncodingMeasure, BinaryEncodingSink};
use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::denial::WorthQueryPackageArchiveDenial as Denial;
use crate::limits::WorthQueryPackageArchiveLimits;

use super::encoding_budget::RecordPayloadEncodingWork;

pub(super) fn payload_encoding_work(
    binding: &WorthQueryPortableApplicationConditionalOperationBinding,
    limits: WorthQueryPackageArchiveLimits,
) -> Result<RecordPayloadEncodingWork, Denial> {
    let limits = limits.narrowed();
    let mut measure = BinaryEncodingMeasure::default();
    write_binding(&mut measure, binding)?;
    RecordPayloadEncodingWork::from_measure(&measure, limits)
}

pub(super) fn write_payload(
    binding: &WorthQueryPortableApplicationConditionalOperationBinding,
    output: &mut BinaryOutput,
) -> Result<(), Denial> {
    write_binding(output, binding)
}

pub(super) fn decode_payload(
    input: &mut BinaryInput<'_>,
) -> Result<WorthQueryPortablePackageRecord, Denial> {
    let binding = WorthQueryPortableApplicationConditionalOperationBinding::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationConditionalOperationBindingParts {
            schema_owner: input.text()?.to_owned(),
            schema_name: input.text()?.to_owned(),
            application_operation: input.text()?.to_owned(),
            input_type: WorthQueryPortableTypeIdentity::from_untrusted(
                ARCHIVE_DECODE,
                input.text()?.to_owned(),
            ),
            domain_operation_slot: input.text()?.to_owned(),
            domain_operation_canonical_identity: input.text()?.to_owned(),
        },
    );
    Ok(WorthQueryPortablePackageRecord::ConditionalApplicationOperation(binding))
}

fn write_binding(
    output: &mut dyn BinaryEncodingSink,
    binding: &WorthQueryPortableApplicationConditionalOperationBinding,
) -> Result<(), Denial> {
    output.text(binding.schema_owner())?;
    output.text(binding.schema_name())?;
    output.text(binding.application_operation())?;
    output.text(binding.input_type())?;
    output.text(binding.domain_operation_slot())?;
    output.text(binding.domain_operation_canonical_identity())
}
