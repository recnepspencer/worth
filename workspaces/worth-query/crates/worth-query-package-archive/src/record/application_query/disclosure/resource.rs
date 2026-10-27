use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::application_schema::wire_vocabulary::{
    decode_type_identity, write_type_identity,
};
use crate::{binary_encoding::BinaryEncodingSink, binary_input::BinaryInput};
use worth_query_declaration::facade::application_resource::{
    ApplicationResourceDisclosureFacet as Facet, ApplicationResourceDisclosureMeaning,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    meaning: &ApplicationResourceDisclosureMeaning,
) -> Result<(), Denial> {
    write_type_identity(output, &meaning.resource_identity())?;
    output.u16(match meaning.facet() {
        Facet::Availability => 1,
        Facet::State => 2,
        Facet::Result => 3,
        Facet::DomainFailure => 4,
        Facet::CleanupDetail => 5,
    })
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationResourceDisclosureMeaning, Denial> {
    let resource = decode_type_identity(input)?;
    let facet = match input.u16()? {
        1 => Facet::Availability,
        2 => Facet::State,
        3 => Facet::Result,
        4 => Facet::DomainFailure,
        5 => Facet::CleanupDetail,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(ApplicationResourceDisclosureMeaning::from_untrusted_fields(
        resource, facet,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary_output::BinaryOutput;
    use crate::record::decode_budget::RecordDecodeAttempt;
    use worth_query_declaration::facade::{
        application_query::ApplicationQueryDisclosureSelector,
        portable_identity::WorthQueryPortableTypeIdentity,
    };

    #[test]
    fn resource_selector_uses_reserved_tag_and_refuses_unknown_facets() {
        let meaning = ApplicationResourceDisclosureMeaning::from_untrusted_fields(
            WorthQueryPortableTypeIdentity::declared("worth.archive.assessment.v1"),
            Facet::Result,
        );
        let selector = ApplicationQueryDisclosureSelector::Resource(meaning);
        let mut output = BinaryOutput::with_capacity(128);
        super::super::write_selector(&mut output, &selector).unwrap();
        let mut bytes = output.into_bytes();
        assert_eq!(&bytes[..2], &7u16.to_be_bytes());
        let mut input = BinaryInput::new(&bytes);
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            crate::limits::WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        assert_eq!(
            super::super::decode_selector(&mut input, &mut budget).unwrap(),
            selector
        );
        assert!(input.is_finished());
        let end = bytes.len();
        bytes[end - 2..].copy_from_slice(&u16::MAX.to_be_bytes());
        let mut input = BinaryInput::new(&bytes);
        assert_eq!(
            super::super::decode_selector(&mut input, &mut budget)
                .unwrap_err()
                .kind(),
            Kind::UnsupportedRecordVariant
        );
    }
}
