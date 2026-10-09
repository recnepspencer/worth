use super::super::AdmittedFactKey;
use super::control;
use super::custody::{authority, request, SERIAL};
use worth_execution::ExecutionAllocationPolicy as Policy;
use worth_foundational::facade::{AspectValue, InternedString, Symbol};

#[test]
fn borrowed_predicate_writer_matches_independent_historical_scalar_bytes() {
    let cases = [
        (AspectValue::Null, "value.kind#4:null;"),
        (
            AspectValue::UInt16(65535),
            "value.kind#8:unsigned;value.width#3:i16;value.unsigned#5:65535;",
        ),
        (
            AspectValue::String(InternedString::Raw("\u{03b2};#:\n".into())),
            "value.kind#4:text;value.text.raw#6:\u{03b2};#:\n;",
        ),
        (
            AspectValue::String(InternedString::Symbol(Symbol(42))),
            "value.kind#4:text;value.text.symbol#2:42;",
        ),
    ];
    for (value, expected) in cases {
        let key = AdmittedFactKey::from_borrowed(
            |writer| write!(writer, "application-indexed-entity-selection:{}", 41),
            Some((&value, 2)),
            control(Policy::SystemAllocation),
        )
        .unwrap();
        assert_eq!(key.locator(), b"application-indexed-entity-selection:41");
        assert_eq!(key.predicate_material(), Some((expected.as_bytes(), 2)));
    }
}

#[test]
fn actual_borrowed_material_retains_its_key_byte_charge_past_lease_drop() {
    if !super::custody::isolated(
        "actual_borrowed_material_retains_its_key_byte_charge_past_lease_drop",
        module_path!(),
    ) {
        return;
    }
    let _serial = SERIAL.lock().unwrap();
    let expected = b"value.kind#4:null;";
    let quote = u64::try_from(1 + expected.len()).unwrap();
    let parent = authority().request_lease(request(quote)).unwrap();
    let key = {
        let child = parent.child(request(quote)).unwrap();
        AdmittedFactKey::from_borrowed(
            |writer| writer.write_str("i"),
            Some((&AspectValue::Null, 2)),
            control(Policy::Execution(&child)),
        )
        .unwrap()
    };
    assert_eq!(key.locator(), b"i");
    assert_eq!(key.predicate_material(), Some((expected.as_slice(), 2)));
    assert!(matches!(
        parent.reserve_memory(1),
        Err(memory) if memory == worth_execution::MemoryLimitDenial { requested: 1, admitted: 0, level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 } }
    ));
    drop(key);
    assert_eq!(parent.reserve_memory(quote).unwrap().bytes(), quote);
}
