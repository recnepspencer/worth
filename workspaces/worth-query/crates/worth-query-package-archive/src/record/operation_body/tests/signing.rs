//! Legacy signing uses v13; classified captures use v14, never a live proof.
use super::{
    decode, write, ApplicationFieldBindingLocus as Locus,
    ApplicationOperationBodyCreatedField as Field, ApplicationOperationBodyCreation,
    ApplicationOperationBodyProgram, BinaryInput, BinaryOutput, Kind, RecordDecodeAttempt,
    ScalarAspectType, WorthQueryPackageArchiveLimits, WorthQueryPortableTypeIdentity,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldPresence as Presence, ApplicationOperationBodyCaptureRole as Role,
    ApplicationOperationBodyRequirementExpression as Expression,
    ApplicationOperationBodyRequirementOperand as Operand,
    ApplicationOperationBodyRequirementProgram as Requirement,
    ApplicationOperationBodyRequirementSource as Source, ApplicationOperationBodySigningSpec,
    ApplicationScalarValueBinding, ApplicationSigningProvenanceBinding,
    ApplicationSigningSubjectProvenanceBinding,
};

fn body(binding: WorthQueryPortableTypeIdentity) -> ApplicationOperationBodyProgram {
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
    ApplicationOperationBodyProgram::from_untrusted_in_scope(
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
                        binding,
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
    )
}

#[test]
fn signed_creation_roundtrips_and_v12_refuses_its_new_source_opcodes() {
    let old = body(ApplicationSigningProvenanceBinding::IDENTITY);
    let certified = body(ApplicationSigningSubjectProvenanceBinding::IDENTITY);
    assert_ne!(old, certified);
    for body in [old, certified] {
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
}

fn classified(role: Role, presence: Presence) -> ApplicationOperationBodyProgram {
    let legacy = body(ApplicationSigningSubjectProvenanceBinding::IDENTITY);
    let creation = legacy.creation().unwrap();
    let mut fields = creation.fields().to_vec();
    let Field::PresentScopeField {
        field,
        binding,
        scalar_family,
        source,
    } = fields.remove(0)
    else {
        unreachable!()
    };
    fields.insert(
        0,
        Field::CapturedScopeField {
            field,
            binding,
            scalar_family,
            source,
            source_presence: presence,
            role,
        },
    );
    ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Draft".into(),
        vec![],
        Some(
            ApplicationOperationBodyCreation::from_untrusted(
                "Evidence".into(),
                "evidence".into(),
                fields,
            )
            .with_untrusted_signing(creation.signing().cloned()),
        ),
        legacy.requirements().to_vec(),
    )
}

#[test]
fn v14_retains_role_and_presence_and_v13_cannot_reinterpret_classified_captures() {
    let mut encodings = std::collections::BTreeSet::new();
    for role in [Role::Subject, Role::AuditContext] {
        for presence in [Presence::Required, Presence::Optional] {
            let body = classified(role, presence);
            let mut output = BinaryOutput::with_capacity(1024);
            write(&mut output, &body).unwrap();
            let mut bytes = output.into_bytes();
            assert_eq!(&bytes[..2], &14_u16.to_be_bytes());
            assert!(
                encodings.insert(bytes.clone()),
                "role and presence must each change portable meaning"
            );
            let mut budget = RecordDecodeAttempt::begin(
                Default::default(),
                bytes.len() as u64,
                WorthQueryPackageArchiveLimits::DEFAULT,
            )
            .unwrap();
            let mut input = BinaryInput::new(&bytes);
            assert_eq!(decode(&mut input, &mut budget).unwrap(), body);
            assert!(input.is_finished());
            bytes[..2].copy_from_slice(&13_u16.to_be_bytes());
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
    }
}
