//! Cold portable descriptions cannot grow unsigned creators or additional provenance slots.
use super::{
    validate_signed_creations, ApplicationScalarValueBinding, ApplicationSigningProvenanceBinding,
    Creation, Member, Presence, Step, Target,
};
use crate::application_schema::{
    ApplicationFieldBindingLocus as Locus, ApplicationOperationBodyProgram as Body,
    ApplicationOperationBodySigningSpec as Signing,
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
