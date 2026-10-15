//! Isolated cold lifecycle law; the real supplier tests prove full schema/program installation.
use super::{validate, Body, Creation, Direction, Field, Member, Presence, Target};
use crate::application_schema::{
    ApplicationFieldBindingLocus as Locus, ApplicationOperationBodyCaptureRole as Role,
    ApplicationOperationBodyCreationLink as Link, ApplicationOperationBodyScopeField as ScopeField,
    ApplicationOperationBodySigningLifecycleSpec as Spec,
    ApplicationOperationBodySigningSpec as Signing, ApplicationRelationIntegrity,
    ApplicationScalarValueBinding, ApplicationSigningSubjectProvenanceBinding,
    ApplicationSigningSupersessionBinding,
};
use worth_foundational::facade::{AspectValue, ScalarAspectType};

fn fixture() -> (Body, Vec<Member>) {
    let spec = Spec {
        history_relation: "History".into(),
        current_relation: "Current".into(),
        source_entity: "Draft".into(),
        consequence_relation: "Supersessions".into(),
        consequence_field: Locus::new("Supersession", "Facts", "Record"),
        maximum_evidence: 32,
    };
    let creation = Creation::from_untrusted(
        "Evidence".into(),
        "evidence".into(),
        vec![
            Field::SigningProvenance {
                field: Locus::new("Evidence", "Facts", "Provenance"),
                binding: ApplicationSigningSubjectProvenanceBinding::IDENTITY,
            },
            Field::CapturedScopeField {
                field: Locus::new("Evidence", "Facts", "Text"),
                source: Locus::new("Draft", "Facts", "Description"),
                binding: crate::portable_identity::WorthQueryPortableTypeIdentity::from_untrusted(
                    "worth.rust.string".into(),
                ),
                scalar_family: ScalarAspectType::String,
                source_presence: Presence::Optional,
                role: Role::Subject,
            },
        ],
    )
    .with_untrusted_signing(Some(Signing::from_untrusted(
        "verification".into(),
        Locus::new("Evidence", "Facts", "Actor"),
        Locus::new("Evidence", "Facts", "Provenance"),
    )))
    .with_untrusted_links(
        ["History", "Current"]
            .into_iter()
            .map(|name| Link {
                relation: name.into(),
                from: "Draft".into(),
                to: "Evidence".into(),
                key: name.into(),
                direction: Direction::FromScope,
            })
            .collect(),
    )
    .with_untrusted_signing_lifecycle(Some(spec));
    let body = Body::from_untrusted_in_scope("Draft".into(), vec![], Some(creation), vec![]);
    let relation = |name: &str, from: &str, to: &str| Member::Relation {
        relation: name.into(),
        from: from.into(),
        to: to.into(),
        integrity: ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    };
    let members = vec![
        Member::OperationBody {
            operation: "Sign".into(),
            body: body.clone(),
        },
        Member::Field {
            entity: "Supersession".into(),
            aspect: "Facts".into(),
            field: "Record".into(),
            presence: Presence::Required,
            scalar_family: ScalarAspectType::String,
            value_type: ApplicationSigningSupersessionBinding::IDENTITY
                .as_str()
                .into(),
            unit: None,
            frame: None,
            writable: false,
            equality_queryable: false,
            unique: false,
        },
        relation("History", "Draft", "Evidence"),
        relation("Current", "Draft", "Evidence"),
        relation("Supersessions", "Evidence", "Supersession"),
    ];
    (body, members)
}

#[test]
fn ordinary_writers_cannot_manufacture_or_remove_lifecycle_truth() {
    let (body, members) = fixture();
    assert!(validate(&body, &members));
    let targets = [
        Target::Create {
            entity: "Supersession".into(),
        },
        Target::Delete {
            entity: "Supersession".into(),
        },
        Target::Write {
            entity: "Supersession".into(),
            aspect: "Facts".into(),
            field: "Record".into(),
        },
        Target::Link {
            relation: "Current".into(),
            from: "Draft".into(),
            to: "Evidence".into(),
        },
        Target::Unlink {
            relation: "History".into(),
            from: "Draft".into(),
            to: "Evidence".into(),
        },
        Target::Unlink {
            relation: "Current".into(),
            from: "Draft".into(),
            to: "Evidence".into(),
        },
        Target::Link {
            relation: "Supersessions".into(),
            from: "Evidence".into(),
            to: "Supersession".into(),
        },
    ];
    for target in targets {
        let mut forged = members.clone();
        forged.push(Member::OperationProgram {
            operation: "Bypass".into(),
            target,
        });
        assert!(!validate(&body, &forged));
    }
    let mut own = members;
    own.push(Member::OperationProgram {
        operation: "Sign".into(),
        target: Target::Link {
            relation: "Current".into(),
            from: "Draft".into(),
            to: "Evidence".into(),
        },
    });
    assert!(validate(&body, &own));
}
#[test]
fn self_mutation_and_wrong_consequence_shapes_fail_cold() {
    let (body, members) = fixture();
    let changing = Body::from_untrusted_in_scope(
        "Draft".into(),
        vec![ScopeField::Assign {
            field: Locus::new("Draft", "Facts", "Description"),
            binding: crate::portable_identity::WorthQueryPortableTypeIdentity::from_untrusted(
                "worth.rust.string".into(),
            ),
            value: AspectValue::String(worth_foundational::facade::InternedString::Raw(
                "new".into(),
            )),
        }],
        body.creation().cloned(),
        vec![],
    );
    assert!(!validate(&changing, &members));
    let mut writable_members = members.clone();
    let Member::Field { writable, .. } = &mut writable_members[1] else {
        unreachable!()
    };
    *writable = true;
    assert!(!validate(&body, &writable_members));
    let mut malformed = members;
    let Member::Field { presence, .. } = &mut malformed[1] else {
        unreachable!()
    };
    *presence = Presence::Optional;
    assert!(!validate(&body, &malformed));
}
#[test]
fn lifecycle_associations_and_limits_are_canonical_schema_meaning() {
    use crate::application_schema::canonical_identity::{
        canonical_identity, ApplicationSchemaCanonicalHeader,
    };
    let (body, _) = fixture();
    let identity = |body| {
        canonical_identity(
            ApplicationSchemaCanonicalHeader {
                owner: "test",
                name: "Lifecycle",
                major: 1,
                minor: 0,
            },
            &[Member::OperationBody {
                operation: "Sign".into(),
                body,
            }],
            &[],
        )
    };
    let base = identity(body.clone());
    for index in 0..5 {
        let mut spec = body
            .creation()
            .unwrap()
            .signing_lifecycle()
            .unwrap()
            .clone();
        match index {
            0 => spec.history_relation.push('2'),
            1 => spec.current_relation.push('2'),
            2 => spec.consequence_relation.push('2'),
            3 => spec.consequence_field = Locus::new("Other", "Facts", "Record"),
            _ => spec.maximum_evidence += 1,
        }
        let changed = Body::from_untrusted_in_scope(
            "Draft".into(),
            vec![],
            Some(
                body.creation()
                    .unwrap()
                    .clone()
                    .with_untrusted_signing_lifecycle(Some(spec)),
            ),
            vec![],
        );
        assert_ne!(base, identity(changed));
    }
}
