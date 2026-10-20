//! Member tag 33 extends the indexed resource head without changing tag 7 bytes.
use super::super::super::super::super::decode_budget::RecordDecodeAttempt;
use super::super::super::super::super::sequence::{decode_sequence, write_sequence};
use super::super::super::super::authorization_path;
use super::bindings;
use crate::{
    binary_encoding::BinaryEncodingSink,
    binary_input::BinaryInput,
    denial::{
        WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
    },
};
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityRelationBinding, ApplicationCapabilityResourcePath,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    path: &ApplicationCapabilityResourcePath,
) -> Result<(), Denial> {
    let root = path
        .root_slot()
        .ok_or_else(|| Denial::new(Kind::UnsupportedRecordVariant))?;
    if path.traversals().is_empty() {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    bindings::write_context_slot(output, root)?;
    write_sequence(
        output,
        path.traversals(),
        authorization_path::write_traversal,
    )
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    head: ApplicationCapabilityRelationBinding,
) -> Result<ApplicationCapabilityResourcePath, Denial> {
    let root = bindings::decode_context_slot(input)?;
    let traversals = decode_sequence(input, budget, 14, |input, _| {
        authorization_path::decode_traversal(input)
    })?;
    if traversals.is_empty() {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    Ok(ApplicationCapabilityResourcePath::from_untrusted_parts(
        head,
        Some(root),
        traversals,
    ))
}
