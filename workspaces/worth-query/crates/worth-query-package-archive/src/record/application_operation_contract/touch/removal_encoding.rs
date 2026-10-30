//! The new portable touch appends tag7; field writes retain tag3 and their payload.
use super::*;
use worth_foundational::facade::*;
#[test]
fn portable_aspect_removal_round_trips_exact_contract_with_appended_tag() {
    let shape = aspects()
        .struct_fields()
        .required("left", ScalarAspectType::UInt64)
        .required("right", ScalarAspectType::UInt64)
        .finish()
        .unwrap();
    let contract = aspects()
        .contract()
        .for_key(AspectKey::new("input").unwrap())
        .identified_by(AspectIdentity(31))
        .at_revision(AspectContractRevision(2))
        .struct_with(
            shape,
            AspectMaskContract::struct_fields(),
            AbsenceLaw::Optional,
            AspectEquivalenceBasis::DeclaredStructFields,
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        );
    let scopes = [
        Scope::WriteField {
            schema: "S".into(),
            entity: "E".into(),
            contract: contract.clone(),
            field_path: CanonicalFieldPath::single(FieldKey::new("left").unwrap()),
        },
        Scope::RemoveAspect {
            schema: "S".into(),
            entity: "E".into(),
            contract,
        },
    ];
    for (scope, tag) in scopes.into_iter().zip([3u16, 7]) {
        let mut output = crate::binary_output::BinaryOutput::with_capacity(1024);
        write(&mut output, &scope).unwrap();
        let bytes = output.into_bytes();
        assert_eq!(&bytes[..2], &tag.to_be_bytes());
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            crate::limits::WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        assert_eq!(
            decode(&mut BinaryInput::new(&bytes), &mut budget).unwrap(),
            scope
        );
    }
}
