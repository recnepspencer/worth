use worth_query_execution::facade::application_inspection::WorthQueryProducerOutputConnectionDenial::*;

#[test]
fn output_refusal_keeps_the_furthest_join_and_first_equally_deep_candidate() {
    let mut cause = NoDisclosedOutputCandidate;
    cause.include(ContractRevisionMismatch);
    cause.include(NativeRevisionNotCreatedByProducer);
    assert_eq!(cause, ContractRevisionMismatch);
    cause.include(CanonicalEffectAmbiguous);
    cause.include(CanonicalFieldUnchanged);
    assert_eq!(cause, CanonicalEffectAmbiguous);
    let mut reversed = NativeRevisionNotCreatedByProducer;
    reversed.include(ContractRevisionMismatch);
    assert_eq!(reversed, ContractRevisionMismatch);
}
