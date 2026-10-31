//! Query result presence tags: legacy Required1/OptionalField2 are unchanged.
use crate::{
    binary_encoding::BinaryEncodingSink,
    binary_input::BinaryInput,
    denial::{
        WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
    },
};
use worth_query_declaration::facade::application_query::ApplicationQueryResultFieldPresence as Presence;
pub(super) fn write(output: &mut dyn BinaryEncodingSink, presence: Presence) -> Result<(), Denial> {
    output.u16(match presence {
        Presence::Required => 1,
        Presence::Optional => 2,
        Presence::OptionalAspect => 3,
    })
}
pub(super) fn decode(input: &mut BinaryInput<'_>) -> Result<Presence, Denial> {
    match input.u16()? {
        1 => Ok(Presence::Required),
        2 => Ok(Presence::Optional),
        3 => Ok(Presence::OptionalAspect),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn query_parent_absence_appends_a_tag_without_reinterpreting_field_presence() {
        for (presence, tag) in [
            (Presence::Required, 1u16),
            (Presence::Optional, 2),
            (Presence::OptionalAspect, 3),
        ] {
            let mut output = crate::binary_output::BinaryOutput::with_capacity(2);
            write(&mut output, presence).unwrap();
            let bytes = output.into_bytes();
            assert_eq!(bytes, tag.to_be_bytes());
            assert_eq!(decode(&mut BinaryInput::new(&bytes)).unwrap(), presence);
        }
        assert!(decode(&mut BinaryInput::new(&[0, 4])).is_err());
    }
}
