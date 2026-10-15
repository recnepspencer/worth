//! Codec roundtrip is descriptive only; supplier tests prove fresh native producer affinity.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyRequirementExpression as Expression,
    ApplicationOperationBodyRequirementOperand as Operand,
    ApplicationOperationBodyRequirementProgram as Requirement,
    ApplicationOperationBodyRequirementSource as Source,
    ApplicationOperationBodySigningLifecycleSpec as Lifecycle,
    ApplicationSigningEvidenceSource as Evidence,
};
#[test]
fn current_signed_evidence_v16_preserves_the_producer_and_v15_cannot_accept_it() {
    let evidence = Evidence {
        mutation_binding: "SignBinding".into(),
        operation: "Sign".into(),
        evidence_entity: "Evidence".into(),
        provenance: ApplicationFieldBindingLocus::new("Evidence", "Facts", "Provenance"),
        purpose: "verify".into(),
        producer_meaning: "a".repeat(64),
        lifecycle: Lifecycle {
            history_relation: "History".into(),
            current_relation: "Current".into(),
            source_entity: "Item".into(),
            consequence_relation: "Supersessions".into(),
            consequence_field: ApplicationFieldBindingLocus::new("Supersession", "Facts", "Record"),
            maximum_evidence: 32,
        },
    };
    let reader =
        worth_foundational::expression_api::expressions().reading_within(Expression::PROFILE);
    let requirement = Requirement::from_untrusted(
        "verified".into(),
        vec![Operand::from_untrusted(
            "verified".into(),
            Source::SignedEvidence(evidence),
        )],
        None,
        reader.parse("verified").unwrap().encode(),
        "verified".into(),
    )
    .unwrap();
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Item".into(),
        vec![],
        None,
        vec![requirement],
    );
    let mut output = BinaryOutput::with_capacity(1024);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &16_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(decode(&mut input, &mut budget).unwrap(), body);
    assert!(input.is_finished());
    let mut measure = crate::binary_encoding::BinaryEncodingMeasure::default();
    write(&mut measure, &body).unwrap();
    assert_eq!(measure.nested_entries(), budget.finish().nested_entries());
    bytes[..2].copy_from_slice(&15_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    assert!(decode(&mut BinaryInput::new(&bytes), &mut budget).is_err());
}
