//! Explicit fixed-width admission preserves the old scalar/domain wire contract.
use worth_query_consumer_values::PositiveLength;
use worth_query_decl::facade::application_schema::*;
use worth_query_topology_entry::TopologyLengthBinding;
#[test]
fn admitted_length_preserves_full_width_zero_and_wrong_family_meanings() {
    let mut observed = Vec::new();
    for value in [1, u64::MAX] {
        let raw = ApplicationValue::UInt64(value);
        let decoded = TopologyLengthBinding::decode_admitted(&raw, &mut |work, bytes| {
            observed.push((work, bytes));
            Ok(())
        })
        .unwrap();
        assert_eq!(PositiveLength::get(&decoded), value);
        assert_eq!(TopologyLengthBinding::encode(&decoded).unwrap(), raw);
    }
    assert_eq!(observed, [(1, 0), (1, 0)]);
    assert!(
        matches!(TopologyLengthBinding::decode_admitted(&ApplicationValue::UInt64(0),&mut |_,_| Ok(())),Err(ApplicationValueDecodeDenial::CodecRejected { binding_identity }) if binding_identity==TopologyLengthBinding::IDENTITY)
    );
    let wrong =
        TopologyLengthBinding::decode_admitted(&ApplicationValue::Bool(true), &mut |_, _| {
            panic!("wrong family never enters the UInt64 decoder")
        });
    assert!(
        matches!(wrong,Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch { binding_identity,.. }) if binding_identity==TopologyLengthBinding::IDENTITY)
    );
    assert!(TopologyLengthBinding::canonical_contract().is_none());
    assert!(TopologyLengthBinding::NATIVE_INPUT_ADMISSION.is_none());
}
#[test]
fn original_admission_refusal_precedes_even_a_zero_domain_refusal() {
    let denied =
        TopologyLengthBinding::decode_admitted(&ApplicationValue::UInt64(0), &mut |work, bytes| {
            assert_eq!((work, bytes), (1, 0));
            Err(ValueCodecDenial::Admission)
        });
    assert!(
        matches!(denied,Err(ApplicationValueDecodeDenial::Canonical { binding_identity,cause:ValueCodecDenial::Admission }) if binding_identity==TopologyLengthBinding::IDENTITY)
    );
}
