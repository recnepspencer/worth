use super::*;

// A direct domain-contract constructor cannot turn an unmarked structured
// binding into a leaf claim that bypasses the source guard.
struct Unmarked;
impl ApplicationScalarValueBinding for Unmarked {
    type Value = String;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "fixture.unmarked-domain.v1";
    const SCALAR_FAMILY: worth_foundational::facade::ScalarAspectType =
        worth_foundational::facade::ScalarAspectType::String;
    const SUPPORTS_ADMITTED_DECODE: bool = true;
    fn canonical_contract() -> Option<ApplicationCanonicalValueContract> {
        Some(ApplicationCanonicalValueContract::domain_of::<String>(
            BOUNDS,
        ))
    }
    fn validate(_: &String) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
    fn encode(
        _: &String,
    ) -> Result<worth_foundational::facade::AspectValue, ApplicationValueEncodeDenial> {
        panic!("refuse before encoding effects")
    }
}
impl ApplicationReadableScalarValueBinding for Unmarked {
    fn decode(
        _: &worth_foundational::facade::AspectValue,
    ) -> Result<String, ApplicationValueDecodeDenial> {
        panic!("refuse before decoding effects")
    }
    fn decode_admitted(
        _: &worth_foundational::facade::AspectValue,
        _: &mut ValueDecodeAdmission<'_>,
    ) -> Result<String, ApplicationValueDecodeDenial> {
        panic!("refuse before admitted decoding effects")
    }
}

#[test]
fn an_unmarked_binding_cannot_satisfy_a_domain_atom_contract() {
    let declaration = UnmarkedSchema::declaration().unwrap();
    let error = index::<UnmarkedSchema>()
        .bind_application_schema(declaration)
        .unwrap_err();
    assert_eq!(
        error.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::FieldBindingContractMismatch
    );
    let details = format!("{error:?}");
    assert!(details.contains("ContractMismatch"), "{details}");
    assert!(details.contains("UnmarkedField"), "{details}");
}

struct RawBytes;
impl ApplicationScalarValueBinding for RawBytes {
    type Value = String;
    type Unit = ();
    type Decode = ApplicationValueDecodeUnavailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "fixture.raw-bytes.v1";
    const SCALAR_FAMILY: worth_foundational::facade::ScalarAspectType =
        worth_foundational::facade::ScalarAspectType::InlineBytes;
    fn validate(_: &String) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
    fn encode(
        _: &String,
    ) -> Result<worth_foundational::facade::AspectValue, ApplicationValueEncodeDenial> {
        panic!("predicate admission must precede encoding")
    }
}

#[test]
fn domain_and_inline_byte_fields_refuse_semantic_equality_before_codec_effects() {
    // These are actual authored field contracts. The independent literal
    // subjects distinguish the canonical-domain and raw-byte carrier paths.
    let domain = DomainEquality::declaration().unwrap();
    let raw = RawEquality::declaration().unwrap();
    for (declaration, subject) in [
        (
            domain.erased(),
            format!(
                "DomainEntity:{}:DomainField",
                DomainField::at("DomainField").aspect()
            ),
        ),
        (
            raw.erased(),
            format!("RawEntity:{}:RawField", RawField::at("RawField").aspect()),
        ),
    ] {
        let refusal = collect_declared_fields(declaration).unwrap_err();
        assert_eq!(refusal.kind(), DenialKind::ContractMismatch);
        assert_eq!(refusal.subject(), subject);
    }
}

#[path = "domain_posture/unmarked_schema.rs"]
mod unmarked_schema;
use unmarked_schema::*;
#[path = "domain_posture/domain_equality.rs"]
mod domain_equality;
use domain_equality::*;
#[path = "domain_posture/raw_equality.rs"]
mod raw_equality;
use raw_equality::*;

impl ApplicationReadableScalarValueBinding for RawBytes {
    fn decode(
        _: &worth_foundational::facade::AspectValue,
    ) -> Result<String, ApplicationValueDecodeDenial> {
        panic!("predicate admission must precede decoder entry")
    }
}
