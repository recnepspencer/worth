use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_installation::facade::{
    WorthQueryArtifactContractReference, WorthQueryArtifactFamilyIdentity,
    WorthQueryArtifactProtocolVersion, WorthQueryArtifactSchemaVersion,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::WorthQueryPackageArchiveDenial as Denial;

pub(super) fn write_reference(
    output: &mut dyn BinaryEncodingSink,
    reference: &WorthQueryArtifactContractReference,
) -> Result<(), Denial> {
    output.text(reference.family().as_str())?;
    output.u32(reference.schema_version().get())?;
    output.u32(reference.protocol_version().get())
}

pub(super) fn decode_reference(
    input: &mut BinaryInput<'_>,
) -> Result<WorthQueryArtifactContractReference, Denial> {
    Ok(WorthQueryArtifactContractReference::from_untrusted_fields(
        ARCHIVE_DECODE,
        WorthQueryArtifactFamilyIdentity::from_untrusted_string(
            ARCHIVE_DECODE,
            input.text()?.to_owned(),
        ),
        WorthQueryArtifactSchemaVersion::new(input.u32()?),
        WorthQueryArtifactProtocolVersion::new(input.u32()?),
    ))
}
