//! An optional scoped extension preserves ordinary member and inverse wire bytes.
use super::super::wire_vocabulary::{decode_type_identity, write_type_identity};
use crate::record::{
    decode_budget::RecordDecodeAttempt,
    sequence::{decode_sequence, write_sequence},
};
use crate::{
    binary_encoding::BinaryEncodingSink,
    binary_input::BinaryInput,
    denial::{
        WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
    },
};
use worth_query_declaration::facade::application_program::DeclaredCoupledOutputConnection;

const MARKER: &[u8; 4] = b"WQCO";
const VERSION: u16 = 1;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    connections: &[DeclaredCoupledOutputConnection],
) -> Result<(), Denial> {
    if connections.is_empty() {
        return Ok(());
    }
    output.raw_bytes(MARKER)?;
    output.u16(VERSION)?;
    write_sequence(output, connections, |output, connection| {
        output.text(&connection.connection)?;
        output.text(&connection.family)?;
        output.text(&connection.source_binding)?;
        write_type_identity(output, &connection.source_query)?;
        write_type_identity(output, &connection.source_input)
    })
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Vec<DeclaredCoupledOutputConnection>, Denial> {
    if !input.remaining_starts_with(MARKER) {
        return Ok(Vec::new());
    }
    input.take(MARKER.len())?;
    if input.u16()? != VERSION {
        return Err(Denial::new(Kind::UnsupportedRecordVersion));
    }
    decode_sequence(input, budget, 5, |input, _| {
        Ok(DeclaredCoupledOutputConnection {
            connection: input.text()?.to_owned(),
            family: input.text()?.to_owned(),
            source_binding: input.text()?.to_owned(),
            source_query: decode_type_identity(input)?,
            source_input: decode_type_identity(input)?,
        })
    })
}
