//! Signing and present-optional source meaning require v13, never a live proof.
use super::{
    decode, write, ApplicationFieldBindingLocus as Locus,
    ApplicationOperationBodyCreatedField as Field, ApplicationOperationBodyCreation,
    ApplicationOperationBodyProgram, BinaryInput, BinaryOutput, Kind, RecordDecodeAttempt,
    ScalarAspectType, WorthQueryPackageArchiveLimits, WorthQueryPortableTypeIdentity,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyRequirementExpression as Expression,
    ApplicationOperationBodyRequirementOperand as Operand,
    ApplicationOperationBodyRequirementProgram as Requirement,
    ApplicationOperationBodyRequirementSource as Source, ApplicationOperationBodySigningSpec,
    ApplicationScalarValueBinding, ApplicationSigningProvenanceBinding,
};

#[test]
fn signed_creation_roundtrips_and_v12_refuses_its_new_source_opcodes() {
    let actor = Locus::new("Evidence", "Facts", "Actor");
    let provenance = Locus::new("Evidence", "Facts", "Provenance");
    let reader =
        worth_foundational::expression_api::expressions().reading_within(Expression::PROFILE);
    let assigned = Requirement::from_untrusted(
        "assigned".into(),
        vec![Operand::from_untrusted(
            "signer".into(),
            Source::PrincipalIdentity {
                scope: "Draft".into(),
                mutation_binding: "SignBinding".into(),
                principal_binding: "PrincipalBinding".into(),
                field: Locus::new("Principal", "Identity", "Id"),
                binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.u64".into()),
                scalar_family: ScalarAspectType::UInt64,
            },
        )],
        None,
        reader
            .parse("signer == exact_cast<UInt64>(1)")
            .unwrap()
            .encode(),
        "signer".into(),
    )
    .unwrap();
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Draft".into(),
        vec![],
        Some(
            ApplicationOperationBodyCreation::from_untrusted(
                "Evidence".into(),
                "evidence".into(),
                vec![
                    Field::PresentScopeField {
                        field: Locus::new("Evidence", "Facts", "Text"),
                        binding: WorthQueryPortableTypeIdentity::from_untrusted(
                            "worth.rust.string".into(),
                        ),
                        scalar_family: ScalarAspectType::String,
                        source: Locus::new("Draft", "Facts", "Description"),
                    },
                    Field::PrincipalIdentity {
                        field: actor.clone(),
                        binding: WorthQueryPortableTypeIdentity::from_untrusted(
                            "worth.rust.u64".into(),
                        ),
                        scalar_family: ScalarAspectType::UInt64,
                        mutation_binding: "SignBinding".into(),
                        principal_binding: "PrincipalBinding".into(),
                        source: Locus::new("Principal", "Identity", "Id"),
                    },
                    Field::SigningProvenance {
                        field: provenance.clone(),
                        binding: ApplicationSigningProvenanceBinding::IDENTITY,
                    },
                ],
            )
            .with_untrusted_signing(Some(
                ApplicationOperationBodySigningSpec::from_untrusted(
                    "verification".into(),
                    actor,
                    provenance,
                ),
            )),
        ),
        vec![assigned],
    );
    let mut output = BinaryOutput::with_capacity(1024);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &13_u16.to_be_bytes());
    let budget = || {
        RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap()
    };
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(decode(&mut input, &mut budget()).unwrap(), body);
    assert!(input.is_finished());
    bytes[..2].copy_from_slice(&12_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    assert_eq!(
        decode(&mut BinaryInput::new(&bytes), &mut budget)
            .unwrap_err()
            .kind(),
        Kind::UnsupportedRecordVariant
    );
}
