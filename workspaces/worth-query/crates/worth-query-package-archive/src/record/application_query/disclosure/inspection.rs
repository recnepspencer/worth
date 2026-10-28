//! Selector8/v1 retains all fourteen query-binding fields, not a display name.
use super::*;
use crate::record::application_schema::query_binding_reference;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryInspectionMetadataFacet as Facet,
    ApplicationQueryInspectionMetadataMeaning as Meaning,
};

pub(super) fn write(output: &mut dyn BinaryEncodingSink, meaning: &Meaning) -> Result<(), Denial> {
    output.u16(1)?;
    query_binding_reference::write(output, meaning.binding())?;
    output.u16(match meaning.facet() {
        Facet::ObservationIdentity => 1,
        Facet::EvidenceLinks => 2,
        Facet::EvidenceCounts => 3,
        Facet::EvidenceAvailability => 4,
        Facet::Explanation => 5,
        Facet::DiagnosticFields => 6,
    })
}

pub(super) fn decode(input: &mut BinaryInput<'_>) -> Result<Meaning, Denial> {
    if input.u16()? != 1 {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    let binding = query_binding_reference::decode(input)?;
    let facet = match input.u16()? {
        1 => Facet::ObservationIdentity,
        2 => Facet::EvidenceLinks,
        3 => Facet::EvidenceCounts,
        4 => Facet::EvidenceAvailability,
        5 => Facet::Explanation,
        6 => Facet::DiagnosticFields,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(Meaning::from_untrusted_fields(binding, facet))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_inspection_selector_version_refuses_before_binding_fields() {
        let mut input = BinaryInput::new(&[0, 2]);
        assert_eq!(
            decode(&mut input).unwrap_err().kind(),
            Kind::UnsupportedRecordVariant
        );
    }
}
