//! Counted work includes recursive length measurement and JSON's counting pass.
use super::Writer;
use std::cell::Cell;

#[test]
fn nested_authored_fields_charge_every_measurement_and_emission_visit() {
    let visits = Cell::new(0usize);
    let charge = |bytes| {
        visits.set(visits.get() + bytes);
        Ok(())
    };
    let mut encoded = Vec::new();
    Writer::new(
        &mut |bytes| {
            encoded.extend_from_slice(bytes);
            Ok(())
        },
        &charge,
    )
    .record(&mut |outer| outer.record(&mut |inner| inner.json(&"ab")))
    .unwrap();
    // JSON visits four bytes to count, then writes its eight-byte length and
    // four-byte body. Each enclosing record counts and emits its complete body.
    assert_eq!(visits.get(), 8 + 2 * (8 + 2 * (4 + 8 + 4)));
    assert_eq!(encoded.len(), 8 + 8 + 8 + 4);
    assert_eq!(&encoded[24..], b"\"ab\"");
}
