use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;
use worth_query_installation::facade::{
    WorthQueryPortableApplicationOperationContractParts,
    WorthQueryPortableApplicationOperationContractRecord, WorthQueryPortablePackageRecord,
};

use crate::binary_encoding::{BinaryEncodingMeasure, BinaryEncodingSink};
use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::denial::WorthQueryPackageArchiveDenial as Denial;
use crate::limits::WorthQueryPackageArchiveLimits;

use super::decode_budget::RecordDecodeAttempt;
use super::encoding_budget::RecordPayloadEncodingWork;
use super::sequence::{
    decode_sequence, require_canonical_sequence, require_canonical_sequence_by_order,
    write_sequence,
};

mod external_effect;
mod graph_read;
mod reconciliation;
mod touch;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(super) fn payload_byte_length(
    record: &WorthQueryPortableApplicationOperationContractRecord,
    limits: WorthQueryPackageArchiveLimits,
) -> Result<u64, Denial> {
    Ok(payload_encoding_work(record, limits)?.payload_bytes())
}

pub(super) fn payload_encoding_work(
    record: &WorthQueryPortableApplicationOperationContractRecord,
    limits: WorthQueryPackageArchiveLimits,
) -> Result<RecordPayloadEncodingWork, Denial> {
    let limits = limits.narrowed();
    if let Some(body) = record.body() {
        super::operation_body::require_nesting_depth(body, limits.maximum_nesting_depth())?;
    }
    let mut measure = BinaryEncodingMeasure::default();
    write_record(&mut measure, record)?;
    RecordPayloadEncodingWork::from_measure(&measure, limits)
}

pub(super) fn write_payload(
    record: &WorthQueryPortableApplicationOperationContractRecord,
    output: &mut BinaryOutput,
) -> Result<(), Denial> {
    write_record(output, record)
}

pub(super) fn decode_payload(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryPortablePackageRecord, Denial> {
    let schema = input.text()?.to_owned();
    let operation = input.text()?.to_owned();
    let input_type = WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.to_owned());
    let graph_reads = decode_sequence(input, budget, 2, graph_read::decode)?;
    require_canonical_sequence_by_order(&graph_reads, graph_read::canonical_order)?;
    let touches = decode_sequence(input, budget, 2, touch::decode)?;
    require_canonical_sequence_by_order(&touches, touch::canonical_order)?;
    let emissions = decode_sequence(input, budget, 4, |input, _| Ok(input.text()?.to_owned()))?;
    require_canonical_sequence(&emissions)?;
    let external_effect = external_effect::decode(input)?;
    let reconciliation = reconciliation::decode(input)?;
    let body = if input.is_finished() {
        None
    } else {
        Some(super::operation_body::decode(input, budget)?)
    };
    Ok(
        WorthQueryPortablePackageRecord::ApplicationOperationContract(
            WorthQueryPortableApplicationOperationContractRecord::from_untrusted_parts(
                WorthQueryPortableApplicationOperationContractParts {
                    schema,
                    operation,
                    input_type,
                    graph_reads,
                    touches,
                    emissions,
                    external_effect,
                    reconciliation,
                    body,
                },
            ),
        ),
    )
}

fn write_record(
    output: &mut dyn BinaryEncodingSink,
    record: &WorthQueryPortableApplicationOperationContractRecord,
) -> Result<(), Denial> {
    output.text(record.schema())?;
    output.text(record.operation())?;
    output.text(record.input_type().as_str())?;
    write_sequence(output, record.graph_reads(), graph_read::write)?;
    write_sequence(output, record.touches(), touch::write)?;
    write_sequence(output, record.emissions(), |output, emission| {
        output.text(emission)
    })?;
    external_effect::write(output, record.external_effect())?;
    reconciliation::write(output, record.reconciliation())?;
    if let Some(body) = record.body() {
        super::operation_body::write(output, body)?;
    }
    Ok(())
}
