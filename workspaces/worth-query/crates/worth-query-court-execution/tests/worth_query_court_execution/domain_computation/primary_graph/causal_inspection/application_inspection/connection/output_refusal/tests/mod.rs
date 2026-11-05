use worth_query_execution::facade::application_inspection::WorthQueryProducerOutputConnectionDenial::*;

#[test]
fn output_refusal_keeps_the_furthest_join_and_first_equally_deep_candidate() {
    let mut cause = NoDisclosedOutputCandidate;
    cause.include_for_test(ContractRevisionMismatch);
    cause.include_for_test(NativeRevisionNotCreatedByProducer);
    assert_eq!(cause, ContractRevisionMismatch);
    cause.include_for_test(CanonicalEffectAmbiguous);
    cause.include_for_test(CanonicalFieldUnchanged);
    assert_eq!(cause, CanonicalEffectAmbiguous);
    let mut reversed = NativeRevisionNotCreatedByProducer;
    reversed.include_for_test(ContractRevisionMismatch);
    assert_eq!(reversed, ContractRevisionMismatch);
}
