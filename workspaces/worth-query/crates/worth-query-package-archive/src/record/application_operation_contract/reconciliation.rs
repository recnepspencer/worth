use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_installation::facade::WorthQueryPortableInstalledReconciliationProcedureRecord;

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::WorthQueryPackageArchiveDenial as Denial;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    reconciliation: Option<&WorthQueryPortableInstalledReconciliationProcedureRecord>,
) -> Result<(), Denial> {
    super::super::foundational_value::write_bool(output, reconciliation.is_some())?;
    if let Some(reconciliation) = reconciliation {
        output.text(reconciliation.procedure_slot())?;
    }
    Ok(())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
) -> Result<Option<WorthQueryPortableInstalledReconciliationProcedureRecord>, Denial> {
    if !super::super::foundational_value::decode_bool(input)? {
        return Ok(None);
    }
    Ok(Some(
        WorthQueryPortableInstalledReconciliationProcedureRecord::from_untrusted_procedure_slot(
            ARCHIVE_DECODE,
            input.text()?.to_owned(),
        ),
    ))
}
