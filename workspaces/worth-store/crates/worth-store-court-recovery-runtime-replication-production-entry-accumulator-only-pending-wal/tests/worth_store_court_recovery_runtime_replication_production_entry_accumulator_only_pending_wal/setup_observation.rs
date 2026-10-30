use worth_store_testkit_recovery_runtime::setup_observation::{
    SetupConstructionInputs, SetupObservation, SetupObservationDisposition,
};

#[test]
fn the_same_typed_construction_inputs_have_the_same_fingerprint() {
    let construct = || {
        let mut inputs = SetupConstructionInputs::default();
        inputs.unsigned("target", 4);
        inputs.bytes("artifact", &[1, 2, 3]);
        inputs.boolean("checkpoint", true);
        inputs
    };
    assert_eq!(
        construct().fingerprint("journal-prefix"),
        construct().fingerprint("journal-prefix")
    );
    assert_ne!(
        construct().fingerprint("journal-prefix"),
        construct().fingerprint("checkpoint-prefix")
    );
}

#[test]
fn field_lengths_names_types_and_order_cannot_alias_a_different_recipe() {
    let mut separated = SetupConstructionInputs::default();
    separated.bytes("payload", b"ab");
    separated.bytes("payload", b"c");
    let mut regrouped = SetupConstructionInputs::default();
    regrouped.bytes("payload", b"a");
    regrouped.bytes("payload", b"bc");
    assert_ne!(
        separated.fingerprint("recipe"),
        regrouped.fingerprint("recipe")
    );

    let mut first_name = SetupConstructionInputs::default();
    first_name.bytes("a", b"bc");
    let mut second_name = SetupConstructionInputs::default();
    second_name.bytes("ab", b"c");
    assert_ne!(
        first_name.fingerprint("recipe"),
        second_name.fingerprint("recipe")
    );

    let mut scalar = SetupConstructionInputs::default();
    scalar.unsigned("value", 1);
    let mut bytes = SetupConstructionInputs::default();
    bytes.bytes("value", &1_u64.to_le_bytes());
    assert_ne!(scalar.fingerprint("recipe"), bytes.fingerprint("recipe"));

    let mut reordered = SetupConstructionInputs::default();
    reordered.bytes("payload", b"c");
    reordered.bytes("payload", b"ab");
    assert_ne!(
        separated.fingerprint("recipe"),
        reordered.fingerprint("recipe")
    );
}

#[test]
fn disabled_observation_does_not_construct_inputs_or_touch_files() {
    let called = std::cell::Cell::new(false);
    let result = SetupObservation::disabled()
        .record("disabled-builder", &|_| called.set(true))
        .unwrap();
    assert_eq!(result, SetupObservationDisposition::Disabled);
    assert!(!called.get());
}
