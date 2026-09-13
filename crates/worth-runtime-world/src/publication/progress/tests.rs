use super::{
    owner_effect_count_from_postures, RelationalAttemptProgressPosture,
    SignalAttemptProgressPosture,
};

#[test]
fn owner_effect_projection_covers_zero_one_and_two_performed_owners() {
    let cases = [
        (
            RelationalAttemptProgressPosture::Prepared,
            SignalAttemptProgressPosture::PreparedForExecution,
            0,
        ),
        (
            RelationalAttemptProgressPosture::Performed,
            SignalAttemptProgressPosture::PreparedForExecution,
            1,
        ),
        (
            RelationalAttemptProgressPosture::Settled,
            SignalAttemptProgressPosture::Performed,
            2,
        ),
    ];
    for (relational, signal, expected) in cases {
        assert_eq!(
            owner_effect_count_from_postures(relational, signal),
            expected
        );
    }
}
