//! Portable original-source disclosure meaning, independent of correction.
use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::application_schema::wire_vocabulary::{
    decode_type_identity, write_type_identity,
};
use worth_query_declaration::facade::application_operation::{
    DeclaredApplicationSourceDisclosure, DeclaredApplicationSourceDisclosureParts,
};

use super::super::query_binding_reference;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    disclosure: Option<&DeclaredApplicationSourceDisclosure>,
) -> Result<(), Denial> {
    let Some(disclosure) = disclosure else {
        return output.u16(0);
    };
    output.u16(1)?;
    write_type_identity(output, &disclosure.parts().identity)?;
    write_type_identity(output, &disclosure.parts().source_binding)?;
    query_binding_reference::write(output, &disclosure.parts().query)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
) -> Result<Option<DeclaredApplicationSourceDisclosure>, Denial> {
    match input.u16()? {
        0 => Ok(None),
        1 => Ok(Some(
            DeclaredApplicationSourceDisclosure::from_untrusted_parts(
                DeclaredApplicationSourceDisclosureParts {
                    identity: decode_type_identity(input)?,
                    source_binding: decode_type_identity(input)?,
                    query: query_binding_reference::decode(input)?,
                },
            ),
        )),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
