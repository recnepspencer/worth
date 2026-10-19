//! Cold portable descriptions cannot grow unsigned creators or additional provenance slots.
use super::{validate_signed_creations, Creation, Member, Presence, Step, Target};
use crate::application_schema::{
    ApplicationFieldBindingLocus as Locus, ApplicationOperationBodyCaptureRole as Role,
    ApplicationOperationBodyProgram as Body, ApplicationOperationBodySigningSpec as Signing,
    ApplicationScalarValueBinding, ApplicationSigningProvenanceBinding,
    ApplicationSigningSubjectProvenanceBinding,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity as Identity;
use worth_foundational::facade::ScalarAspectType as Family;

fn members(extra_provenance: bool) -> Vec<Member> {
    let actor = Locus::new("Evidence", "Facts", "Actor");
    let provenance = Locus::new("Evidence", "Facts", "Provenance");
    let mut fields = vec![
        Step::PrincipalIdentity {
            field: actor.clone(),
            binding: Identity::from_untrusted("worth.rust.u64".into()),
            scalar_family: Family::UInt64,
            mutation_binding: "SignBinding".into(),
            principal_binding: "PrincipalBinding".into(),
            source: Locus::new("Principal", "Facts", "Id"),
        },
        Step::SigningProvenance {
            field: provenance.clone(),
            binding: ApplicationSigningProvenanceBinding::IDENTITY,
        },
    ];
    if extra_provenance {
        fields.push(Step::SigningProvenance {
            field: Locus::new("Evidence", "Facts", "Other"),
            binding: ApplicationSigningProvenanceBinding::IDENTITY,
        });
    }
    let creation = Creation::from_untrusted("Evidence".into(), "evidence".into(), fields)
        .with_untrusted_signing(Some(Signing::from_untrusted(
            "verification".into(),
            actor,
            provenance,
        )));
    let field = |name: &str, scalar_family, value_type: &str| Member::Field {
        entity: "Evidence".into(),
        aspect: "Facts".into(),
        field: name.into(),
        presence: Presence::Required,
        scalar_family,
        value_type: value_type.into(),
        unit: None,
        frame: None,
        writable: false,
        equality_queryable: false,
        unique: false,
    };
    vec![
        field("Actor", Family::UInt64, "worth.rust.u64"),
        field(
            "Provenance",
            Family::String,
            ApplicationSigningProvenanceBinding::IDENTITY.as_str(),
        ),
        Member::OperationBody {
            operation: "Sign".into(),
            body: Body::from_untrusted_in_scope("Draft".into(), vec![], Some(creation), vec![]),
        },
        Member::OperationProgram {
            operation: "Sign".into(),
            target: Target::Create {
                entity: "Evidence".into(),
            },
        },
    ]
}
#[test]
fn a_second_provenance_initializer_is_refused_even_when_one_matches_the_declared_slot() {
    assert_eq!(validate_signed_creations(&members(false)), Ok(()));
    assert!(validate_signed_creations(&members(true)).is_err());
}
#[test]
fn unsigned_alternative_creators_and_mutable_signed_fields_are_refused() {
    let mut alternative = members(false);
    alternative.push(Member::OperationProgram {
        operation: "Bypass".into(),
        target: Target::Create {
            entity: "Evidence".into(),
        },
    });
    assert!(validate_signed_creations(&alternative).is_err());
    let mut mutable = members(false);
    let Member::Field { writable, .. } = &mut mutable[0] else {
        unreachable!()
    };
    *writable = true;
    assert!(validate_signed_creations(&mutable).is_err());
    let mut missing = members(false);
    missing.push(Member::Field {
        entity: "Evidence".into(),
        aspect: "Facts".into(),
        field: "MissingSnapshot".into(),
        presence: Presence::Required,
        scalar_family: Family::UInt64,
        value_type: "worth.rust.u64".into(),
        unit: None,
        frame: None,
        writable: false,
        equality_queryable: false,
        unique: false,
    });
    assert!(validate_signed_creations(&missing).is_err());
}

fn captured_members(binding: Identity, role: Option<Role>, signed: bool) -> Vec<Member> {
    let mut members = members(false);
    let Member::Field { value_type, .. } = &mut members[1] else {
        unreachable!()
    };
    *value_type = binding.as_str().into();
    let Member::OperationBody { body, .. } = &mut members[2] else {
        unreachable!()
    };
    let creation = body.creation().unwrap();
    let mut fields = creation.fields().to_vec();
    let Step::SigningProvenance { binding: slot, .. } = &mut fields[1] else {
        unreachable!()
    };
    *slot = binding;
    let field = Locus::new("Evidence", "Facts", "Text");
    let source = Locus::new("Draft", "Facts", "Description");
    fields.push(match role {
        Some(role) => Step::CapturedScopeField {
            field,
            source,
            role,
            binding: Identity::from_untrusted("worth.rust.string".into()),
            scalar_family: Family::String,
            source_presence: Presence::Optional,
        },
        None => Step::PresentScopeField {
            field,
            source,
            binding: Identity::from_untrusted("worth.rust.string".into()),
            scalar_family: Family::String,
        },
    });
    let signing = signed.then(|| creation.signing().unwrap().clone());
    if !signed {
        fields.retain(|step| !matches!(step, Step::SigningProvenance { .. }));
    }
    *body = Body::from_untrusted_in_scope(
        "Draft".into(),
        vec![],
        Some(
            Creation::from_untrusted("Evidence".into(), "evidence".into(), fields)
                .with_untrusted_signing(signing),
        ),
        vec![],
    );
    members.push(Member::Field {
        entity: "Evidence".into(),
        aspect: "Facts".into(),
        field: "Text".into(),
        presence: Presence::Required,
        scalar_family: Family::String,
        value_type: "worth.rust.string".into(),
        unit: None,
        frame: None,
        writable: false,
        equality_queryable: false,
        unique: false,
    });
    if !signed {
        members.retain(
            |member| !matches!(member, Member::Field { field, .. } if field == "Provenance"),
        );
    }
    members
}

#[test]
fn certified_captures_need_explicit_roles_and_cannot_be_reinterpreted_as_legacy_or_unsigned() {
    let v2 = ApplicationSigningSubjectProvenanceBinding::IDENTITY;
    assert_eq!(
        validate_signed_creations(&captured_members(v2.clone(), None, false)),
        Ok(())
    );
    assert!(validate_signed_creations(&captured_members(v2.clone(), None, true)).is_err());
    for role in [Role::Subject, Role::AuditContext] {
        assert_eq!(
            validate_signed_creations(&captured_members(v2.clone(), Some(role), true)),
            Ok(())
        );
        assert!(validate_signed_creations(&captured_members(
            ApplicationSigningProvenanceBinding::IDENTITY,
            Some(role),
            true
        ))
        .is_err());
        assert!(
            validate_signed_creations(&captured_members(v2.clone(), Some(role), false)).is_err()
        );
    }
    // Old scope initializer meaning stays accepted with its old provenance profile.
    assert_eq!(
        validate_signed_creations(&captured_members(
            ApplicationSigningProvenanceBinding::IDENTITY,
            None,
            true
        )),
        Ok(())
    );
}

#[test]
fn capture_role_and_presence_each_change_canonical_schema_meaning() {
    use crate::application_schema::canonical_identity::{
        canonical_identity, ApplicationSchemaCanonicalHeader,
    };
    let identity = |members: &[Member]| {
        canonical_identity(
            ApplicationSchemaCanonicalHeader {
                owner: "capture-test",
                name: "Capture",
                major: 1,
                minor: 0,
            },
            members,
            &[],
        )
    };
    let subject = captured_members(
        ApplicationSigningSubjectProvenanceBinding::IDENTITY,
        Some(Role::Subject),
        true,
    );
    let audit = captured_members(
        ApplicationSigningSubjectProvenanceBinding::IDENTITY,
        Some(Role::AuditContext),
        true,
    );
    assert_ne!(identity(&subject), identity(&audit));
    let mut required = subject.clone();
    let Member::OperationBody { body, .. } = &mut required[2] else {
        unreachable!()
    };
    let creation = body.creation().unwrap();
    let mut fields = creation.fields().to_vec();
    let Step::CapturedScopeField {
        source_presence, ..
    } = fields.last_mut().unwrap()
    else {
        unreachable!()
    };
    *source_presence = Presence::Required;
    *body = Body::from_untrusted_in_scope(
        "Draft".into(),
        vec![],
        Some(
            Creation::from_untrusted("Evidence".into(), "evidence".into(), fields)
                .with_untrusted_signing(creation.signing().cloned()),
        ),
        vec![],
    );
    assert_ne!(identity(&subject), identity(&required));
}
