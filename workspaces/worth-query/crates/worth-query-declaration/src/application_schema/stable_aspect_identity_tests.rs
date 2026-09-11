use worth_foundational::facade::{
    AspectContractRevision, AspectIdentity, CanonicalBasisLocus, CanonicalBasisValue,
    CanonicalIntegerWidth, InternedString,
};

use super::{ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember};
use std::any::TypeId;

crate::worth_query_application_schema! {
    schema StableAspectSchema {
        owner: "worth.test",
        version: (1, 0),
        members: |schema| {
            schema
                .entity(Account::reference())
                .aspect(Account::reference(), AccountFacts::reference())
                .field(Account::reference(), Balance::reference())
        }
    }
}

crate::worth_query_entity!(Account for StableAspectSchema);
crate::worth_query_aspect!(
    AccountFacts for StableAspectSchema, Account;
    identity = AspectIdentity(0x9161_1f01),
    revision = AspectContractRevision(2),
);
crate::worth_query_field!(
    Balance for StableAspectSchema, Account, AccountFacts:
    u64 => super::U64ApplicationValueBinding, read_only, equality
);

#[test]
fn authored_identity_and_revision_survive_reference_and_erasure() {
    let reference = AccountFacts::reference();
    assert_eq!(reference.identity(), AspectIdentity(0x9161_1f01));
    assert_eq!(reference.revision(), AspectContractRevision(2));

    let declaration = StableAspectSchema::declaration().unwrap();
    let aspect = declaration
        .erased()
        .members()
        .iter()
        .find(|member| matches!(member, ApplicationSchemaMember::Aspect { .. }))
        .expect("the authored aspect is retained");
    assert!(matches!(
        aspect,
        ApplicationSchemaMember::Aspect {
            identity: AspectIdentity(0x9161_1f01),
            revision: AspectContractRevision(2),
            ..
        }
    ));
}

#[test]
fn canonical_schema_v11_identity_is_frozen_for_legacy_equivalent_meaning() {
    let declaration = StableAspectSchema::declaration().unwrap();
    let sequence = declaration.identity().canonical_basis().payload();
    assert_eq!(
        sequence.version().as_str(),
        "worth-query-application-schema-v11"
    );
    assert_eq!(
        canonical_entries_golden(sequence.entries()),
        "header.major=u32:1\nheader.minor=u32:0\nheader.name=text:StableAspectSchema\nheader.owner=text:worth.test\nmember-count=u64:3\nmember[0].entity=text:Account\nmember[0].kind=text:entity\nmember[1].aspect=text:AccountFacts\nmember[1].entity=text:Account\nmember[1].identity=u64:2439061249\nmember[1].kind=text:aspect\nmember[1].revision=u64:2\nmember[2].aspect=text:AccountFacts\nmember[2].entity=text:Account\nmember[2].equality-queryable=bool:true\nmember[2].field=text:Balance\nmember[2].kind=text:field\nmember[2].presence=text:required\nmember[2].scalar-family=text:uint64\nmember[2].unit=null\nmember[2].value-type=text:worth.rust.u64\nmember[2].writable=bool:false"
    );
    let entries = sequence.entries();
    let identity = canonical_entry(entries, ".identity");
    let revision = canonical_entry(entries, ".revision");
    assert!(identity.0 < revision.0);
    assert_eq!(
        identity.1,
        &CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: 0x9161_1f01,
        }
    );
    assert_eq!(
        revision.1,
        &CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: 2,
        }
    );
}

#[test]
fn aspect_member_order_is_canonical() {
    let first = ApplicationSchemaDeclarationBuilder::<StableAspectSchema>::for_schema()
        .entity(Account::reference())
        .aspect(Account::reference(), AccountFacts::reference())
        .field(Account::reference(), Balance::reference())
        .build()
        .unwrap();
    let reordered = ApplicationSchemaDeclarationBuilder::<StableAspectSchema>::for_schema()
        .field(Account::reference(), Balance::reference())
        .aspect(Account::reference(), AccountFacts::reference())
        .entity(Account::reference())
        .build()
        .unwrap();
    assert_eq!(first.identity(), reordered.identity());
}

#[test]
fn field_declaration_retains_the_exact_compiler_local_binding_recipe() {
    let declaration = StableAspectSchema::declaration().unwrap();
    let recipe = declaration
        .member_provenance()
        .field_bindings()
        .first()
        .expect("the field binding recipe is retained");

    assert_eq!(recipe.locus().entity(), "Account");
    assert_eq!(recipe.locus().aspect(), "AccountFacts");
    assert_eq!(recipe.locus().field(), "Balance");
    assert_eq!(recipe.binding_identity().as_str(), "worth.rust.u64");
    assert_eq!(
        recipe.binding_type(),
        TypeId::of::<super::U64ApplicationValueBinding>()
    );
    assert_eq!(recipe.value_type(), TypeId::of::<u64>());
    assert!(recipe.decode().is_some());
    assert!(recipe.identity_capable());
    assert!(recipe.signed_aggregate_decode().is_none());
    assert_eq!(
        (recipe.encode())(&42_u64).unwrap(),
        super::ApplicationValue::UInt64(42)
    );
    assert!((recipe.validate())(&"wrong value type").is_err());
}

fn canonical_entry<'a>(
    entries: &'a [worth_foundational::facade::CanonicalBasisEntry],
    suffix: &str,
) -> (usize, &'a CanonicalBasisValue) {
    entries
        .iter()
        .enumerate()
        .find_map(|(index, entry)| match entry.locus() {
            CanonicalBasisLocus::Named(InternedString::Raw(name)) if name.ends_with(suffix) => {
                Some((index, entry.value()))
            }
            _ => None,
        })
        .expect("the aspect canonical component is retained")
}

fn canonical_entries_golden(entries: &[worth_foundational::facade::CanonicalBasisEntry]) -> String {
    entries
        .iter()
        .map(|entry| {
            let CanonicalBasisLocus::Named(InternedString::Raw(locus)) = entry.locus() else {
                panic!("schema identity golden requires named raw loci")
            };
            let value = match entry.value() {
                CanonicalBasisValue::ExactText(InternedString::Raw(value)) => {
                    format!("text:{value}")
                }
                CanonicalBasisValue::UnsignedInteger {
                    width: CanonicalIntegerWidth::Bits32,
                    value,
                } => format!("u32:{value}"),
                CanonicalBasisValue::UnsignedInteger {
                    width: CanonicalIntegerWidth::Bits64,
                    value,
                } => format!("u64:{value}"),
                CanonicalBasisValue::Bool(value) => format!("bool:{value}"),
                CanonicalBasisValue::Null => "null".to_owned(),
                value => panic!("unexpected schema identity golden value: {value:?}"),
            };
            format!("{locus}={value}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
