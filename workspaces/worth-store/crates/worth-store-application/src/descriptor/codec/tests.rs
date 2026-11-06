//! Recorded bindings are descriptive; a changed revision never advances them.
use super::*;
use worth_foundational::facade::{ApplicationHomeBinding, CanonicalDigestId, HomeProgramBinding};

fn binding(schema: u8, revision: u8) -> ApplicationHomeBinding {
    let program = HomeProgramBinding::new(
        "recorded-program".into(),
        CanonicalDigestId::new([revision; 32]),
    );
    ApplicationHomeBinding::new(
        CanonicalDigestId::new([1; 32]),
        CanonicalDigestId::new([schema; 32]),
        program.clone(),
        vec![program],
    )
    .unwrap()
}

#[test]
fn changed_program_or_schema_requires_a_transition_preserving_the_recorded_binding() {
    let recorded = binding(2, 3);
    let descriptor = HomeDescriptor {
        identity: worth_foundational::facade::HomeIdentity::from_observed_bytes([1; 16]).unwrap(),
        incarnation: worth_foundational::facade::HomeIncarnation::initial(),
        binding: recorded.clone(),
        build: worth_foundational::facade::ApplicationBuildIdentity::parse(
            "11111111-1111-4111-8111-111111111111",
        )
        .unwrap(),
    };
    let mut binding_bytes = Vec::new();
    crate::descriptor::binding::emit(&recorded, &mut binding_bytes);
    let mut bytes = Vec::new();
    emit(
        &descriptor,
        "runtime",
        "profile",
        &binding_bytes,
        &mut bytes,
    );
    assert!(
        matches!(admit_record(&bytes, [1; 16], "runtime", "profile", &binding_bytes).unwrap(), DescriptorBindingAdmission::Exact(incarnation) if incarnation == descriptor.incarnation)
    );
    for requested in [binding(4, 3), binding(2, 5)] {
        let mut requested_bytes = Vec::new();
        crate::descriptor::binding::emit(&requested, &mut requested_bytes);
        let DescriptorBindingAdmission::TransitionRequired {
            recorded: found, ..
        } = admit_record(&bytes, [1; 16], "runtime", "profile", &requested_bytes).unwrap()
        else {
            panic!("a changed program or schema must name its transition requirement")
        };
        assert_eq!(crate::descriptor::binding::decode(found).unwrap(), recorded);
        assert_ne!(found, requested_bytes);
    }
    assert!(matches!(
        admit_record(
            &bytes,
            [1; 16],
            "another-runtime",
            "profile",
            &binding_bytes
        ),
        Err(Denial::BindingMismatch)
    ));
    assert_eq!(
        read_record(&bytes, [1; 16], "runtime", "profile")
            .unwrap()
            .incarnation,
        descriptor.incarnation
    );
}
