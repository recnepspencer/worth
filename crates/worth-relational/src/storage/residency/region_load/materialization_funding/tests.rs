use super::*;
#[test]
fn zero_length_vectors_have_no_backing_quote() {
    assert_eq!(vector::<u128>(0).unwrap(), 0);
    assert_eq!(column::<u64>(0).unwrap(), 0);
}
#[test]
fn vector_quote_covers_values_and_relocation() {
    for count in [1, 8, 31, 32, 33, 1024] {
        assert!(vector::<u128>(count).unwrap() >= array::<u128>(count).unwrap() * 2);
        assert!(column::<u64>(count).unwrap() >= array::<u64>(count).unwrap());
    }
}
#[test]
fn allocation_arithmetic_overflow_is_a_request_refusal() {
    assert!(matches!(
        times(u64::MAX, 2),
        Err(Failure::Request(
            crate::execution::RelationalExecutionDenialCause::MemoryOverflow
        ))
    ));
}
