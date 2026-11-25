//! Tag 8 wraps a recorded inverse that grants omitted reversals: the inner
//! mechanism without grants, then the granted field loci. Older readers have
//! no tag 8 and refuse the record rather than install a partial undo as full.
use super::*;

pub(super) const TAG: u16 = 8;

type WriteMechanism =
    fn(&mut dyn BinaryEncodingSink, &PortableCorrectionMechanism) -> Result<(), Denial>;
type DecodeMechanism = fn(
    &mut BinaryInput<'_>,
    &mut RecordDecodeAttempt,
) -> Result<PortableCorrectionMechanism, Denial>;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    value: &PortableRecordedInverse,
    inner: WriteMechanism,
) -> Result<(), Denial> {
    output.u16(TAG)?;
    let demand = value.preimage_demand();
    let granted = demand.omitted_reversals();
    let without = PortableRecordedInverse::from_untrusted_fields(
        ARCHIVE_DECODE,
        value.inverse_operation_slot().to_owned(),
        value.lowering_correspondence().clone(),
        value.postcondition().clone(),
        demand
            .clone()
            .with_untrusted_omitted_reversals(ARCHIVE_DECODE, Vec::new()),
    );
    inner(
        output,
        &PortableCorrectionMechanism::RecordedInverse(without),
    )?;
    write_sequence(output, granted, |output, locus| {
        output.text(locus.entity())?;
        output.text(locus.aspect())?;
        output.text(locus.field())
    })
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    inner: DecodeMechanism,
) -> Result<PortableCorrectionMechanism, Denial> {
    // A nested tag 8 decodes with grants already present: refused.
    let value = match inner(input, budget)? {
        PortableCorrectionMechanism::RecordedInverse(value)
            if value.preimage_demand().omitted_reversals().is_empty() =>
        {
            value
        }
        _ => return Err(Denial::new(Kind::InvalidRecordShape)),
    };
    let granted = decode_sequence(input, budget, 12, |input, _| {
        Ok(PortablePreImageLocus::from_untrusted_fields(
            ARCHIVE_DECODE,
            input.text()?.to_owned(),
            input.text()?.to_owned(),
            input.text()?.to_owned(),
        ))
    })?;
    if granted.is_empty() {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    Ok(PortableCorrectionMechanism::RecordedInverse(
        PortableRecordedInverse::from_untrusted_fields(
            ARCHIVE_DECODE,
            value.inverse_operation_slot().to_owned(),
            value.lowering_correspondence().clone(),
            value.postcondition().clone(),
            value
                .preimage_demand()
                .clone()
                .with_untrusted_omitted_reversals(ARCHIVE_DECODE, granted),
        ),
    ))
}
