//! Committed-index selector payload; version is checked before field allocation.
use super::*;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryManagedIndexMeaning as Meaning,
    WorthQueryPortableQueryManagedIndexParts as Parts,
};

pub(super) fn write(output: &mut dyn BinaryEncodingSink, meaning: &Meaning) -> Result<(), Denial> {
    meaning
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let parts = meaning.parts();
    output.u16(parts.semantic_version)?;
    output.text(&parts.index)?;
    output.text(&parts.probe_parameter)?;
    if parts.semantic_version == 2 {
        crate::record::managed_index_refinement::write(
            output,
            meaning
                .refinement()
                .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?,
        )?;
    }
    Ok(())
}
pub(super) fn decode(input: &mut BinaryInput<'_>) -> Result<Meaning, Denial> {
    let semantic_version = input.u16()?;
    if !matches!(semantic_version, 1 | 2) {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    let parts = Parts {
        semantic_version,
        index: input.text()?.to_owned(),
        probe_parameter: input.text()?.to_owned(),
    };
    let meaning = if semantic_version == 2 {
        Meaning::from_refinement_parts(
            parts,
            crate::record::managed_index_refinement::decode(input)?,
        )
    } else {
        Meaning::from_untrusted_parts(parts)
    };
    meaning
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    Ok(meaning)
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_query_declaration::facade::application_query::{
        ApplicationManagedIndexMatchPosture, ApplicationManagedIndexRefinement,
    };
    #[test]
    fn refined_selector_round_trip_keeps_explicit_mode_and_domain_contract() {
        let old = Meaning::from_untrusted_parts(Parts {
            semantic_version: 1,
            index: "Prefix".into(),
            probe_parameter: "probe".into(),
        });
        let mut parts = old.parts().clone();
        parts.semantic_version = 2;
        assert!(Meaning::from_untrusted_parts(parts.clone())
            .validate()
            .is_err());
        let refined = Meaning::from_refinement_parts(
            parts,
            ApplicationManagedIndexRefinement::row_local("domain.exact-name.v1", 7).meaning(),
        );
        let mut encodings = Vec::new();
        for meaning in [old, refined] {
            let mut output = crate::binary_output::BinaryOutput::with_capacity(128);
            write(&mut output, &meaning).unwrap();
            let bytes = output.into_bytes();
            let mut input = BinaryInput::new(&bytes);
            let restored = decode(&mut input).unwrap();
            assert!(input.is_finished());
            assert_eq!(restored, meaning);
            assert_eq!(
                restored.posture() == ApplicationManagedIndexMatchPosture::Refined,
                restored.refinement().is_some()
            );
            encodings.push(bytes);
        }
        assert_ne!(encodings[0], encodings[1]);
        assert_eq!(
            decode(&mut BinaryInput::new(&[0, 3])).unwrap_err().kind(),
            Kind::UnsupportedRecordVariant
        );
        assert_eq!(
            decode(&mut BinaryInput::new(&[0, 2])).unwrap_err().kind(),
            Kind::Truncated
        );
    }
}
