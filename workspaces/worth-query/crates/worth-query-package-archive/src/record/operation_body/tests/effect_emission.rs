//! V43 carries a body's ordinary effect; no older profile can drop it, and an
//! unknown presence tag refuses.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyEffectEmission as Emission;

fn body() -> ApplicationOperationBodyProgram {
    ApplicationOperationBodyProgram::from_untrusted_in_scope(
        ARCHIVE_DECODE,
        "Inbox".into(),
        vec![],
        None,
        vec![],
    )
    .with_untrusted_effect_emission(
        ARCHIVE_DECODE,
        Some(Emission {
            effect: "ProjectUpdatedEffect".into(),
            input_binding: WorthQueryPortableTypeIdentity::from_untrusted(
                ARCHIVE_DECODE,
                "rename.input".into(),
            ),
            payload_binding: WorthQueryPortableTypeIdentity::from_untrusted(
                ARCHIVE_DECODE,
                "project.updated".into(),
            ),
        }),
    )
}

fn budget(bytes: &[u8]) -> RecordDecodeAttempt {
    RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap()
}

#[test]
fn effect_emission_v43_roundtrips_and_refuses_older_profiles_and_unknown_tags() {
    let body = body();
    let mut output = BinaryOutput::with_capacity(1024);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &43_u16.to_be_bytes());
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(decode(&mut input, &mut budget(&bytes)).unwrap(), body);
    assert!(input.is_finished());
    // A pre-v43 profile has no emission slot, so the effect cannot be dropped.
    let denied = super::super::effect_emission::write(
        &mut BinaryOutput::with_capacity(256),
        body.effect_emission(),
        42,
    )
    .unwrap_err();
    assert_eq!(denied.kind(), Kind::UnsupportedRecordVariant);
    // The presence tag immediately precedes the length-prefixed effect name.
    let effect = [&20_u32.to_be_bytes()[..], b"ProjectUpdatedEffect"].concat();
    let tag = bytes
        .windows(effect.len())
        .position(|window| window == effect)
        .unwrap()
        - 1;
    assert_eq!(bytes[tag], 1);
    bytes[tag] = 2;
    let refused = decode(&mut BinaryInput::new(&bytes), &mut budget(&bytes)).unwrap_err();
    assert_eq!(refused.kind(), Kind::InvalidBooleanEncoding);
    // Bodies without an emission keep their historical profile.
    let plain = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        ARCHIVE_DECODE,
        "Inbox".into(),
        vec![],
        None,
        vec![],
    );
    let mut output = BinaryOutput::with_capacity(256);
    write(&mut output, &plain).unwrap();
    assert_ne!(&output.into_bytes()[..2], &43_u16.to_be_bytes());
}
