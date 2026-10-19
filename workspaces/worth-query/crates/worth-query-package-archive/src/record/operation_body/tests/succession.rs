//! Transport preserves succession meaning but confers no completed-source authority.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodySuccession;
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationOutputPosture as Posture,
        ApplicationMutationOutputRoleCardinality as Cardinality,
        ApplicationMutationOutputRoleDescription,
    },
    application_schema::ApplicationOperationBodySuccessionOutputMatch as OutputMatch,
};

#[test]
fn succession_output_match_v23_roundtrips_and_older_profiles_refuse_instead_of_dropping_it() {
    // Transport-only world: schema/installation validity is proved by the supplier's
    // real portable readmission and guarded journey, not these descriptive literals.
    let succession = ApplicationOperationBodySuccession::from_untrusted(
        "workflow".into(),
        "renew".into(),
        "item".into(),
    );
    let matching = OutputMatch::from_untrusted(
        "selected".into(),
        "certify".into(),
        ApplicationMutationOutputRoleDescription {
            name: "round".into(),
            entity: "Round".into(),
            posture: Posture::Preserve,
            cardinality: Cardinality::ExactlyOne,
        },
    );
    let plain = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Item".into(),
        vec![],
        None,
        vec![],
    )
    .with_untrusted_succession(Some(succession.clone()));
    let body = plain
        .clone()
        .with_untrusted_succession(Some(succession.with_untrusted_output_match(matching)));
    let mut output = BinaryOutput::with_capacity(512);
    write(&mut output, &body).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &23_u16.to_be_bytes());
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
    assert_ne!(
        body.succession().unwrap().canonical_record(),
        plain.succession().unwrap().canonical_record()
    );
    for version in [21_u16, 22] {
        let mut downgraded = bytes.clone();
        downgraded[..2].copy_from_slice(&version.to_be_bytes());
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            downgraded.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        assert!(
            decode(&mut BinaryInput::new(&downgraded), &mut budget).is_err(),
            "v{version} must not ignore the match"
        );
    }
    let mut old = BinaryOutput::with_capacity(256);
    write(&mut old, &plain).unwrap();
    assert_eq!(&old.into_bytes()[..2], &21_u16.to_be_bytes());
}

#[test]
fn succession_v21_roundtrips_and_cannot_be_silently_downgraded() {
    let plain = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Root".into(),
        vec![],
        None,
        vec![],
    );
    let body = plain.clone().with_untrusted_succession(Some(
        ApplicationOperationBodySuccession::from_untrusted(
            "workflow".into(),
            "binding".into(),
            "root".into(),
        ),
    ));
    assert_ne!(body, plain);
    let mut output = BinaryOutput::with_capacity(256);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &21_u16.to_be_bytes());
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
    bytes[..2].copy_from_slice(&20_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert!(decode(&mut input, &mut budget)
        .unwrap()
        .succession()
        .is_none());
    assert!(
        !input.is_finished(),
        "v20 cannot consume the v21 attachment"
    );
    for bytes in [vec![2], vec![1], vec![1, 0, 0]] {
        assert!(super::super::succession::decode(&mut BinaryInput::new(&bytes), 21).is_err());
    }
}
