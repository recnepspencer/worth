use super::*;

#[test]
fn equal_source_membership_cannot_reuse_a_different_input_digest() {
    let selection = WorthQueryObservedSourceSelection::selecting_for_test(1);
    let edition = InstalledProducerEdition::for_test([0x51; 32]);
    let original = PreparedInputReuseKey::new(selection.clone(), [0x61; 32], edition);
    let continued = original.continued_by_republication();
    assert!(
        original
            .same_prepared_input_as(&continued, |_| Ok::<_, ()>(()))
            .unwrap(),
        "equal complete prepared input keys must permit reuse"
    );
    let changed = PreparedInputReuseKey::new(selection, [0x62; 32], edition);
    assert!(
        !original
            .same_prepared_input_as(&changed, |_| Ok::<_, ()>(()))
            .unwrap(),
        "reuse must compare the input digest even when source membership is unchanged"
    );
}
