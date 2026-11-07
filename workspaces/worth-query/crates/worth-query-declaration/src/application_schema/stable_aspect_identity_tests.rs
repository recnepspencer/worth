use worth_foundational::facade::{
    AspectContractRevision, AspectIdentity, CanonicalBasisLocus, CanonicalBasisValue,
    CanonicalIntegerWidth, InternedString,
};

use super::{ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember};
use std::any::TypeId;

use super::fragment::*;
use super::{
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
    ValueDecodeAdmission,
};
struct StableAspectSchema;
impl Fragment for StableAspectSchema {
    const NAME: &'static str = "StableAspectSchema";
    type Bindings = Bindings;
    type Members = (Account,);
}
impl ApplicationSchema for StableAspectSchema {
    const OWNER: &'static str = "worth.test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
}
struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        use_admission: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        use_admission(work)
    }
}
impl Scalar<u64> for Bindings {
    type Binding = super::U64ApplicationValueBinding;
    const WRITABLE: bool = false;
}
struct Account;
impl Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (AccountFacts,);
    type Relations = ();
}
struct AccountFacts;
impl Aspect for AccountFacts {
    const NAME: &'static str = "AccountFacts";
    type Row = IdentityRow;
}
fn aspect_contract(
    declaration: &super::ErasedApplicationSchemaDeclaration,
) -> (AspectIdentity, AspectContractRevision) {
    declaration
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::Aspect {
                identity,
                revision,
                absence: worth_foundational::facade::AbsenceLaw::Required,
                ..
            } => Some((*identity, *revision)),
            _ => None,
        })
        .expect("the declared required aspect is retained")
}
#[test]
fn derived_identity_and_revision_are_stable_and_survive_erasure() {
    let first = StableAspectSchema::declaration().unwrap();
    let second = StableAspectSchema::declaration().unwrap();
    assert_eq!(
        aspect_contract(first.erased()),
        aspect_contract(second.erased())
    );
    assert_eq!(first.identity(), second.identity());
}

#[test]
fn canonical_schema_current_identity_contains_all_description_axes() {
    let declaration = StableAspectSchema::declaration().unwrap();
    let sequence = declaration.identity().canonical_basis().payload();
    assert_eq!(
        sequence.version().as_str(),
        "worth-query-application-schema-v18"
    );
    let (AspectIdentity(aspect_identity), AspectContractRevision(aspect_revision)) =
        aspect_contract(declaration.erased());
    assert_eq!(
        canonical_entries_golden(sequence.entries()),
        format!("contribution-count=u64:0\nheader.major=u32:1\nheader.minor=u32:0\nheader.name=text:StableAspectSchema\nheader.owner=text:worth.test\nmember-count=u64:3\nmember[0].entity=text:Account\nmember[0].kind=text:entity\nmember[1].absence=text:required\nmember[1].aspect=text:a5afdd3aaadd07ed2\nmember[1].entity=text:Account\nmember[1].identity=u64:{aspect_identity}\nmember[1].kind=text:aspect\nmember[1].revision=u64:{aspect_revision}\nmember[2].aspect=text:a5afdd3aaadd07ed2\nmember[2].entity=text:Account\nmember[2].equality-queryable=bool:true\nmember[2].field=text:value\nmember[2].frame=null\nmember[2].kind=text:field\nmember[2].presence=text:required\nmember[2].scalar-family=text:uint64\nmember[2].unique=bool:false\nmember[2].unit=null\nmember[2].value-type=text:worth.rust.u64\nmember[2].writable=bool:false")
    );
    let entries = sequence.entries();
    let identity = canonical_entry(entries, ".identity");
    let revision = canonical_entry(entries, ".revision");
    assert!(identity.0 < revision.0);
    assert_eq!(
        identity.1,
        &CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: u128::from(aspect_identity),
        }
    );
    assert_eq!(
        revision.1,
        &CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: u128::from(aspect_revision),
        }
    );
}

#[test]
fn aspect_member_order_is_canonical() {
    let declaration = StableAspectSchema::declaration().unwrap();
    let members = declaration.erased().members().to_vec();
    let first = ApplicationSchemaDeclarationBuilder::<StableAspectSchema>::from_test_members(
        members.clone(),
    )
    .build()
    .unwrap();
    let mut reversed = members;
    reversed.reverse();
    let reordered =
        ApplicationSchemaDeclarationBuilder::<StableAspectSchema>::from_test_members(reversed)
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
    assert_eq!(recipe.locus().aspect(), "a5afdd3aaadd07ed2");
    assert_eq!(recipe.locus().field(), "value");
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
            CanonicalBasisLocus::Named(InternedString::Stored(name))
                if name.as_str().ends_with(suffix) =>
            {
                let name = name.as_string();
                {
                    Some((index, entry.value()))
                }
            }
            CanonicalBasisLocus::Named(InternedString::Raw(name))
                if name.as_str().ends_with(suffix) =>
            {
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
                CanonicalBasisValue::ExactText(InternedString::Stored(value)) => {
                    let value = value.as_string();
                    {
                        format!("text:{value}")
                    }
                }
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

pub struct IdentityRow(u64);
impl super::fragment::Record<IdentityRow> for Bindings {
    fn record() -> impl super::fragment::Shape<IdentityRow, Self> {
        super::fragment::field("value", |row: &IdentityRow| row.0)
            .equality_queryable(true)
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(IdentityRow(value))
            })
    }
}
