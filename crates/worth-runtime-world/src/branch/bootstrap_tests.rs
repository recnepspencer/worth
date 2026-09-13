#[test]
fn bootstrap_intent_carries_exact_inputs_and_starts_at_generation_zero() {
    let fixture = crate::branch::reference_test_fixture::real_fixture(4, 4);
    let expected_relational = fixture.basis.relational_basis().clone();
    let expected_signal = fixture.basis.signal_basis().clone();
    let expected_correspondence = fixture.basis.correspondence_basis().clone();
    let intent = fixture.bootstrap_intent();
    let (creation, relational, signal, correspondence, generation) = intent.into_parts();

    assert_eq!(creation.name().as_str(), "root");
    assert_eq!(relational, expected_relational);
    assert_eq!(signal.branch_id(), expected_signal.branch_id());
    assert_eq!(
        signal.admission_identity(),
        expected_signal.admission_identity()
    );
    assert_eq!(correspondence, expected_correspondence);
    assert_eq!(generation.get(), 0);
}
